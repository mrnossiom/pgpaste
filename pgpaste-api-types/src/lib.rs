#![warn(
	// clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo,
	rustdoc::broken_intra_doc_links
)]

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
	pub visibility: Visibility,
	pub burn_at: SystemTime,
	pub inner: Vec<u8>,
}
