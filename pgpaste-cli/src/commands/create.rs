use crate::{args::CreateArgs, config::Config};
use pgpaste_api_types::{
	api::{CreateBody, CreateResponse},
	Visibility,
};
use reqwest::{blocking::Client, header, Method, StatusCode, Url};
use rmp_serde::decode::from_slice;
use sequoia_openpgp::{
	policy::StandardPolicy,
	serialize::stream::{Encryptor, LiteralWriter, Message, Signer},
	Cert,
};
use std::io::Write;

pub(crate) fn create(args: &CreateArgs, config: &Config) -> eyre::Result<()> {
	// TODO
	let key = config.keys.clone().ok_or(eyre::eyre!(""))?;

	let content = args.content()?;

	let bytes = match args.mode {
		Visibility::Public => sign(&content, &key).map_err(|err| eyre::eyre!(Box::new(err)))?,
		Visibility::Private => encrypt(&content, &key).map_err(|err| eyre::eyre!(Box::new(err)))?,
		Visibility::Protected => todo!(),
	};

	let res = post(config.server.clone(), bytes, &args.slug, args)?;

	println!("Your paste is available with the slug `{}`", res.slug);

	Ok(())
}

fn post(
	mut server: Url,
	content: Vec<u8>,
	slug: &Option<String>,
	args: &CreateArgs,
) -> eyre::Result<CreateResponse> {
	let client = Client::default();

	let query = CreateBody {
		slug: slug.clone(),
		visibility: args.mode,
		burn_in: args.burn_in()?,
		burn_after_read: args.burn_after_read,
		inner: content,
	};

	server.set_path("/api/paste");

	let method = if args.overwrite {
		Method::PUT
	} else {
		Method::POST
	};

	let response = client
		.request(method, server)
		.header(header::CONTENT_TYPE, "application/msgpack")
		.body(rmp_serde::to_vec(&query)?)
		.send()?;

	let response = match response.status() {
		StatusCode::CREATED => from_slice::<CreateResponse>(&response.bytes()?)?,
		StatusCode::CONFLICT => eyre::bail!("Paste name already exists"),
		code => eyre::bail!("Unknown error: {}, {}", code, response.text()?),
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
