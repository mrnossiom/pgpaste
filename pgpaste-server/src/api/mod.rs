use crate::{
	database::{models::Paste, prelude::*, schema::pastes},
	error::ServerError,
	AppState,
};
use axum::{
	extract::{DefaultBodyLimit, Path, State},
	http::StatusCode,
	response::IntoResponse,
	routing::{get, Router},
};
use eyre::Context;

mod create;

pub(crate) fn api_router() -> Router<AppState> {
	Router::new()
		.route(
			"/paste/:slug",
			get(get_paste)
				.post(create::create_signed_paste)
				.delete(|| async { StatusCode::NOT_IMPLEMENTED }),
		)
		.route("/key/:fingerprint/list", get(get_key_pastes))
		// Set the limit to the default 2 MiB
		.layer(DefaultBodyLimit::max(2 * 1024))
}

async fn get_paste(
	State(state): State<AppState>,
	Path(paste_slug): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state
		.database
		.get()
		.await
		.wrap_err("Could not get a db handle")?;

	let res = Paste::with_slug(&paste_slug)
		.select(pastes::content)
		.first::<Vec<u8>>(&mut conn)
		.await
		.wrap_err("")?;

	Ok((StatusCode::OK, res))
}

async fn get_key_pastes() -> impl IntoResponse {
	StatusCode::NOT_IMPLEMENTED
}
