use crate::{args::CreateArgs, config::Config};
use pgpaste_api_types::{CreateQuery, CreateResponse, Visibility};
use reqwest::{blocking::Client, StatusCode, Url};
use sequoia_openpgp::{
	policy::StandardPolicy,
	serialize::stream::{Encryptor, LiteralWriter, Message, Signer},
	Cert,
};
use std::io::{stdin, Write};

pub(crate) fn create(args: &CreateArgs, config: &Config) -> eyre::Result<()> {
	// TODO
	let key = config.keys.clone().ok_or(eyre::eyre!(""))?;

	let content = if let Some(content) = &args.content {
		content.clone()
	} else if let Some(file) = &args.file {
		std::fs::read_to_string(file)?
	} else if atty::isnt(atty::Stream::Stdin) {
		std::io::read_to_string(stdin())?
	} else {
		eyre::bail!("I could not get paste content by a `--file`, a `--content` or stdin.")
	};

	let bytes = match args.mode {
		Visibility::Public => sign(&content, &key).map_err(|err| eyre::eyre!(Box::new(err)))?,
		Visibility::Private => encrypt(&content, &key).map_err(|err| eyre::eyre!(Box::new(err)))?,
		Visibility::Protected => todo!(),
	};

	let res = post(config.server.clone(), bytes, &args.slug, args)?;

	println!(
		"Your paste is available at {}",
		config.server.join(&res.slug)?
	);

	Ok(())
}

fn post(
	mut server: Url,
	content: Vec<u8>,
	slug: &Option<String>,
	args: &CreateArgs,
) -> eyre::Result<CreateResponse> {
	let client = Client::default();

	let query = CreateQuery {
		slug: slug.clone(),
		visibility: args.mode,
		overwrite: None,
	};

	server.set_path("/api/paste");

	let response = client.post(server).query(&query).body(content).send()?;

	let response: CreateResponse = match response.status() {
		StatusCode::CREATED => rmp_serde::from_slice(&response.bytes()?)?,
		StatusCode::CONFLICT => eyre::bail!("Paste name already exists"),
		code => eyre::bail!("Unknown error: {} ", code),
	};

	Ok(response)
}

/// Signs the given message.
fn sign(plaintext: &str, cert: &Cert) -> sequoia_openpgp::Result<Vec<u8>> {
	let mut signed_message = Vec::new();
	let policy = StandardPolicy::new();

	// Get the keypair to do the signing from the Cert.
	let Some(valid_key) = cert
		.keys()
		.unencrypted_secret()
		.with_policy(&policy, None)
		.alive()
		.revoked(false)
		.for_signing()
		.next() else {
			// TODO: WTF does sequoia return anyhow errors?
			// return Err(eyre::eyre!("no signing key found").into());
			panic!("no signing key found");
		};

	let keypair = valid_key.key().clone().into_keypair()?;

	// Start streaming an OpenPGP message.
	let message = Message::new(&mut signed_message);

	// We want to sign a literal data packet.
	let signer = Signer::new(message, keypair).build()?;

	// Emit a literal data packet.
	let mut literal = LiteralWriter::new(signer).build()?;

	// Sign the data.
	literal.write_all(plaintext.as_bytes())?;

	// Finalize the OpenPGP message to make sure that all data is
	// written.
	literal.finalize()?;

	Ok(signed_message)
}

/// Encrypts the given message.
fn encrypt(plaintext: &str, recipient: &Cert) -> sequoia_openpgp::Result<Vec<u8>> {
	let mut encrypted_message = Vec::new();
	let policy = StandardPolicy::new();

	let recipients = recipient
		.keys()
		.with_policy(&policy, None)
		.supported()
		.alive()
		.revoked(false)
		.for_transport_encryption();

	// Start streaming an OpenPGP message.
	let message = Message::new(&mut encrypted_message);

	// We want to encrypt a literal data packet.
	let message = Encryptor::for_recipients(message, recipients).build()?;

	// Emit a literal data packet.
	let mut message = LiteralWriter::new(message).build()?;

	// Encrypt the data.
	message.write_all(plaintext.as_bytes())?;

	// Finalize the OpenPGP message to make sure that all data is
	// written.
	message.finalize()?;

	Ok(encrypted_message)
}
