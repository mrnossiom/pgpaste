use super::POLICY;
use crate::ToEyreError;
use async_compat::Compat;
use eyre::ContextCompat;
use rpassword::prompt_password;
use sequoia_net::{KeyServer, Policy};
use sequoia_openpgp::{
	crypto::KeyPair,
	serialize::stream::{Encryptor, LiteralWriter, Message, Signer},
	Cert, KeyHandle, KeyID,
};
use std::{borrow::Cow, collections::HashMap, io::Write};

/// Signs the given message.
pub(crate) fn sign(plaintext: &str, helper: &SendHelper) -> eyre::Result<Vec<u8>> {
	let keypair = helper.signing_key()?;

	let mut signed_message: Vec<u8> = Vec::new();
	let message = Message::new(&mut signed_message);
	let signer = Signer::new(message, keypair).build().to_eyre()?;
	let mut literal = LiteralWriter::new(signer).build().to_eyre()?;

	literal.write_all(plaintext.as_bytes())?;
	literal.finalize().to_eyre()?;

	Ok(signed_message)
}

/// Signs the given message and protect it with a user password.
pub(crate) fn protect(
	plaintext: &str,
	helper: &SendHelper,
	password: &str,
) -> eyre::Result<Vec<u8>> {
	let keypair = helper.signing_key()?;

	// Start streaming an OpenPGP message.
	let mut signed_message: Vec<u8> = Vec::new();
	let message = Message::new(&mut signed_message);
	let signer = Signer::new(message, keypair).build().to_eyre()?;
	let literal = LiteralWriter::new(signer).build().to_eyre()?;
	let mut encryptor = Encryptor::with_passwords(literal, [password])
		.build()
		.to_eyre()?;

	encryptor.write_all(plaintext.as_bytes())?;
	encryptor.finalize().to_eyre()?;

	Ok(signed_message)
}

/// Encrypts the given message.
pub(crate) fn encrypt(
	plaintext: &str,
	helper: &SendHelper,
	recipient: KeyHandle,
) -> eyre::Result<Vec<u8>> {
	let keypair = helper.signing_key()?;

	let recipient_cert = helper.get_cert(recipient)?;
	let recipients = recipient_cert
		.keys()
		.with_policy(POLICY, None)
		.alive()
		.revoked(false)
		.supported()
		.for_transport_encryption();

	let mut encrypted_message = Vec::new();
	let message = Message::new(&mut encrypted_message);
	let signer = Signer::new(message, keypair).build().to_eyre()?;
	let literal = LiteralWriter::new(signer).build().to_eyre()?;
	let mut encryptor = Encryptor::for_recipients(literal, recipients)
		.build()
		.to_eyre()?;

	encryptor.write_all(plaintext.as_bytes())?;
	encryptor.finalize().to_eyre()?;

	Ok(encrypted_message)
}

#[derive(Debug)]
pub(crate) struct SendHelper<'a> {
	default_cert: &'a Cert,

	private_certs: &'a [Cert],
	public_certs: &'a [Cert],
	hints: HashMap<KeyID, String>,
}

impl<'a> SendHelper<'a> {
	/// Creates a Helper for the given Certs with appropriate secrets.
	pub(crate) fn new(
		default_cert_handle: &KeyHandle,
		private_certs: &'a [Cert],
		public_certs: &'a [Cert],
	) -> eyre::Result<Self> {
		let mut hints = HashMap::new();

		for secret_cert in private_certs {
			let identity = secret_cert
				.with_policy(POLICY, None)
				.and_then(|cert| cert.primary_userid())
				.ok()
				.map_or_else(
					|| format!("{}", secret_cert.keyid()),
					|uid| format!("{} ({})", uid.userid(), secret_cert.keyid()),
				);
			hints.insert(secret_cert.keyid(), identity);
		}

		let default_cert = private_certs
			.iter()
			.find(|c| &c.key_handle() == default_cert_handle)
			.ok_or_else(|| {
				eyre::eyre!("specified default cert is not part of the given private certs")
			})?;

		Ok(Self {
			default_cert,

			private_certs,
			public_certs,
			hints,
		})
	}

	pub(crate) fn signing_key(&self) -> eyre::Result<KeyPair> {
		let mut iter = self
			.default_cert
			.keys()
			.secret()
			.with_policy(POLICY, None)
			.alive()
			.revoked(false)
			.for_signing();

		let mut key = iter
			.next()
			.wrap_err("the default cert has no valid key for signing")?
			.key()
			.clone();

		if key.secret().is_encrypted() {
			loop {
				let password = prompt_password(format!(
					"Password to decrypt key {}: ",
					self.hints
						.get(&key.keyid())
						.expect("keyid come from the same source as hints")
				))?;

				let algo = key.pk_algo();
				if key
					.secret_mut()
					.decrypt_in_place(algo, &password.into())
					.is_ok()
				{
					break;
				};

				log::error!("Wrong password, try again");
			}
		}

		Ok(key.into_keypair().expect("key was decrypted"))
	}

	// TODO: get recipient key by handle using local keyring
	fn get_cert(&self, recipient: KeyHandle) -> eyre::Result<Cow<'a, Cert>> {
		let cached_cert = self
			.public_certs
			.iter()
			.find(|c| c.key_handle() == recipient);

		let cert = match cached_cert {
			Some(cert) => Cow::Borrowed(cert),
			None => Cow::Owned(fetch_key_handle(recipient)?),
		};

		Ok(cert)
	}
}

fn fetch_key_handle(key: KeyHandle) -> eyre::Result<Cert> {
	let mut key_server = KeyServer::keys_openpgp_org(Policy::Encrypted).to_eyre()?;

	smol::block_on(Compat::new(async { key_server.get(key).await.to_eyre() }))
}
