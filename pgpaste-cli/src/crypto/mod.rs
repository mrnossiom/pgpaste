//! Cryptographic functions to sign/verify or encrypt/decrypt a message.

use sequoia_openpgp::policy::StandardPolicy;

mod receive;
mod send;

pub(crate) use receive::{ReceiveHelper, decrypt, verify};
pub(crate) use send::{SendHelper, encrypt, protect, sign};

/// Default policy used for certificate verification
const POLICY: &StandardPolicy = &StandardPolicy::new();
