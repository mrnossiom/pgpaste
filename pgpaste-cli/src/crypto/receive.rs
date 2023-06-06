use crate::ToEyreError;
use eyre::WrapErr;
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
	policy::StandardPolicy,
	types::SymmetricAlgorithm,
	Cert, Fingerprint, KeyHandle, KeyID,
};
use std::{collections::HashMap, io};

// TODO: use the message date for the policy
const POLICY: &StandardPolicy = &StandardPolicy::new();

pub(crate) fn verify(message: &[u8], sender: Cert) -> eyre::Result<Vec<u8>> {
	let helper = Helper::new(&[sender], &[])?;
	let mut decryptor = VerifierBuilder::from_bytes(&message)
		.to_eyre()?
		.with_policy(POLICY, None, helper)
		.to_eyre()?;

	let mut out = Vec::new();
	io::copy(&mut decryptor, &mut out)?;

	Ok(out)
}

pub(crate) fn decrypt(ciphertext: &[u8], recipient: Cert) -> eyre::Result<Vec<u8>> {
	let helper = Helper::new(&[], &[recipient])?;
	let mut decryptor = DecryptorBuilder::from_bytes(ciphertext)
		.to_eyre()?
		.with_policy(POLICY, None, helper)
		.to_eyre()?;

	let mut out = Vec::new();
	io::copy(&mut decryptor, &mut out)?;

	Ok(out)
}

// TODO: implement local keyring
/// This helper provides secrets for the decryption, fetches public
/// keys for the signature verification and implements the
/// verification policy.
struct Helper {
	certs: Vec<Cert>,
	secret_keys: HashMap<KeyID, Key<SecretParts, UnspecifiedRole>>,
	hints: HashMap<KeyID, String>,
}

impl Helper {
	/// Creates a Helper for the given Certs with appropriate secrets.
	fn new(secrets: &[Cert], public_certs: &[Cert]) -> eyre::Result<Self> {
		let mut secret_keys: HashMap<KeyID, Key<SecretParts, UnspecifiedRole>> = HashMap::new();
		let mut hints = HashMap::new();

		for secret_cert in secrets {
			let identity = secret_cert
				.with_policy(POLICY, None)
				.and_then(|cert| cert.primary_userid())
				.ok()
				.map_or_else(
					|| format!("{}", secret_cert.keyid()),
					|uid| format!("{} ({})", uid.userid(), secret_cert.keyid()),
				);
			hints.insert(secret_cert.keyid(), identity);

			for ka in secret_cert
				.keys()
				.with_policy(POLICY, None)
				.for_transport_encryption()
				.for_storage_encryption()
			{
				let secret_key = ka
					.key()
					.parts_as_secret()
					.to_eyre()
					.wrap_err("Cert does not contain secret keys")?;

				secret_keys.insert(ka.key().keyid(), secret_key.clone());
			}
		}

		Ok(Self {
			certs: public_certs.to_vec(),
			secret_keys,
			hints,
		})
	}

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

impl VerificationHelper for Helper {
	fn get_certs(&mut self, ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
		let concerned_certs = self
			.certs
			.iter()
			.filter(|cert| {
				ids.iter().any(|handle| match handle {
					KeyHandle::Fingerprint(fin) => fin == &cert.fingerprint(),
					KeyHandle::KeyID(id) => id == &cert.keyid(),
				})
			})
			.cloned()
			.collect::<Vec<_>>();

		Ok(concerned_certs)
	}

	// TODO: implement message structure verification policy
	fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
		for layer in structure.iter() {
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
impl DecryptionHelper for Helper {
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
		// TODO: custom sessions keys

		// First, we try those keys that we can use without prompting
		// for a password.
		for pkesk in pkesks {
			if let Some(key) = self.secret_keys.get_mut(pkesk.recipient()) {
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
			if let Some(key) = self.secret_keys.get_mut(keyid) {
				let keypair: Box<dyn Decryptor> = loop {
					if let Ok(keypair) = key.clone().into_keypair() {
						break Box::new(keypair);
					}

					let prompt = format!(
						"Enter password to decrypt key {}: ",
						self.hints.get(keyid).unwrap()
					);
					let pass = rpassword::prompt_password(prompt)?.into();

					match decrypt_key(key, &pass) {
						Ok(decryptor) => break decryptor,
						Err(error) => eprintln!("Could not unlock key: {error:?}"),
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

			eprintln!("Bad password.");
		}
	}
}

fn decrypt_key(
	key: &mut Key<SecretParts, UnspecifiedRole>,
	pass: &crypto::Password,
) -> sequoia_openpgp::Result<Box<dyn Decryptor>> {
	let algo = key.pk_algo();
	key.secret_mut().decrypt_in_place(algo, pass)?;

	Ok(Box::new(key.clone().into_keypair()?))
}
