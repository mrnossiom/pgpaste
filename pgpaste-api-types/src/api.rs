//! API types for each path

use crate::{Paste, SystemTime, Visibility};
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// `MsgPack` Body payload for POST `/api/paste` endpoint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBody {
	/// New paste slug
	pub slug: Option<String>,
	#[serde(with = "crate::mime_proxy")]
	/// Content mime type
	pub mime: Mime,
	/// The paste visibility
	pub visibility: Visibility,
	/// The paste lifetime
	pub burn_in: Option<Duration>,
	/// Whether the paste should be deleted after reading
	pub burn_after_read: bool,
	/// The inner OpenPGP message
	pub inner: Vec<u8>,
}

/// `MsgPack` Body response for POST `/api/paste` endpoint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateResponse {
	/// New paste slug
	pub slug: String,
	/// The time at which the paste will be deleted
	pub burn_at: SystemTime,
}

/// `MsgPack` Body payload for GET `/api/paste` endpoint
pub type ReadBody = ();
/// `MsgPack` Body response for GET `/api/paste` endpoint
pub type ReadResponse = Paste;
