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

use super::extract::MsgPack;

pub(crate) async fn get_paste(
	State(state): State<AppState>,
	Path(paste_slug): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state.database.get().await?;

	let paste = Paste::with_slug(&paste_slug)
		.select(Paste::as_select())
		.first::<Paste>(&mut conn)
		.await
		.optional()
		.wrap_err("Failed to load paste")?;

	let paste = if let Some(paste) = paste {
		// if paste.burn_at < SystemTime::now() {
		// 	return Err(ServerError::UserFacing(UserFacingServerError::PasteBurned));
		// }

		paste
	} else {
		return Err(UserFacingServerError::PasteNotFound.into());
	};

	let res = ReadResponse {
		slug: paste.slug,
		visibility: paste.visibility.into(),
		inner: paste.content,
		burn_at: SystemTime::now(),
	};

	Ok((StatusCode::OK, MsgPack(res)))
}

pub(crate) async fn get_key_pastes() -> impl IntoResponse {
	StatusCode::NOT_IMPLEMENTED
}
