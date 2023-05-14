use crate::{
	api::extract::MsgPack,
	database::{models::NewPaste, prelude::*},
	error::{ServerError, UserFacingServerError},
	AppState,
};
use axum::{
	body::Bytes,
	extract::{Query, State},
	http::{header, HeaderMap, HeaderValue, StatusCode},
	response::IntoResponse,
};
use eyre::Context;
use pgpaste_api_types::{CreateQuery, CreateResponse};
use sequoia_openpgp::{parse::Parse, Message};

pub(crate) async fn create_signed_paste(
	State(state): State<AppState>,
	Query(query): Query<CreateQuery>,
	headers: HeaderMap,
	content: Bytes,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state.database.get().await?;
	let slug = query.slug.unwrap_or_else(|| petname::petname(4, "-"));

	if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
		if content_type != HeaderValue::from_static("application/pgp-encrypted") {
			return Err(UserFacingServerError::InvalidContentType.into());
		}
	}

	let cert = Message::from_bytes(&content)
		.map_err(|e| tracing::error!(error = ?e))
		.map_err(|_| ServerError::UserFacing(UserFacingServerError::InvalidCert))?;

	dbg!(cert.descendants().collect::<Vec<_>>());

	let paste = NewPaste {
		slug: &slug,
		visibility: &query.visibility.into(),
		content: &content,
	};

	paste
		.insert()
		.execute(&mut conn)
		.await
		.wrap_err("Could not insert paste")?;

	tracing::debug!(slug = slug, "Created {:?} paste", query.visibility);

	Ok((StatusCode::CREATED, MsgPack(CreateResponse { slug })))
}
