#![warn(
	// clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo,
	rustdoc::broken_intra_doc_links
)]

use mime::Mime;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub mod api;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
	Private,
	Protected,
	Public,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paste {
	pub slug: String,
	#[serde(with = "mime_proxy")]
	pub mime: Mime,
	pub visibility: Visibility,
	pub burn_at: SystemTime,
	pub inner: Vec<u8>,
}

mod mime_proxy {
	use mime::Mime;
	use serde::{Deserialize, Deserializer, Serializer};

	pub fn serialize<S>(mime: &Mime, ser: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		ser.serialize_str(mime.to_string().as_str())
	}

	pub fn deserialize<'de, D>(deser: D) -> Result<Mime, D::Error>
	where
		D: Deserializer<'de>,
	{
		let plain_mime: String = Deserialize::deserialize(deser)?;

		plain_mime.parse().map_err(serde::de::Error::custom)
	}
}
