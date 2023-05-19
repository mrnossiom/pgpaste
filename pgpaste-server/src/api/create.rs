use crate::{
	api::extract::MsgPack,
	database::{
		models::{NewPaste, NewPublicKey, PublicKey},
		prelude::*,
		schema::pastes,
	},
	error::{ServerError, UserFacingServerError},
	AppState, ToEyreError,
};
use axum::{
	extract::State,
	http::{header, HeaderMap, HeaderValue, Method, StatusCode},
	response::IntoResponse,
};
use eyre::Context;
use pgpaste_api_types::api::{CreateBody, CreateResponse};
use sequoia_net::{KeyServer, Policy};
use sequoia_openpgp::{
	packet::Signature, parse::Parse, serialize::MarshalInto, Cert, Message, Packet,
};
use std::time::{Duration, SystemTime};

const YEAR: Duration = Duration::from_secs(60 * 60 * 24 * 365);
const WEEK: Duration = Duration::from_secs(60 * 60 * 24 * 7);

#[tracing::instrument(skip(state, content))]
pub(crate) async fn create_signed_paste(
	State(state): State<AppState>,
	headers: HeaderMap,
	method: Method,
	MsgPack(content): MsgPack<CreateBody>,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state.database.get().await?;
	let slug = content.slug.unwrap_or_else(|| petname::petname(4, "-"));
	// TODO: take into account if the user can overwrite
	let overwrite = method == Method::PUT;

	if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
		if content_type != HeaderValue::from_static(mime::APPLICATION_MSGPACK.as_ref()) {
			return Err(UserFacingServerError::InvalidContentType.into());
		}
	}

	if content.burn_in > Some(YEAR) {
		return Err(UserFacingServerError::InvalidBurnIn.into());
	}

	let cert = Message::from_bytes(&content.inner)
		.map_err(|e| tracing::error!(error = ?e))
		.map_err(|_| ServerError::UserFacing(UserFacingServerError::InvalidCert))?;

	// TODO: remove
	tracing::debug!(parsed = ?cert.descendants().collect::<Vec<_>>());

	let Some(fingerprint) = cert.descendants().find_map(|p| {
		if let Packet::Signature(Signature::V4(sig)) = p {
			sig.issuer_fingerprints().next()
		} else {
			None
		}
	}) else {
		return Err(UserFacingServerError::InvalidMessageStructure.into());
	};

	let pub_key = PublicKey::with_fingerprint(fingerprint)
		.first::<PublicKey>(&mut conn)
		.await
		.optional()
		.wrap_err("Could not get public key")?;

	// TODO: handle errors
	let cert = if let Some(key) = pub_key {
		// TODO: check rates and premium
		Cert::from_bytes(&key.cert).to_eyre()?
	} else {
		let mut key_server = KeyServer::keys_openpgp_org(Policy::Encrypted).to_eyre()?;
		let cert = key_server.get(fingerprint).await.to_eyre()?;

		let new_pub_key = NewPublicKey {
			fingerprint: fingerprint.as_bytes(),
			cert: &cert.export_to_vec().to_eyre()?,
		};
		new_pub_key.insert().execute(&mut conn).await?;

		cert
	};

	// TODO: verify signature with `cert`

	let now = SystemTime::now();
	let paste = NewPaste {
		slug: &slug,
		visibility: &content.visibility.into(),
		content: &content.inner,
		burn_at: &(now + content.burn_in.unwrap_or(WEEK)),
		created_at: &now,
		burn_after_read: content.burn_after_read,
	};

	paste
		.insert()
		.on_conflict(pastes::id)
		// TODO: check overwrite
		// .do_update().set(&paste)
		.do_nothing()
		.execute(&mut conn)
		.await
		.wrap_err("Could not insert paste")?;

	tracing::debug!(slug = slug, "Created {:?} paste", content.visibility);

	Ok((
		StatusCode::CREATED,
		MsgPack(CreateResponse {
			slug,
			burn_at: now + content.burn_in.unwrap_or(WEEK),
		}),
	))
}
