use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
};
use diesel_async::pooled_connection::deadpool;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ServerError {
	#[error(transparent)]
	Eyre(#[from] eyre::Report),
	#[error("Diesel error: {0}")]
	Database(#[from] diesel::result::Error),
	#[error("DbPool error: {0}")]
	Pool(#[from] deadpool::PoolError),

	#[error(transparent)]
	UserFacing(#[from] UserFacingServerError),
}

impl IntoResponse for ServerError {
	fn into_response(self) -> Response {
		tracing::error!(error = ?self);

		match self {
			Self::Database(_) | Self::Eyre(_) | Self::Pool(_) => {
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
			Self::UserFacing(error) => error.into_response(),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum UserFacingServerError {
	#[error("Invalid content type")]
	InvalidContentType,
	#[error("Invalid cert")]
	InvalidCert,

	#[error("Paste burned")]
	PasteBurned,
	#[error("Paste not found")]
	PasteNotFound,
}

impl IntoResponse for UserFacingServerError {
	fn into_response(self) -> Response {
		tracing::error!(error = ?self);

		match self {
			Self::InvalidContentType => {
				(StatusCode::BAD_REQUEST, "Invalid content type").into_response()
			}
			Self::InvalidCert => (StatusCode::BAD_REQUEST, "Invalid cert").into_response(),

			Self::PasteBurned => (StatusCode::GONE, "Paste burned").into_response(),
			Self::PasteNotFound => (StatusCode::NOT_FOUND, "Paste not found").into_response(),
		}
	}
}
