//! `pgpaste` server

use std::fmt::Display;

use axum::Router;
use eyre::Context;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
	api::api_router,
	config::{AppState, Config},
	routes::pastes_router,
	routines::setup_routines,
};

mod api;
mod config;
mod crypto;
mod database;
mod error;
mod routes;
mod routines;

#[tokio::main]
async fn main() -> eyre::Result<()> {
	Registry::default()
		.with(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "info,pgpaste_server=debug".into()),
		)
		.with(
			tracing_subscriber::fmt::layer()
				.with_file(true)
				.with_line_number(true),
		)
		.init();

	let config = Config::from_dotenv()?;
	let state = AppState::new(config)?;

	database::run_migrations(&state.config)?;

	let app = Router::new()
		.nest("/api", api_router())
		.nest("/p", pastes_router())
		// .route("/:*", web())
		.layer(TraceLayer::new_for_http())
		.with_state(state.clone());

	setup_routines(state).await?;

	tracing::debug!("Starting server");
	let listener = TcpListener::bind("0.0.0.0:3000")
		.await
		.wrap_err("could not bind to the specified interface")?;

	axum::serve(listener, app)
		.await
		.wrap_err("there was an error while serving the router")?;

	Ok(())
}

/// Compat trait to interop between eyre and sequoia v1.x (anyhow) errors
pub(crate) trait ToEyreError<T> {
	/// Convert to eyre error
	fn to_eyre(self) -> eyre::Result<T>;

	/// Convert to eyre error and wrap with a message
	#[inline]
	fn to_wrap_err<D>(self, msg: D) -> eyre::Result<T>
	where
		Self: Sized,
		D: Display + Send + Sync + 'static,
	{
		self.to_eyre().wrap_err(msg)
	}
}

impl<T> ToEyreError<T> for sequoia_openpgp::Result<T> {
	fn to_eyre(self) -> eyre::Result<T> {
		self.map_err(|err| eyre::eyre!(Box::new(err)))
	}
}
