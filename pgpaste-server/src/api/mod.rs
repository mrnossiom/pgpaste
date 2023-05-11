use crate::AppState;
use axum::{
	extract::DefaultBodyLimit,
	http::StatusCode,
	routing::{get, Router},
};

mod create;
mod read;

pub(crate) fn api_router() -> Router<AppState> {
	Router::new()
		.route(
			"/paste/:slug",
			get(read::get_paste)
				.post(create::create_signed_paste)
				.delete(|| async { StatusCode::NOT_IMPLEMENTED }),
		)
		.route("/key/:fingerprint/list", get(read::get_key_pastes))
		// Set the limit to the default 2 MiB
		.layer(DefaultBodyLimit::max(2 * 1024))
}
