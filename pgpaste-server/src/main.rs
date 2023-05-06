use api::api_router;
use axum::{Router, Server};
use database::run_migrations;
use diesel_async::{
	pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
	AsyncPgConnection,
};
use eyre::Context;
use std::fmt;
use tower_http::trace::TraceLayer;

mod api;
mod database;
mod error;

#[derive(Clone)]
struct AppState {
	database: Pool<AsyncPgConnection>,
}

impl fmt::Debug for AppState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Data").finish_non_exhaustive()
	}
}

impl AppState {
	fn new() -> eyre::Result<Self> {
		let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
			"postgres://server:server@172.19.0.2/server",
		);
		let database = Pool::builder(manager)
			.build()
			.context("failed to create database pool")?;

		Ok(Self { database })
	}
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	tracing_subscriber::fmt::init();

	run_migrations("postgres://server:server@172.19.0.2/server")?;

	let app = Router::new()
		.nest("/api", api_router())
		.layer(TraceLayer::new_for_http())
		.with_state(AppState::new()?);

	Server::bind(&"0.0.0.0:3000".parse().unwrap())
		.serve(app.into_make_service())
		.await
		.unwrap();

	Ok(())
}
