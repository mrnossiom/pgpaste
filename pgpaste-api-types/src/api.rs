use crate::{Paste, SystemTime, Visibility};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBody {
	pub slug: Option<String>,
	pub visibility: Visibility,
	pub burn_in: Option<Duration>,
	pub burn_after_read: bool,
	pub inner: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateResponse {
	pub slug: String,
	pub burn_at: SystemTime,
}

pub type ReadBody = ();
pub type ReadResponse = Paste;
