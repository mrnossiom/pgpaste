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

// TODO: change `anyhow::Error` to `sequoia_openpgp::Error` when 2.0 is released
#[derive(Debug, thiserror::Error)]
pub(crate) enum UserFacingServerError {
	#[error("Invalid content type")]
	InvalidContentType,
	#[error("Invalid cert")]
	InvalidCert(anyhow::Error),
	#[error("Invalid message structure")]
	InvalidMessageStructure,
	#[error("Invalid signature")]
	InvalidSignature(eyre::Error),

	#[error("Paste not found")]
	PasteNotFound,
	#[error("Burn date is too far in the future")]
	InvalidBurnIn,
}

impl IntoResponse for UserFacingServerError {
	fn into_response(self) -> Response {
		let code = match self {
			Self::InvalidCert(_)
			| Self::InvalidContentType
			| Self::InvalidMessageStructure
			| Self::InvalidBurnIn
			| Self::InvalidSignature(_) => StatusCode::BAD_REQUEST,

			Self::PasteNotFound => StatusCode::NOT_FOUND,
		};

		(code, format!("{self}")).into_response()
	}
}
