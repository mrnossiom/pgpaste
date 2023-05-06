use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
	Private,
	Protected,
	Public,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateQuery {
	pub visibility: Visibility,
	pub overwrite: Option<bool>,
}

#[cfg(test)]
mod tests {
	use serde_urlencoded::{from_str, to_string};
	use std::error::Error;

	#[test]
	fn one() -> Result<(), Box<dyn Error>> {
		let query = super::CreateQuery {
			visibility: super::Visibility::Private,
			overwrite: Some(false),
		};

		let _string = dbg!(to_string(query)?);

		Ok(())
	}

	#[test]
	fn two() -> Result<(), Box<dyn Error>> {
		let query = "visibility=private";

		let _parsed: super::CreateQuery = dbg!(from_str(query)?);

		Ok(())
	}
}
