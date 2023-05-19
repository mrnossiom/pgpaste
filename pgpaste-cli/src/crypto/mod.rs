use sequoia_openpgp::policy::StandardPolicy;

mod receive;
mod send;

pub(crate) use receive::{decrypt, verify};
pub(crate) use send::{encrypt, protect, sign};

const POLICY: &StandardPolicy = &StandardPolicy::new();
