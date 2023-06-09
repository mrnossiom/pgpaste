use sequoia_openpgp::policy::StandardPolicy;

mod receive;
mod send;

pub(crate) use receive::{decrypt, verify, ReceiveHelper};
pub(crate) use send::{encrypt, protect, sign, SendHelper};

const POLICY: &StandardPolicy = &StandardPolicy::new();
