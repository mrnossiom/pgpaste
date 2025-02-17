//! State and configuration

use std::{
	env::{self, VarError},
	fmt,
};

use diesel_async::{
	AsyncPgConnection,
	pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use dotenvy::dotenv;
use eyre::Context;
use secrecy::{ExposeSecret, SecretString};

/// App global configuration
#[derive(Debug, Clone)]
pub(crate) struct Config {
	/// The `Postgres` connection uri
	pub(crate) database_url: SecretString,
}

/// Resolve an environment variable or return an appropriate error
fn required_env_var(name: &str) -> eyre::Result<String> {
	match env::var(name) {
		Ok(val) => Ok(val),
		Err(VarError::NotPresent) => Err(eyre::eyre!("{} must be set in the environnement", name)),
		Err(VarError::NotUnicode(_)) => {
			Err(eyre::eyre!("{} does not contains Unicode valid text", name))
		}
	}
}

impl Config {
	/// Parse the config from `.env` file
	pub(crate) fn from_dotenv() -> eyre::Result<Self> {
		// Load the `.env` file ond error if not found
		dotenv()?;

		Ok(Self {
			database_url: SecretString::from(required_env_var("DATABASE_URL")?),
		})
	}
}

/// App global state
#[derive(Clone)]
pub(crate) struct AppState {
	/// Current config
	pub(crate) config: Config,
	/// Database connection pool
	pub(crate) database: Pool<AsyncPgConnection>,
}

impl fmt::Debug for AppState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Data").finish_non_exhaustive()
	}
}

impl AppState {
	/// Initialize the app state
	pub(crate) fn new(config: Config) -> eyre::Result<Self> {
		let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
			config.database_url.expose_secret(),
		);
		let database = Pool::builder(manager)
			.build()
			.context("failed to create database pool")?;

		Ok(Self { config, database })
	}
}
