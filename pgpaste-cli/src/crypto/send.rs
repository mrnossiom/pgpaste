use super::POLICY;
use crate::ToEyreError;
use sequoia_openpgp::{
	serialize::stream::{Encryptor, LiteralWriter, Message, Signer},
	Cert,
};
use std::io::Write;

/// Signs the given message.
pub(crate) fn sign(plaintext: &str, sender: &Cert) -> eyre::Result<Vec<u8>> {
	// Get the keypair to do the signing from the Cert.
	let Some(valid_key) = sender
		.keys()
		.unencrypted_secret()
		.with_policy(POLICY, None)
		.alive()
		.revoked(false)
		.for_signing()
		.next() else
	{
		eyre::bail!("no signing key found");
	};

	let keypair = valid_key.key().clone().into_keypair().to_eyre()?;

	let mut signed_message: Vec<u8> = Vec::new();
	let message = Message::new(&mut signed_message);
	let signer = Signer::new(message, keypair).build().to_eyre()?;
	let mut literal = LiteralWriter::new(signer).build().to_eyre()?;

	literal.write_all(plaintext.as_bytes())?;
	literal.finalize().to_eyre()?;

	Ok(signed_message)
}

/// Signs the given message and protect it with a user password.
pub(crate) fn protect(plaintext: &str, sender: &Cert, password: &str) -> eyre::Result<Vec<u8>> {
	let Some(valid_key) = sender
		.keys()
		// TODO
		.unencrypted_secret()
		.with_policy(POLICY, None)
		.alive()
		.revoked(false)
		.supported()
		.for_signing()
		.next() else
	{
		eyre::bail!("no signing key found");
	};

	let keypair = valid_key.key().clone().into_keypair().to_eyre()?;

	// Start streaming an OpenPGP message.
	let mut signed_message: Vec<u8> = Vec::new();
	let message = Message::new(&mut signed_message);
	let signer = Signer::new(message, keypair).build().to_eyre()?;
	let encryptor = Encryptor::with_passwords(signer, [password])
		.build()
		.to_eyre()?;
	let mut literal = LiteralWriter::new(encryptor).build().to_eyre()?;

	literal.write_all(plaintext.as_bytes())?;
	literal.finalize().to_eyre()?;

	Ok(signed_message)
}

/// Encrypts the given message.
pub(crate) fn encrypt(plaintext: &str, sender: &Cert, recipient: &Cert) -> eyre::Result<Vec<u8>> {
	// Get the keypair to do the signing from the Cert.
	let Some(valid_key) = sender
		.keys()
		// TODO
		.unencrypted_secret()
		.with_policy(POLICY, None)
		.alive()
		.revoked(false)
		.supported()
		.for_signing()
		.next() else
	{
		eyre::bail!("no signing key found");
	};

	let keypair = valid_key.key().clone().into_keypair().to_eyre()?;

	let recipients = recipient
		.keys()
		.with_policy(POLICY, None)
		.alive()
		.revoked(false)
		.supported()
		.for_transport_encryption();

	let mut encrypted_message = Vec::new();
	let message = Message::new(&mut encrypted_message);
	let signer = Signer::new(message, keypair).build().to_eyre()?;
	let message = Encryptor::for_recipients(signer, recipients)
		.build()
		.to_eyre()?;
	let mut message = LiteralWriter::new(message).build().to_eyre()?;

	message.write_all(plaintext.as_bytes())?;
	message.finalize().to_eyre()?;

	Ok(encrypted_message)
}
