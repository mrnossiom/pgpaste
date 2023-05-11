use crate::{
	database::{models::NewPaste, prelude::*, schema::pastes},
	error::{ServerError, UserFacingServerError},
	AppState,
};
use axum::{
	body::Bytes,
	extract::{Path, Query, State},
	http::{header, HeaderMap, HeaderValue, StatusCode},
	response::IntoResponse,
};
use eyre::Context;
use pgpaste_api_types::CreateQuery;
use sequoia_openpgp::{parse::Parse, Message};

pub(crate) async fn create_signed_paste(
	State(state): State<AppState>,
	Path(slug): Path<String>,
	Query(query): Query<CreateQuery>,
	headers: HeaderMap,
	content: Bytes,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state.database.get().await?;

	if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
		if content_type != HeaderValue::from_static("application/pgp-encrypted") {
			return Err(UserFacingServerError::InvalidContentType.into());
		}
	}

	let cert = Message::from_bytes(&content)
		.map_err(|e| tracing::error!(error = ?e))
		.map_err(|_| ServerError::UserFacing(UserFacingServerError::InvalidCert))?;

	dbg!(&cert);

	let paste = NewPaste {
		public_key_id: 1,

		slug: &slug,
		visibility: &query.visibility.into(),
		content: &content,
	};

	let id = paste
		.insert()
		.returning(pastes::id)
		.get_result::<i32>(&mut conn)
		.await
		.wrap_err("Could not insert paste")?;

	tracing::debug!(id = ?id, "Created paste");

	Ok((StatusCode::CREATED, id.to_string()))
}
