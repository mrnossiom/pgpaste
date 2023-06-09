use crate::{args::PGPasteArgs, ToEyreError};
use dirs::config_local_dir;
use eyre::Context;
use reqwest::Url;
use sequoia_openpgp::{parse::Parse, Cert, KeyHandle};
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, path::PathBuf};

// TODO: read from a certificate store

#[derive(Debug, Serialize, Deserialize)]
struct ConfigScheme {
	#[serde(default = "default_server")]
	server: String,

	default_key: Option<String>,
	private_keys: Option<Vec<PathBuf>>,
	public_keys: Option<Vec<PathBuf>>,
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

	pub(crate) default_key: Option<KeyHandle>,
	// TODO: allow to only read public pastes without having to setup certificates
	pub(crate) private_keys: Vec<Cert>,
	pub(crate) public_keys: Vec<Cert>,
}

impl Config {
	pub(crate) fn new(args: &PGPasteArgs) -> eyre::Result<Self> {
		let config = ConfigScheme::parse(args)?;

		let default_key = config
			.default_key
			.map(|key| key.parse::<KeyHandle>().to_eyre())
			.map_or(Ok(None), |v| v.map(Some))
			.wrap_err("not a valid key handle")?;

		let private_keys = read_certs_list(config.private_keys.unwrap_or_default());
		let public_keys = read_certs_list(config.public_keys.unwrap_or_default());

		let server = Url::parse(&args.server.clone().unwrap_or(config.server))?;

		Ok(Self {
			server,
			default_key,
			private_keys,
			public_keys,
		})
	}
}

// TODO: read certs from folders
fn read_certs_list(paths: Vec<PathBuf>) -> Vec<Cert> {
	let mut errors = Vec::new();

	let certs = paths
		.into_iter()
		.map(|path| Cert::from_file(path).to_eyre())
		.filter_map(|r| r.map_err(|err| errors.push(err)).ok())
		.collect();

	for error in errors {
		log::error!("error while reading certificate: {}", error);
	}

	certs
}
