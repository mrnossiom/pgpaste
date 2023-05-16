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
};
use axum::{Router, Server};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

mod api;
mod config;
mod database;
mod error;

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
	Server::bind(&"0.0.0.0:3000".parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
