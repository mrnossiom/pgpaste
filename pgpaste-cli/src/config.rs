use crate::{args::PGPasteArgs, ToEyreError};
use dirs::config_local_dir;
use reqwest::Url;
use sequoia_openpgp::{parse::Parse, Cert};
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct ConfigScheme {
	#[serde(default = "default_server")]
	server: String,

	keys: Option<PathBuf>,
}

fn default_server() -> String {
	"https://pgpaste.org".into()
}

impl ConfigScheme {
	fn parse(args: &PGPasteArgs) -> eyre::Result<Self> {
		let path = if let Some(ref path) = args.config {
			path.clone()
		} else {
			let Some(mut path) = config_local_dir() else {
				eyre::bail!("Could not find config directory");
			};

			path.push("pgpaste.toml");
			path
		};

		let config = match read_to_string(path) {
			Ok(content) => toml::from_str(&content)?,
			Err(err) => {
				if err.kind() != std::io::ErrorKind::NotFound {
					return Err(err.into());
				}

				toml::from_str("")?
			}
		};

		Ok(config)
	}
}

#[derive(Debug)]
pub(crate) struct Config {
	pub(crate) server: Url,

	pub(crate) keys: Option<Cert>,
}

impl Config {
	pub(crate) fn new(args: &PGPasteArgs) -> eyre::Result<Self> {
		let config = ConfigScheme::parse(args)?;

		let keys = config
			.keys
			.map(|path| Cert::from_file(path).to_eyre())
			.map_or(Ok(None), |v| v.map(Some))?;

		let server = Url::parse(&args.server.clone().unwrap_or(config.server))?;

		Ok(Self { server, keys })
	}
}
