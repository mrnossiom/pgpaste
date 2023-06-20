//! Paths normal users could interact with in contrast to API paths

use crate::{
	config::AppState,
	database::{
		models::{Paste, Visibility},
		prelude::*,
	},
	error::{ServerError, UserServerError},
	ToEyreError,
};
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
	routing::get,
	Router,
};
use eyre::{ContextCompat, WrapErr};
use sequoia_openpgp::{parse::Parse, Message};

/// The API routes definition
pub(crate) fn pastes_router() -> Router<AppState> {
	Router::new().route("/:paste_slug", get(get_public_paste))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn get_public_paste<'a>(
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
		return Err(UserServerError::PasteNotFound.into());
	};

	// TODO: see if we want to check the content again

	let message = Message::from_bytes(&paste.content).to_wrap_err("Failed to parse paste")?;

	let message = match paste.visibility {
		Visibility::Public => message
			.body()
			.wrap_err("internal state error, paste tagged as public but has no literal body")?
			.body(),
		Visibility::Protected => return Err(UserServerError::PasteIsProtected.into()),
		Visibility::Private => {
			return Err(UserServerError::PasteIsPrivate.into());
		}
	};

	Ok((
		StatusCode::OK,
		[("Content-Type", "text/plain")],
		message.to_owned(),
	))
}
