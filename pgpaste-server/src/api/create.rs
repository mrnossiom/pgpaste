use std::time::SystemTime;

use crate::{
	api::extract::MsgPack,
	database::{models::NewPaste, prelude::*},
	error::{ServerError, UserFacingServerError},
	AppState,
};
use axum::{
	extract::State,
	http::{header, HeaderMap, HeaderValue, Method, StatusCode},
	response::IntoResponse,
};
use eyre::Context;
use pgpaste_api_types::api::{CreateBody, CreateResponse};
use sequoia_openpgp::{parse::Parse, Message};

pub(crate) async fn create_signed_paste(
	State(state): State<AppState>,
	headers: HeaderMap,
	method: Method,
	MsgPack(content): MsgPack<CreateBody>,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state.database.get().await?;
	let slug = content.slug.unwrap_or_else(|| petname::petname(4, "-"));
	let overwrite = method == Method::PUT;

	if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
		if content_type != HeaderValue::from_static(mime::APPLICATION_MSGPACK.as_ref()) {
			return Err(UserFacingServerError::InvalidContentType.into());
		}
	}

	let cert = Message::from_bytes(&content.inner)
		.map_err(|e| tracing::error!(error = ?e))
		.map_err(|_| ServerError::UserFacing(UserFacingServerError::InvalidCert))?;

	dbg!(cert.descendants().collect::<Vec<_>>());

	if overwrite {
		todo!()
	} else {
		let paste = NewPaste {
			slug: &slug,
			visibility: &content.visibility.into(),
			content: &content.inner,
		};

		paste
			.insert()
			.execute(&mut conn)
			.await
			.wrap_err("Could not insert paste")?;
	}

	tracing::debug!(slug = slug, "Created {:?} paste", content.visibility);

	Ok((
		StatusCode::CREATED,
		MsgPack(CreateResponse {
			slug,
			// TODO
			burn_at: SystemTime::now(),
		}),
	))
}
