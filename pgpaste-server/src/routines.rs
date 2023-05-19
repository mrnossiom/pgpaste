use crate::{
	config::AppState,
	database::{models::Paste, prelude::*},
};
use tokio::{task::JoinHandle, time::Duration};

pub(crate) async fn setup_routines(state: AppState) -> eyre::Result<()> {
	// Setup cleanup burnt pastes routine
	set_interval(
		state.clone(),
		Duration::from_secs(60 * 60),
		delete_burnt_pastes,
	)
	.await;

	Ok(())
}

#[tracing::instrument(skip(state))]
async fn delete_burnt_pastes(state: AppState) -> eyre::Result<()> {
	let mut conn = state.database.get().await?;

	db_dsl::delete(Paste::all_burnt())
		.execute(&mut conn)
		.await?;

	Ok(())
}

pub(crate) async fn set_interval<T, F>(
	state: AppState,
	interval: std::time::Duration,
	future: T,
) -> JoinHandle<()>
where
	T: (Fn(AppState) -> F) + Send + Sync + 'static,
	F: std::future::Future<Output = eyre::Result<()>> + Send,
{
	// The interval time alignment is decided at construction time.
	// For all calls to be evenly spaced, the interval must be constructed first.
	let mut interval = tokio::time::interval(interval);
	// The first tick happens without delay.
	interval.tick().await;

	tokio::task::spawn(async move {
		loop {
			if let Err(err) = future(state.clone()).await {
				tracing::error!(error = ?err);
			};

			interval.tick().await;
		}
	})
}
