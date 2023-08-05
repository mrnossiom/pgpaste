//! Decrypting and verifying messages

use super::POLICY;
use crate::ToEyreError;
use sequoia_openpgp::{
	crypto::{self, Decryptor, SessionKey},
	packet::{
		self,
		key::{SecretParts, UnspecifiedRole},
		Key,
	},
	parse::{
		stream::{
			DecryptionHelper, DecryptorBuilder, MessageLayer, MessageStructure, VerificationHelper,
			VerifierBuilder,
		},
		Parse,
	},
	types::SymmetricAlgorithm,
	Cert, Fingerprint, KeyHandle, KeyID,
};
use std::{collections::HashMap, io};

/// Verify the given message with the given helper.
pub(crate) fn verify(message: &[u8], helper: ReceiveHelper) -> eyre::Result<Vec<u8>> {
	let mut decryptor = VerifierBuilder::from_bytes(&message)
		.to_eyre()?
		.with_policy(POLICY, None, helper)
		.to_eyre()?;

	let mut out = Vec::new();
	io::copy(&mut decryptor, &mut out)?;

	Ok(out)
}

/// Decrypt the given message with the given helper.
pub(crate) fn decrypt(ciphertext: &[u8], helper: ReceiveHelper) -> eyre::Result<Vec<u8>> {
	let mut decryptor = DecryptorBuilder::from_bytes(ciphertext)
		.to_eyre()?
		.with_policy(POLICY, None, helper)
		.to_eyre()?;

	let mut out = Vec::new();
	io::copy(&mut decryptor, &mut out)?;

	Ok(out)
}

/// This helper provides secrets for the decryption, fetches public
/// keys for the signature verification and implements the
/// verification policy.
#[derive(Debug)]
pub(crate) struct ReceiveHelper<'a> {
	/// The certs used for decrypting
	secrets: HashMap<KeyID, Key<SecretParts, UnspecifiedRole>>,
	/// The certs used for verification
	public_certs: &'a [Cert],
	/// Hints used when prompting the user to decrypt their key.
	hints: HashMap<KeyID, String>,
}

impl<'a> ReceiveHelper<'a> {
	/// Creates a Helper for the given Certs with appropriate secrets.
	pub(crate) fn new(private_certs: &[Cert], public_certs: &'a [Cert]) -> eyre::Result<Self> {
		let mut secrets: HashMap<KeyID, Key<SecretParts, UnspecifiedRole>> = HashMap::new();
		let mut hints = HashMap::new();

		for private_cert in private_certs {
			let identity = private_cert
				.with_policy(POLICY, None)
				.and_then(|cert| cert.primary_userid())
				.ok()
				.map_or_else(
					|| format!("{}", private_cert.keyid()),
					|uid| format!("{} ({})", uid.userid(), private_cert.keyid()),
				);

			log::debug!("found identity: {identity}");
			hints.insert(private_cert.keyid(), identity.clone());

			for ka in private_cert
				.keys()
				.with_policy(POLICY, None)
				.for_transport_encryption()
				.for_storage_encryption()
			{
				let secret_key = ka
					.key()
					.parts_as_secret()
					.to_wrap_err("cert does not contain secret keys")?;

				log::debug!("found secret key: {secret_key}");
				hints.insert(ka.keyid(), identity.clone());
				secrets.insert(ka.key().keyid(), secret_key.clone());
			}
		}

		Ok(Self {
			secrets,
			public_certs,
			hints,
		})
	}

	/// Returns the identity of the given key if it is able to decrypt it
	fn try_decrypt<D>(
		pkesk: &packet::PKESK,
		sym_algo: Option<SymmetricAlgorithm>,
		mut keypair: Box<dyn crypto::Decryptor>,
		decrypt: &mut D,
	) -> Option<Fingerprint>
	where
		D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool,
	{
		pkesk
			.decrypt(&mut *keypair, sym_algo)
			.and_then(
				|(algo, sk)| {
					if decrypt(algo, &sk) {
						Some(sk)
					} else {
						None
					}
				},
			)
			.map(|_sk| keypair.public().fingerprint())
	}
}

