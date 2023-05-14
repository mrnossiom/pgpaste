use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
	Private,
	Protected,
	Public,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateQuery {
	pub slug: Option<String>,
	pub visibility: Visibility,
	pub overwrite: Option<bool>,
	// pub cleanup_after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateResponse {
	pub slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Message {
	visibility: Visibility,
	inner: Vec<u8>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use rmp_serde::to_vec as to_msgpack;
	use std::error::Error;

	#[test]
	fn one() -> Result<(), Box<dyn Error>> {
		let query = super::CreateQuery {
			slug: Some("hello-world".into()),
			visibility: Visibility::Private,
			overwrite: Some(false),
		};

		let _serialized = dbg!(serde_urlencoded::to_string(query)?);

		Ok(())
	}

	#[test]
	fn two() -> Result<(), Box<dyn Error>> {
		let query = "visibility=private";

		let _parsed: super::CreateQuery = dbg!(serde_urlencoded::from_str(query)?);

		Ok(())
	}

	#[test]
	fn dlhn() -> Result<(), Box<dyn Error>> {
		let message = Message {
			visibility: Visibility::Private,
			inner: vec![],
		};

		let _serialized = dbg!(to_msgpack(&message))?;

		Ok(())
	}
}
