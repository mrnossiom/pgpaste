use crate::args::PGPasteArgs;
use dirs::config_local_dir;
use reqwest::Url;
use sequoia_openpgp::{parse::Parse, Cert};
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct ConfigScheme {
	server: String,
	keys: Option<PathBuf>,
}

impl Default for ConfigScheme {
	fn default() -> Self {
		Self {
			server: "https://pgpaste.org".into(),
			keys: None,
		}
	}
}

#[derive(Debug)]
pub(crate) struct Config {
	pub(crate) server: Url,

	pub(crate) keys: Option<Cert>,
}

impl Config {
	pub(crate) fn new(args: &PGPasteArgs) -> eyre::Result<Self> {
		let config = Self::parse_config_scheme(args)?;

		let keys = config
			.keys
			.map(|path| Cert::from_file(path).map_err(|err| eyre::eyre!(Box::new(err))))
			.map_or(Ok(None), |v| v.map(Some))?;

		let server = Url::parse(&args.server.clone().unwrap_or(config.server))?;

		Ok(Self { keys, server })
	}

	fn parse_config_scheme(args: &PGPasteArgs) -> eyre::Result<ConfigScheme> {
		let path = match args.config {
			Some(ref path) => path.clone(),
			None => {
				let Some(mut path) = config_local_dir() else {
					eyre::bail!("Could not find config directory");
				};

				path.push("pgpaste.toml");
				path
			}
		};

		let content = match read_to_string(path) {
			Ok(content) => content,
			Err(err) => {
				if err.kind() == std::io::ErrorKind::NotFound {
					return Ok(ConfigScheme::default());
				}

				return Err(err.into());
			}
		};

		Ok(toml::from_str::<ConfigScheme>(&content)?)
	}
}