impl<'a> VerificationHelper for ReceiveHelper<'a> {
	fn get_certs(&mut self, ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
		let concerned_certs = self
			.public_certs
			.iter()
			.filter(|cert| {
				ids.iter()
					.any(|handle| handle == &cert.fingerprint().into())
			})
			.cloned()
			.collect::<Vec<_>>();

		Ok(concerned_certs)
	}

	// TODO: implement message structure verification policy
	fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
		for layer in &*structure {
			match layer {
				MessageLayer::Compression { .. } | MessageLayer::Encryption { .. } => {}
				MessageLayer::SignatureGroup { results } => {
					for result in results {
						if let Err(e) = result {
							anyhow::bail!("signature verification failed: {}", e)
						}
					}
				}
			}
		}

		Ok(())
	}
}

#[allow(clippy::similar_names)]
impl<'a> DecryptionHelper for ReceiveHelper<'a> {
	fn decrypt<D>(
		&mut self,
		pkesks: &[packet::PKESK],
		skesks: &[packet::SKESK],
		sym_algo: Option<SymmetricAlgorithm>,
		mut decrypt: D,
	) -> sequoia_openpgp::Result<Option<Fingerprint>>
	where
		D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool,
	{
		// First, we try those keys that we can use without prompting
		// for a password.
		for pkesk in pkesks {
			if let Some(key) = self.secrets.get_mut(pkesk.recipient()) {
				if let Some(fingerprint) =
					key.clone().into_keypair().ok().and_then(|kp| {
						Self::try_decrypt(pkesk, sym_algo, Box::new(kp), &mut decrypt)
					}) {
					return Ok(Some(fingerprint));
				}
			}
		}

		// Second, we try those keys that are encrypted.
		for pkesk in pkesks {
			// Don't ask the user to decrypt a key if we don't support
			// the algorithm.
			if !pkesk.pk_algo().is_supported() {
				continue;
			}

			let keyid = pkesk.recipient();
			if let Some(key) = self.secrets.get_mut(keyid) {
				let keypair: Box<dyn Decryptor> = loop {
					if let Ok(keypair) = key.clone().into_keypair() {
						break Box::new(keypair);
					}

					log::debug!("key {} is encrypted", keyid);

					let key_password = rpassword::prompt_password(format!(
						"Enter password to decrypt key {}: ",
						self.hints
							.get(keyid)
							.expect("keyid come from the same source as hints")
					))?;

					match decrypt_key(key, &key_password.into()) {
						Ok(decryptor) => break decryptor,
						Err(error) => log::error!("Could not unlock key: {error:?}"),
					}
				};

				if let Some(fp) = Self::try_decrypt(pkesk, sym_algo, keypair, &mut decrypt) {
					return Ok(Some(fp));
				}
			}
		}

		// TODO: wildcards recipients

		if skesks.is_empty() {
			return Err(anyhow::anyhow!("No key to decrypt message"));
		}

		// Finally, try to decrypt using the SKESKs.
		loop {
			let password =
				rpassword::prompt_password("Enter password to decrypt message: ")?.into();

			for skesk in skesks {
				if let Some(_sk) = skesk.decrypt(&password).ok().and_then(|(algo, sk)| {
					if decrypt(algo, &sk) {
						Some(sk)
					} else {
						None
					}
				}) {
					return Ok(None);
				}
			}

			log::error!("Bad password.");
		}
	}
}

/// Decrypts an encrypted secret key
fn decrypt_key(
	key: &mut Key<SecretParts, UnspecifiedRole>,
	pass: &crypto::Password,
) -> sequoia_openpgp::Result<Box<dyn Decryptor>> {
	let algo = key.pk_algo();
	key.secret_mut().decrypt_in_place(algo, pass)?;

	Ok(Box::new(key.clone().into_keypair()?))
}
