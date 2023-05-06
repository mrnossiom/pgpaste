use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
};

pub(crate) enum ServerError {
	Eyre(eyre::Report),
	DatabaseError(diesel::result::Error),
}

impl From<eyre::Report> for ServerError {
	fn from(e: eyre::Report) -> Self {
		ServerError::Eyre(e)
	}
}
impl From<diesel::result::Error> for ServerError {
	fn from(e: diesel::result::Error) -> Self {
		ServerError::DatabaseError(e)
	}
}

impl IntoResponse for ServerError {
	fn into_response(self) -> Response {
		let status_code = match self {
			ServerError::DatabaseError(_) | ServerError::Eyre(_) => {
				StatusCode::INTERNAL_SERVER_ERROR
			}
		};

		status_code.into_response()
	}
}
