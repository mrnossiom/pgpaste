use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use gloo::timers::future::sleep;
use leptos::{mount_to, view};
use pgpaste_app::App;
use std::time::Duration;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;

#[wasm_bindgen_test]
async fn clear() -> Result<(), JsValue> {
	let document = leptos::document();
	let test_node = document.create_element("section")?;
	document.body().unwrap().append_child(&test_node)?;

	// We mount our initial app
	mount_to(
		test_node.clone().unchecked_into(),
		|cx| view! { cx, <App /> },
	);

	assert_eq!("/", &document.location().unwrap().pathname()?);

	// TODO: propose `data-test-id` attributes to leptos
	let nav = test_node.query_selector("nav")?.unwrap();
	let about: HtmlElement = nav
		.query_selector(r#"a[href="/about"]"#)?
		.unwrap()
		.unchecked_into();

	about.click();

	sleep(Duration::from_millis(100)).await;

	assert_eq!("/about", &document.location().unwrap().pathname()?);

	Ok(())
}
