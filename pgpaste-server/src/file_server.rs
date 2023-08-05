//! A basic file server handler that defaults to Leptos error page

use axum::{
	body::{boxed, Body},
	extract::State,
	http::{Request, StatusCode, Uri},
	response::{IntoResponse, Response},
};
use leptos::{view, Errors};
use pgpaste_app::error::{AppError, ErrorTemplate};
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::config::AppState;

/// Handles static files and errors
pub(crate) async fn file_and_error_handler(
	uri: Uri,
	State(options): State<AppState>,
	request: Request<Body>,
) -> Response {
	let root = options.config.leptos.site_root.clone();
	let response = get_static_file(uri.clone(), &root).await;

	if response.status() == StatusCode::OK {
		response.into_response()
	} else {
		let mut errors = Errors::default();
		errors.insert_with_default_key(AppError::NotFound);
		let handler = leptos_axum::render_app_to_stream(
			options.config.leptos.clone(),
			move |cx| view! {cx, <ErrorTemplate outside_errors=errors.clone()/>},
		);
		handler(request).await.into_response()
	}
}

/// Retrieves a static file
async fn get_static_file(uri: Uri, root: &str) -> Response {
	let request = Request::builder()
		.uri(uri.clone())
		.body(Body::empty())
		.expect("body is correct");

	ServeDir::new(root)
		.oneshot(request)
		.await
		.map(|res| res.map(boxed))
		.expect("infaillible")
}
