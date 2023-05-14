use crate::api::api_router;
use axum::{Router, Server};
use diesel_async::{
	pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
	AsyncPgConnection,
};
use dotenvy::dotenv;
use eyre::Context;
use secrecy::{ExposeSecret, Secret};
use std::{
	env::{self, VarError},
	fmt,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

mod api;
mod database;
mod error;

/// App global configuration
#[derive(Debug, Clone)]
pub(crate) struct Config {
	/// The `Postgres` connection uri
	pub(crate) database_url: Secret<String>,

	/// Whether or not to use production defaults
	///
	/// Currently affects nothing
	pub(crate) _production: bool,
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
	fn from_dotenv() -> eyre::Result<Self> {
		// Load the `.env` file ond error if not found
		dotenv()?;

		let production = env::var("PRODUCTION")
			.unwrap_or_else(|_| "false".into())
			.parse::<bool>()
			.map_err(|_| eyre::eyre!("PRODUCTION environnement variable must be a `bool`"))?;

		Ok(Self {
			database_url: Secret::new(required_env_var("DATABASE_URL")?),
			_production: production,
		})
	}
}

#[derive(Clone)]
struct AppState {
	pub(crate) config: Config,
	database: Pool<AsyncPgConnection>,
}

impl fmt::Debug for AppState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Data").finish_non_exhaustive()
	}
}

impl AppState {
	fn new(config: Config) -> eyre::Result<Self> {
		let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
			config.database_url.expose_secret(),
		);
		let database = Pool::builder(manager)
			.build()
			.context("failed to create database pool")?;

		Ok(Self { database, config })
	}
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	Registry::default()
		.with(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "info,pgpaste_server=debug".into()),
		)
		.with(tracing_subscriber::fmt::layer())
		.init();

	let config = Config::from_dotenv()?;
	let state = AppState::new(config)?;

	database::run_migrations(&state.config)?;

	let app = Router::new()
		.nest("/api", api_router())
		.layer(TraceLayer::new_for_http())
		.with_state(state);

	tracing::debug!("Starting server");
	Server::bind(&"0.0.0.0:3000".parse().unwrap())
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
