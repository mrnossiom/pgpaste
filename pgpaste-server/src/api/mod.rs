//! API routes and handlers

use crate::AppState;
use axum::{
	extract::DefaultBodyLimit,
	http::StatusCode,
	routing::{get, post, Router},
};

mod create;
mod read;

/// The API routes definition
pub(crate) fn api_router() -> Router<AppState> {
	Router::new()
		.route(
			"/paste",
			post(create::create_signed_paste).put(create::create_signed_paste),
		)
		.route(
			"/paste/:slug",
			get(read::get_paste).delete(|| async { StatusCode::NOT_IMPLEMENTED }),
		)
		.route("/key/:fingerprint/list", get(read::get_key_pastes))
		// Set the limit to the default 2 MiB
		.layer(DefaultBodyLimit::max(2 * 1024))
}

/// Custom axum extractors
mod extract {
	use axum::{
		async_trait,
		body::{Bytes, HttpBody},
		extract::{rejection::BytesRejection, FromRequest},
		http::{header, HeaderMap, HeaderValue, Request, StatusCode},
		response::{IntoResponse, Response},
		BoxError,
	};
	use bytes::Buf;
	use rmp_serde::{to_vec, Deserializer};
	use sequoia_openpgp::{parse::Parse, Message};
	use serde::{de::DeserializeOwned, Serialize};

	/// Axum extractor for `MsgPack` blobs
	pub struct MsgPack<T>(pub T);

	#[async_trait]
	impl<T, S, B> FromRequest<S, B> for MsgPack<T>
	where
		T: DeserializeOwned,
		B: HttpBody + Send + 'static,
		B::Data: Send,
		B::Error: Into<BoxError>,
		S: Send + Sync,
	{
		type Rejection = MsgPackRejection;

		async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
			if !has_msgpack_content_type(req.headers()) {
				return Err(MsgPackRejection::MissingMsgPackContentType);
			}

			let mut bytes = Bytes::from_request(req, state).await?.reader();
			let mut deserializer = Deserializer::new(&mut bytes);

			let value = T::deserialize(&mut deserializer)?;

			Ok(Self(value))
		}
	}

	/// Whether the request has a msgpack content type
	fn has_msgpack_content_type(headers: &HeaderMap) -> bool {
		let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
			return false;
		};
		let Ok(content_type) = content_type.to_str() else {
			return false;
		};
		let Ok(mime) = content_type.parse::<mime::Mime>() else {
			return false;
		};

		mime.type_() == "application"
			&& (mime.subtype() == "msgpack"
				|| mime.suffix().map_or(false, |name| name == "msgpack"))
	}

	impl<T> From<T> for MsgPack<T>
	where
		T: Serialize,
	{
		fn from(inner: T) -> Self {
			Self(inner)
		}
	}

	impl<T> IntoResponse for MsgPack<T>
	where
		T: Serialize,
	{
		fn into_response(self) -> Response {
			// // Use a small initial capacity of 128 bytes like serde_json::to_vec
			// // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
			// let mut buf = BytesMut::with_capacity(128).writer();
			// let mut ser = Serializer::new(&mut buf).with_struct_map();

			// match self.0.serialize(&mut ser) {

			match to_vec(&self.0) {
				Ok(content) => (
					[(
						header::CONTENT_TYPE,
						HeaderValue::from_static(mime::APPLICATION_MSGPACK.as_ref()),
					)],
					content,
				)
					.into_response(),
				Err(err) => (
					StatusCode::INTERNAL_SERVER_ERROR,
					[(
						header::CONTENT_TYPE,
						HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
					)],
					err.to_string(),
				)
					.into_response(),
			}
		}
	}

	/// Rejection used for [`MsgPack`].
	///
	/// Contains one variant for each way the [`MsgPack`] extractor
	/// can fail.
	#[non_exhaustive]
	#[derive(Debug, thiserror::Error)]
	pub enum MsgPackRejection {
		/// The request body could not be read
		#[error(transparent)]
		Bytes(#[from] BytesRejection),
		/// The request body could not be deserialized
		#[error(transparent)]
		Decode(#[from] rmp_serde::decode::Error),
		/// The request is missing a msgpack content type
		#[error("missing application/msgpack content type")]
		MissingMsgPackContentType,
	}

	impl IntoResponse for MsgPackRejection {
		fn into_response(self) -> Response {
			match self {
				Self::Bytes(err) => err.into_response(),
				Self::Decode(err) => (StatusCode::BAD_REQUEST, format!("{err}")).into_response(),
				Self::MissingMsgPackContentType => {
					(StatusCode::BAD_REQUEST, format!("{self}")).into_response()
				}
			}
		}
	}

	// ----------------------------------------------------------------------

	/// Axum extractor for `OpenPGP` messages
	pub struct PgpMessage(pub Message);

	#[async_trait]
	impl<S, B> FromRequest<S, B> for PgpMessage
	where
		B: HttpBody + Send + 'static,
		B::Data: Send,
		B::Error: Into<BoxError>,
		S: Send + Sync,
	{
		type Rejection = PgpMessageRejection;

		async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
			if !has_pgp_signature_content_type(req.headers()) {
				return Err(PgpMessageRejection::MissingContentType);
			}

			let mut bytes = Bytes::from_request(req, state).await?.reader();
			let message = Message::from_reader(&mut bytes)?;

			Ok(Self(message))
		}
	}

	/// Whether the request has a msgpack content type
	fn has_pgp_signature_content_type(headers: &HeaderMap) -> bool {
		let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
			return false;
		};
		let Ok(content_type) = content_type.to_str() else {
			return false;
		};
		let Ok(mime) = content_type.parse::<mime::Mime>() else {
			return false;
		};

		mime.type_() == "application" && mime.subtype() == "pgp-signature"
	}

	/// Rejection used for [`PgpMessage`].
	///
	/// Contains one variant for each way the [`MsgPack`] extractor
	/// can fail.
	#[non_exhaustive]
	#[derive(Debug, thiserror::Error)]
	pub enum PgpMessageRejection {
		/// The request body could not be read
		#[error(transparent)]
		Bytes(#[from] BytesRejection),
		/// The request body could not be deserialized
		#[error("failed to decode the `OpenPGP` message")]
		Sequoia(#[from] anyhow::Error),
		/// The request is missing the `application/pgp-signature` content type
		#[error("missing `application/pgp-signature` content type")]
		MissingContentType,
	}

	impl IntoResponse for PgpMessageRejection {
		fn into_response(self) -> Response {
			tracing::error!(error = ?self);

			match self {
				Self::Bytes(err) => err.into_response(),
				Self::Sequoia(_) | Self::MissingContentType => {
					(StatusCode::BAD_REQUEST, format!("{self}")).into_response()
				}
			}
		}
	}
}
