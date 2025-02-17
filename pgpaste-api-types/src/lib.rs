//! `pgpaste` API types

// TODO: think about switching to `msgpacker`

use std::time::SystemTime;

use mime::Mime;
use serde::{Deserialize, Serialize};

pub mod api;

/// The visibility of a paste
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
	/// The paste is private and only one recipient can read it
	Private,
	/// The paste is protected by a password
	Protected,
	/// The paste is signed and can be read by anyone
	Public,
}

/// A paste
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paste {
	/// Paste slug
	pub slug: String,
	/// Content mime type
	#[serde(with = "mime_proxy")]
	pub mime: Mime,
	/// The paste visibility
	pub visibility: Visibility,
	/// The time at which the paste will be deleted
	pub burn_at: SystemTime,
	/// The inner OpenPGP message
	pub inner: Vec<u8>,
}

/// Proxy ser/deserialization module for `mime` crate
mod mime_proxy {
	use mime::Mime;
	use serde::{Deserialize, Deserializer, Serializer};

	/// Serialize a [`Mime`] to a string
	pub fn serialize<S>(mime: &Mime, ser: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		ser.serialize_str(mime.to_string().as_str())
	}

	/// Deserialize a [`Mime`] from a string
	pub fn deserialize<'de, D>(de: D) -> Result<Mime, D::Error>
	where
		D: Deserializer<'de>,
	{
		let plain_mime: String = Deserialize::deserialize(de)?;

		plain_mime.parse().map_err(serde::de::Error::custom)
	}
}
