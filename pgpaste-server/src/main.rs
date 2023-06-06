#![warn(
	// clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo,
	rustdoc::broken_intra_doc_links
)]
#![allow(clippy::redundant_pub_crate)]

use crate::{
	api::api_router,
	config::{AppState, Config},
	routines::setup_routines,
};
use axum::{Router, Server};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

mod api;
mod config;
mod crypto;
mod database;
mod error;
mod routines;

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
		.with_state(state.clone());

	setup_routines(state).await?;

	tracing::debug!("Starting server");
	Server::bind(&"0.0.0.0:3000".parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}

pub(crate) trait ToEyreError<T> {
	fn to_eyre(self) -> eyre::Result<T>;
}

impl<T> ToEyreError<T> for sequoia_openpgp::Result<T> {
	fn to_eyre(self) -> eyre::Result<T> {
		self.map_err(|err| eyre::eyre!(Box::new(err)))
	}
}
