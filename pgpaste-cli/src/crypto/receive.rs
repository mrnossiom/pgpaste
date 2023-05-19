use super::POLICY;
use crate::ToEyreError;
use sequoia_openpgp::{
	crypto::{KeyPair, SessionKey},
	packet::{self},
	parse::{
		stream::{
			DecryptionHelper, DecryptorBuilder, GoodChecksum, MessageLayer, MessageStructure,
			VerificationHelper,
		},
		Parse,
	},
	policy::Policy,
	types::SymmetricAlgorithm,
	Cert, Fingerprint, KeyHandle, KeyID,
};
use std::{
	collections::HashMap,
	io::{self, Write},
};

pub(crate) fn verify(message: &[u8], cert: Cert) -> eyre::Result<Vec<u8>> {
	// Now, create a decryptor with a helper using the given Certs.
	let helper = Helper::new(vec![cert]);
	let mut decryptor = DecryptorBuilder::from_bytes(&message)
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
struct Helper {
	keys: HashMap<KeyID, (Fingerprint, KeyPair)>,
}

impl Helper {
	/// Creates a Helper for the given Certs with appropriate secrets.
	fn new(certs: Vec<Cert>) -> Self {
		// Map (sub)KeyIDs to primary fingerprints and secrets.
		let mut keys = HashMap::new();
		for cert in certs {
			for ka in cert
				.keys()
				.unencrypted_secret()
				.with_policy(POLICY, None)
				.supported()
				.for_storage_encryption()
				.for_transport_encryption()
			{
				keys.insert(
					ka.key().keyid(),
					(cert.fingerprint(), ka.key().clone().into_keypair().unwrap()),
				);
			}
		}

		Self { keys }
	}
}

impl DecryptionHelper for Helper {
	fn decrypt<D>(
		&mut self,
		pkesks: &[packet::PKESK],
		_skesks: &[packet::SKESK],
		sym_algo: Option<SymmetricAlgorithm>,
		mut decrypt: D,
	) -> sequoia_openpgp::Result<Option<Fingerprint>>
	where
		D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool,
	{
		// Try each PKESK until we succeed.
		let mut recipient = None;
		for pkesk in pkesks {
			if let Some((fp, pair)) = self.keys.get_mut(pkesk.recipient()) {
				if pkesk
					.decrypt(pair, sym_algo)
					.map_or(false, |(algo, session_key)| decrypt(algo, &session_key))
				{
					recipient = Some(fp.clone());
					break;
				}
			}
		}

		Ok(recipient)
	}
}

impl VerificationHelper for Helper {
	fn get_certs(&mut self, _ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
		Ok(Vec::new()) // Feed the Certs to the verifier here.
	}

	fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
		for layer in structure.iter() {
			match layer {
				MessageLayer::Compression { algo } => eprintln!("Compressed using {algo}"),
				MessageLayer::Encryption {
					sym_algo,
					aead_algo,
				} => aead_algo.map_or_else(
					|| {
						eprintln!("Encrypted using {sym_algo}");
					},
					|aead_algo| {
						eprintln!("Encrypted and protected using {sym_algo}/{aead_algo}");
					},
				),
				MessageLayer::SignatureGroup { ref results } => {
					for result in results {
						match result {
							Ok(GoodChecksum { ka, .. }) => {
								eprintln!("Good signature from {}", ka.cert());
							}
							Err(e) => eprintln!("Error: {e:?}"),
						}
					}
				}
			}
		}
		Ok(()) // Implement your verification policy here.
	}
}

pub(crate) fn decrypt(
	policy: &dyn Policy,
	sink: &mut dyn Write,
	ciphertext: &[u8],
	recipient: &Cert,
) -> sequoia_openpgp::Result<()> {
	// Make a helper that that feeds the recipient's secret key to the
	// decryptor.
	let helper = HelperTwo { secret: recipient };

	// Now, create a decryptor with a helper using the given Certs.
	let mut decryptor =
		DecryptorBuilder::from_bytes(ciphertext)?.with_policy(policy, None, helper)?;

	// Decrypt the data.
	io::copy(&mut decryptor, sink)?;

	Ok(())
}

struct HelperTwo<'a> {
	secret: &'a Cert,
}

impl<'a> VerificationHelper for HelperTwo<'a> {
	fn get_certs(&mut self, _ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
		// Return public keys for signature verification here.
		Ok(Vec::new())
	}

	fn check(&mut self, _structure: MessageStructure) -> sequoia_openpgp::Result<()> {
		// Implement your signature verification policy here.
		Ok(())
	}
}

impl<'a> DecryptionHelper for HelperTwo<'a> {
	fn decrypt<D>(
		&mut self,
		pkesks: &[packet::PKESK],
		_skesks: &[packet::SKESK],
		sym_algo: Option<SymmetricAlgorithm>,
		mut decrypt: D,
	) -> sequoia_openpgp::Result<Option<Fingerprint>>
	where
		D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool,
	{
		let Some(key) = self.secret
			.keys()
			// TODO
			.unencrypted_secret()
			.with_policy(POLICY, None)
			.alive()
			.revoked(false)
			.supported()
			.for_transport_encryption()
			.next() else
		{
			anyhow::bail!("no signing key found");
		};

		// The secret key is not encrypted.
		let mut pair = key.key().clone().into_keypair()?;

		pkesks[0]
			.decrypt(&mut pair, sym_algo)
			.map(|(algo, session_key)| decrypt(algo, &session_key));

		// XXX: In production code, return the Fingerprint of the
		// recipient's Cert here
		Ok(None)
	}
}
