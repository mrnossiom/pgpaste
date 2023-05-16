use serde::{Deserialize, Serialize};
use std::time::SystemTime;

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

pub mod api {
	use super::*;
	use serde::{Deserialize, Serialize};
	use std::time::Duration;

	#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
	#[serde(deny_unknown_fields)]
	pub struct CreateBody {
		pub slug: Option<String>,
		pub visibility: Visibility,
		pub burn_in: Option<Duration>,
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::error::Error;

	#[test]
	fn to_msgpack_output() -> Result<(), Box<dyn Error>> {
		let message = Paste {
			visibility: Visibility::Private,
			slug: "test".into(),
			burn_at: SystemTime::UNIX_EPOCH,
			inner: vec![],
		};

		let _serialized = dbg!(rmp_serde::to_vec(&message))?;

		Ok(())
	}
}
