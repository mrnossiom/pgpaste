#[cfg(feature = "hydrate")]
pub(crate) async fn fetch(
	cx: leptos::Scope,
	path: &str,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
	use gloo_net::http::Request;
	use web_sys::AbortController;

	let abort_controller = AbortController::new().ok();
	let abort_signal = abort_controller.as_ref().map(|a| a.signal());

	leptos::on_cleanup(cx, move || {
		if let Some(abort_controller) = abort_controller {
			abort_controller.abort()
		}
	});

	let res = Request::get(path)
		.abort_signal(abort_signal.as_ref())
		.send()
		.await?;

	Ok(res)
}
