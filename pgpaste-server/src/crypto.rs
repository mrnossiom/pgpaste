//! Certs verification module

use crate::ToEyreError;
use sequoia_openpgp::{
	parse::{
		stream::{MessageLayer, MessageStructure, VerificationHelper, VerifierBuilder},
		Parse,
	},
	policy::StandardPolicy,
	Cert, KeyHandle,
};
use std::io;

/// Default policy used for certificate verification
const POLICY: &StandardPolicy = &StandardPolicy::new();

/// Parses and verify the given `OpenPGP` message
pub(crate) fn verify(message: &[u8], helper: SignatureHelper) -> eyre::Result<Vec<u8>> {
	let mut decryptor = VerifierBuilder::from_bytes(&message)
		.to_eyre()?
		.with_policy(POLICY, None, helper)
		.to_eyre()?;

	let mut bytes = Vec::new();
	io::copy(&mut decryptor, &mut bytes)?;

	Ok(bytes)
}

/// This helper provides secrets for the decryption, fetches public
/// keys for the signature verification and implements the
/// verification policy.
pub(crate) struct SignatureHelper {
	/// The certificate used for verification.
	cert: Cert,
}

impl SignatureHelper {
	/// Creates a [`SignatureHelper`] for the given certificate.
	pub(crate) const fn new(cert: Cert) -> Self {
		Self { cert }
	}
}

impl VerificationHelper for SignatureHelper {
	fn get_certs(&mut self, ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
		if ids
			.iter()
			.any(|handle| handle == &self.cert.fingerprint().into())
		{
			Ok(vec![self.cert.clone()])
		} else {
			Ok(vec![])
		}
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
