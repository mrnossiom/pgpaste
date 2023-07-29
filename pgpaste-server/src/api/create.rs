//! Routes handlers for creating pastes

use crate::{
	api::extract::MsgPack,
	crypto::{verify, SignatureHelper},
	database::{
		models::{Certificate, NewPaste, NewPublicKey, PublicKey},
		prelude::*,
		schema::{pastes, public_keys},
	},
	error::{ServerError, UserServerError},
	AppState, ToEyreError,
};
use axum::{
	body::Bytes,
	extract::State,
	http::{HeaderMap, Method, StatusCode},
	response::IntoResponse,
};
use eyre::Context;
use pgpaste_api_types::api::{CreateBody, CreateResponse};
use sequoia_net::{KeyServer, Policy};
use sequoia_openpgp::{packet::Signature, parse::Parse, serialize::MarshalInto, Message, Packet};
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
	content: Bytes,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state.database.get().await?;

	// TODO: check content type

	let signed_paste = Message::from_bytes(&content)
		.to_eyre()
		.map_err(UserServerError::InvalidCert)?;

	let fingerprint = signed_paste
		.descendants()
		.find_map(|p| match p {
			Packet::Signature(Signature::V4(sig)) => sig.issuer_fingerprints().next(),
			_ => None,
		})
		.ok_or(UserServerError::InvalidMessageStructure)?;

	let cert = if let Some(cert) = PublicKey::with_fingerprint(fingerprint)
		.select(public_keys::cert)
		.first::<Certificate>(&mut conn)
		.await
		.optional()
		.wrap_err("could not get public key profile")?
	{
		cert.into()
	} else {
		// TODO: actually seems vulnerable, since we could be rate-limited by the keyserver

		let mut key_server = KeyServer::keys_openpgp_org(Policy::Encrypted).to_eyre()?;

		key_server
			.get(fingerprint)
			.await
			.to_eyre()
			.map_err(UserServerError::CertUnknown)?
	};

	let helper = SignatureHelper::new(cert.clone());
	let bytes = verify(&content, helper).map_err(UserServerError::InvalidSignature)?;

	let paste_query = rmp_serde::from_slice::<CreateBody>(&bytes)
		.map_err(UserServerError::MsgPackBodyIsInvalid)?;

	let content = Message::from_bytes(&paste_query.message)
		.to_eyre()
		.map_err(UserServerError::InvalidCert)?;

	let now = SystemTime::now();
	let slug = paste_query.slug.unwrap_or_else(|| petname::petname(4, "-"));
	// TODO: take into account if the user can overwrite
	let overwrite = method == Method::PUT;
	let burn_at = now + paste_query.burn_in.unwrap_or(WEEK);

	if paste_query.burn_in > Some(YEAR) {
		return Err(UserServerError::InvalidBurnIn.into());
	}

	let id = if let Some(id) = PublicKey::with_fingerprint(fingerprint)
		.select(public_keys::id)
		.first::<i32>(&mut conn)
		.await
		.optional()
		.wrap_err("could not get public key profile")?
	{
		// TODO: check rates and premium

		id
	} else {
		let new_pub_key = NewPublicKey {
			fingerprint: fingerprint.as_bytes(),
			cert: (&cert).into(),
			is_premium: false,
		};

		new_pub_key
			.insert()
			.returning(public_keys::id)
			.get_result(&mut conn)
			.await
			.wrap_err("could not insert new public key")?
	};

	let paste = NewPaste {
		public_key_id: id,
		slug: &slug,
		mime: (&paste_query.mime).into(),
		visibility: &(&paste_query.visibility).into(),
		content: &content.to_vec().to_wrap_err("msg")?,
		burn_at: &burn_at,
		created_at: &now,
		burn_after_read: paste_query.burn_after_read,
	};

	// TODO: find a way to box insert queries
	// TODO: handle conflict errors
	if overwrite {
		paste
			.insert()
			.on_conflict(pastes::id)
			.do_update()
			.set(&paste)
			.execute(&mut conn)
			.await
			.wrap_err("Could not insert paste")?;
	} else {
		paste
			.insert()
			.on_conflict(pastes::id)
			.do_nothing()
			.execute(&mut conn)
			.await
			.wrap_err("Could not insert paste")?;
	}

	tracing::debug!(slug = slug, "Created {:?} paste", paste.visibility);

	Ok((
		StatusCode::CREATED,
		MsgPack(CreateResponse { slug, burn_at }),
	))
}
