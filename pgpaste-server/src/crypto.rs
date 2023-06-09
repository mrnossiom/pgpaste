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

const POLICY: &StandardPolicy = &StandardPolicy::new();

pub(crate) fn verify(message: &[u8], helper: Helper) -> eyre::Result<()> {
	let mut decryptor = VerifierBuilder::from_bytes(&message)
		.to_eyre()?
		.with_policy(POLICY, None, helper)
		.to_eyre()?;

	io::copy(&mut decryptor, &mut io::sink())?;

	Ok(())
}

/// This helper provides secrets for the decryption, fetches public
/// keys for the signature verification and implements the
/// verification policy.
pub(crate) struct Helper<'a> {
	certs: &'a [Cert],
}

impl<'a> Helper<'a> {
	/// Creates a Helper for the given Certs with appropriate secrets.
	pub(crate) const fn new(certs: &'a [Cert]) -> Self {
		Self { certs }
	}
}

impl<'a> VerificationHelper for Helper<'a> {
	fn get_certs(&mut self, ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
		let concerned_certs = self
			.certs
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
