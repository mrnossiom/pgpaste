//! Routes handlers for reading pastes

use super::extract::MsgPack;
use crate::{
	database::{models::Paste, prelude::*},
	error::{ServerError, UserFacingServerError},
	AppState,
};
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
};
use eyre::Context;
use pgpaste_api_types::api::ReadResponse;
use std::time::SystemTime;

#[tracing::instrument(skip(state))]
pub(crate) async fn get_paste(
	State(state): State<AppState>,
	Path(paste_slug): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state.database.get().await?;

	let Some(paste) = Paste::with_slug(&paste_slug)
		.select(Paste::as_select())
		.first::<Paste>(&mut conn)
		.await
		.optional()
		.wrap_err("Failed to load paste")? else
	{
		return Err(UserFacingServerError::PasteNotFound.into());
	};

	let res = ReadResponse {
		slug: paste.slug,
		mime: paste.mime.into(),
		visibility: paste.visibility.into(),
		inner: paste.content,
		burn_at: SystemTime::now(),
	};

	Ok((StatusCode::OK, MsgPack(res)))
}

#[allow(clippy::unused_async)]
#[tracing::instrument]
pub(crate) async fn get_key_pastes() -> Result<impl IntoResponse, ServerError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}
