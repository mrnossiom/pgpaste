//! CLI parsed configuration

use std::{fs::read_to_string, path::PathBuf};

use dirs::config_local_dir;
use eyre::Context;
use reqwest::Url;
use sequoia_openpgp::{Cert, KeyHandle, parse::Parse};
use serde::{Deserialize, Serialize};

use crate::{ToEyreError, args::PGPasteArgs};

/// Config scheme as represented on disk
#[derive(Debug, Serialize, Deserialize)]
struct ConfigScheme {
	/// The pgpaste server to query
	#[serde(default = "default_server")]
	server: String,

	/// The default key to use when encrypting or signing pastes
	default_key: Option<String>,
	/// A set of private keys to use when encrypting or decrypting pastes
	private_keys: Option<Vec<PathBuf>>,
	/// A set of public keys to use when signing or verifying pastes
	public_keys: Option<Vec<PathBuf>>,
}

/// The default public instance of a pgpaste server
fn default_server() -> String {
	"https://pgpaste.org".into()
}

impl ConfigScheme {
	/// Parse the config file for further processing
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

/// The real config parsed and used in code.
#[derive(Debug)]
pub(crate) struct Config {
	/// The pgpaste server to query
	pub(crate) server: Url,

	/// The default key to use when encrypting or signing pastes
	pub(crate) default_key: Option<KeyHandle>,
	/// A set of private keys to use when encrypting or decrypting pastes
	pub(crate) private_keys: Vec<Cert>,
	/// A set of public keys to use when signing or verifying pastes
	pub(crate) public_keys: Vec<Cert>,
}

impl Config {
	/// Initialize the config
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
/// Read a list of certificates from a list of paths
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
