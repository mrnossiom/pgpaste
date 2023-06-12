//! Cryptographic functions to sign/verify or encrypt/decrypt a message.

use sequoia_openpgp::policy::StandardPolicy;

mod receive;
mod send;

pub(crate) use receive::{decrypt, verify, ReceiveHelper};
pub(crate) use send::{encrypt, protect, sign, SendHelper};

/// Default policy used for certificate verification
const POLICY: &StandardPolicy = &StandardPolicy::new();
