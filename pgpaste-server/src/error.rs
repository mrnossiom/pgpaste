//! Error types returned by handlers

use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
};
use diesel_async::pooled_connection::deadpool;

/// Error type returned by path handlers
#[derive(Debug, thiserror::Error)]
pub(crate) enum ServerError {
	// -- Internal server errors
	/// Generic error with context
	#[error(transparent)]
	Eyre(#[from] eyre::Report),
	/// Database transaction error
	#[error("Diesel error: {0}")]
	Database(#[from] diesel::result::Error),
	/// Database pool connection error
	#[error("DbPool error: {0}")]
	Pool(#[from] deadpool::PoolError),

	/// Provide additional informations to the user
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
/// Error type that can provide additional informations to the user
#[derive(Debug, thiserror::Error)]
pub(crate) enum UserFacingServerError {
	/// Invalid content type
	#[error("Invalid content type")]
	InvalidContentType,
	/// Invalid cert
	#[error("Invalid cert")]
	InvalidCert(eyre::Error),
	/// Invalid message structure
	#[error("Invalid message structure")]
	InvalidMessageStructure,
	/// Invalid signature
	#[error("Invalid signature")]
	InvalidSignature(eyre::Error),

	/// Queried paste not found
	#[error("Paste not found")]
	PasteNotFound,
	/// Burn date is too far in the future
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
