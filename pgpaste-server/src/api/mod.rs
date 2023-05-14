use crate::AppState;
use axum::{
	extract::DefaultBodyLimit,
	http::StatusCode,
	routing::{get, post, Router},
};

mod create;
mod read;

pub(crate) fn api_router() -> Router<AppState> {
	Router::new()
		.route("/paste", post(create::create_signed_paste))
		.route(
			"/paste/:slug",
			get(read::get_paste).delete(|| async { StatusCode::NOT_IMPLEMENTED }),
		)
		.route("/key/:fingerprint/list", get(read::get_key_pastes))
		// Set the limit to the default 2 MiB
		.layer(DefaultBodyLimit::max(2 * 1024))
}

mod extract {
	use axum::{
		async_trait,
		body::{Bytes, HttpBody},
		extract::{rejection::BytesRejection, FromRequest},
		http::{header, HeaderMap, HeaderValue, Request, StatusCode},
		response::{IntoResponse, Response},
		BoxError,
	};
	use bytes::{Buf, BufMut, BytesMut};
	use rmp_serde::{Deserializer, Serializer};
	use serde::{de::DeserializeOwned, Serialize};

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
			if !dlhn_content_type(req.headers()) {
				return Err(MsgPackRejection::MissingDlhnContentType);
			}

			let mut bytes = Bytes::from_request(req, state).await?.reader();
			let mut deserializer = Deserializer::new(&mut bytes);

			let value = T::deserialize(&mut deserializer)?;

			Ok(Self(value))
		}
	}

	fn dlhn_content_type(headers: &HeaderMap) -> bool {
		let Some(content_type) = headers.get(header::CONTENT_TYPE) else { return false; };

		let Ok(content_type) = content_type.to_str() else { return false; };

		let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
			mime
		} else {
			return false;
		};

		let is_json_content_type = mime.type_() == "application"
			&& (mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json"));

		is_json_content_type
	}

	impl<T> From<T> for MsgPack<T> {
		fn from(inner: T) -> Self {
			Self(inner)
		}
	}

	impl<T> IntoResponse for MsgPack<T>
	where
		T: Serialize,
	{
		fn into_response(self) -> Response {
			// Use a small initial capacity of 128 bytes like serde_json::to_vec
			// https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
			let mut buf = BytesMut::with_capacity(128).writer();
			let mut ser = Serializer::new(&mut buf);

			match self.0.serialize(&mut ser) {
				Ok(()) => (
					[(
						header::CONTENT_TYPE,
						HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
					)],
					buf.into_inner().freeze(),
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

	#[derive(Debug, thiserror::Error)]
	#[non_exhaustive]
	/// Rejection used for [`Json`](super::Json).
	///
	/// Contains one variant for each way the [`Json`](super::Json) extractor
	/// can fail.
	pub enum MsgPackRejection {
		#[error(transparent)]
		Bytes(#[from] BytesRejection),
		#[error(transparent)]
		Decode(#[from] rmp_serde::decode::Error),
		#[error("missing dlhn content type")]
		MissingDlhnContentType,
	}

	impl IntoResponse for MsgPackRejection {
		fn into_response(self) -> Response {
			match self {
				Self::Bytes(err) => err.into_response(),
				Self::Decode(err) => (StatusCode::BAD_REQUEST, format!("{}", err)).into_response(),
				Self::MissingDlhnContentType => {
					(StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
				}
			}
		}
	}
}
