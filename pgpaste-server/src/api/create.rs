//! Routes handlers for creating pastes

use crate::{
	api::extract::MsgPack,
	crypto::{verify, Helper},
	database::{
		models::{NewPaste, NewPublicKey, PublicKey},
		prelude::*,
		schema::{pastes, public_keys},
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

/// A year
const YEAR: Duration = Duration::from_secs(60 * 60 * 24 * 365);
/// A week
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

	let message = Message::from_bytes(&content.inner)
		.to_eyre()
		.map_err(UserFacingServerError::InvalidCert)?;

	let Some(fingerprint) = message.descendants().find_map(|p| {
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
		.wrap_err("could not get public key profile")?;

	let (cert, id) = if let Some(key) = pub_key {
		// TODO: check rates and premium
		let cert = Cert::from_bytes(&key.cert).to_eyre()?;

		(cert, key.id)
	} else {
		let mut key_server = KeyServer::keys_openpgp_org(Policy::Encrypted).to_eyre()?;
		let cert = key_server
			.get(fingerprint)
			.await
			.to_eyre()
			.wrap_err("cert unknown in public store")?;

		let new_pub_key = NewPublicKey {
			fingerprint: fingerprint.as_bytes(),
			cert: &cert.export_to_vec().to_eyre()?,
		};

		let id = new_pub_key
			.insert()
			.returning(public_keys::id)
			.get_result(&mut conn)
			.await?;

		(cert, id)
	};

	let cert_vec = vec![cert];
	let helper = Helper::new(&cert_vec);
	if let Err(e) = verify(&content.inner, helper) {
		return Err(UserFacingServerError::InvalidSignature(e).into());
	};

	let now = SystemTime::now();
	let paste = NewPaste {
		public_key_id: id,
		slug: &slug,
		mime: content.mime.clone().into(),
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
