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
	/// Database pool connection error
	#[error("DbPool error: {0}")]
	Pool(#[from] deadpool::PoolError),

	/// Provide additional informations to the user
	#[error(transparent)]
	User(#[from] UserServerError),
}

impl IntoResponse for ServerError {
	fn into_response(self) -> Response {
		tracing::error!(error = ?self);

		match self {
			Self::Eyre(_) | Self::Pool(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
			Self::User(error) => error.into_response(),
		}
	}
}

// TODO: change `anyhow::Error` to `sequoia_openpgp::Error` when 2.0 is released
/// Error type that can provide additional informations to the user
#[derive(Debug, thiserror::Error)]
pub(crate) enum UserServerError {
	/// Invalid cert
	#[error("Invalid cert")]
	InvalidCert(eyre::Error),
	/// Invalid message structure
	#[error("Invalid message structure")]
	InvalidMessageStructure,
	/// Invalid signature
	#[error("Invalid signature")]
	InvalidSignature(eyre::Error),

	/// Certificate is not known within the default keyserver `keys.openpgp.org`
	#[error("Certificate is not known within the default keyserver `keys.openpgp.org`")]
	CertUnknown(eyre::Error),

	/// TODO
	#[error("Certificate is not known within the default keyserver `keys.openpgp.org`")]
	MsgPackBodyIsInvalid(#[from] rmp_serde::decode::Error),

	/// Queried paste not found
	#[error("Paste not found")]
	PasteNotFound,
	/// Burn date is too far in the future
	#[error("Burn date is too far in the future")]
	InvalidBurnIn,

	/// Paste is private and cannot be accessed like this
	#[error("Paste is private and cannot be accessed like this")]
	PasteIsPrivate,

	/// Paste is protected and cannot be accessed without a password
	#[error("Paste is protected and cannot be accessed without a password")]
	PasteIsProtected,
}

impl IntoResponse for UserServerError {
	fn into_response(self) -> Response {
		let code = match self {
			Self::InvalidCert(_)
			| Self::InvalidMessageStructure
			| Self::InvalidBurnIn
			| Self::InvalidSignature(_)
			| Self::CertUnknown(_)
			| Self::MsgPackBodyIsInvalid(_)
			| Self::PasteIsPrivate
			| Self::PasteIsProtected => StatusCode::BAD_REQUEST,

			Self::PasteNotFound => StatusCode::NOT_FOUND,
		};

		(code, format!("{self}")).into_response()
	}
}
