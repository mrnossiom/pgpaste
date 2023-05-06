use crate::{
	database::{models::NewPaste, prelude::*, schema::pastes},
	error::ServerError,
	AppState,
};
use axum::{
	body::Bytes,
	extract::{Path, Query, State},
	http::StatusCode,
	response::IntoResponse,
};
use eyre::Context;
use pgpaste_api_types::CreateQuery;

pub(crate) async fn create_signed_paste(
	State(state): State<AppState>,
	Path(slug): Path<String>,
	Query(query): Query<CreateQuery>,
	content: Bytes,
) -> Result<impl IntoResponse, ServerError> {
	let mut conn = state
		.database
		.get()
		.await
		.wrap_err("Could not get a db handle")?;

	let paste = NewPaste {
		public_key_id: 1,

		slug: &slug,
		visibility: &query.visibility.into(),
		content: &content,
	};

	let id = paste
		.insert()
		.returning(pastes::id)
		.get_result::<i32>(&mut conn)
		.await
		.wrap_err("Could not insert paste")?;

	Ok((StatusCode::CREATED, id.to_string()))
}
