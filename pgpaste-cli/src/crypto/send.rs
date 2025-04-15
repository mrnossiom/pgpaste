//! Create and encrypt pastes.

use std::{borrow::Cow, collections::HashMap, io::Write};

use async_compat::Compat;
use eyre::ContextCompat;
use rpassword::prompt_password;
use sequoia_net::{KeyServer, Policy};
use sequoia_openpgp::{
	Cert, KeyHandle, KeyID,
	crypto::KeyPair,
	serialize::stream::{Encryptor2, LiteralWriter, Message, Signer},
};

use super::POLICY;
use crate::ToEyreError;

/// Signs the given message.
pub(crate) fn sign(content: &[u8], helper: &SendHelper) -> eyre::Result<Vec<u8>> {
	let keypair = helper.signing_key()?;

	let mut signed_message: Vec<u8> = Vec::new();
	let message = Message::new(&mut signed_message);
	let signer = Signer::new(message, keypair).build().to_eyre()?;
	let mut literal = LiteralWriter::new(signer).build().to_eyre()?;

	literal.write_all(content)?;
	literal.finalize().to_eyre()?;

	Ok(signed_message)
}

// IDEA: merge protect and encrypt into one function

/// Signs the given message and protect it with a user password.
pub(crate) fn protect(
	content: &[u8],
	helper: &SendHelper,
	password: &str,
	sign: bool,
) -> eyre::Result<Vec<u8>> {
	// Start streaming an OpenPGP message.
	let mut signed_message: Vec<u8> = Vec::new();
	let message = Message::new(&mut signed_message);

	let next = if sign {
		let keypair = helper.signing_key()?;
		Signer::new(message, keypair).build().to_eyre()?
	} else {
		message
	};

	let encryptor = Encryptor2::with_passwords(next, [password])
		.build()
		.to_eyre()?;
	let mut literal = LiteralWriter::new(encryptor).build().to_eyre()?;

	literal.write_all(content)?;
	literal.finalize().to_eyre()?;

	Ok(signed_message)
}

/// Encrypts the given message.
pub(crate) fn encrypt(
	content: &[u8],
	helper: &SendHelper,
	recipient: KeyHandle,
	sign: bool,
) -> eyre::Result<Vec<u8>> {
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

	let next = if sign {
		let keypair = helper.signing_key()?;
		Signer::new(message, keypair).build().to_eyre()?
	} else {
		message
	};

	let encryptor = Encryptor2::for_recipients(next, recipients)
		.build()
		.to_eyre()?;
	let mut literal = LiteralWriter::new(encryptor).build().to_eyre()?;

	literal.write_all(content)?;
	literal.finalize().to_eyre()?;

	Ok(encrypted_message)
}

/// A helper to create and encrypt pastes.
#[derive(Debug)]
pub(crate) struct SendHelper<'a> {
	/// The cert used for signing and/or encrypting.
	default_cert: &'a Cert,

	/// Private certs used for signing.
	_private_certs: &'a [Cert],
	/// Public certs used for encryption.
	public_certs: &'a [Cert],
	/// Hints used when prompting the user to decrypt their key.
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

			_private_certs: private_certs,
			public_certs,
			hints,
		})
	}

	/// The keypair to use when signing pastes
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

	/// Returns the cert for the given key handle whether it is in the cache or by fetching it
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

/// Fetches the given key from the wellknown `OpenPGP` keyserver.
fn fetch_key_handle(key: KeyHandle) -> eyre::Result<Cert> {
	let mut key_server = KeyServer::keys_openpgp_org(Policy::Encrypted).to_eyre()?;

	smol::block_on(Compat::new(async { key_server.get(key).await.to_eyre() }))
}
