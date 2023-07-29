use leptos::{mount_to_body, view};
use leptos_tutorial::App;

fn main() {
	// initializes logging using the `log` crate
	_ = console_log::init_with_level(log::Level::Debug);
	console_error_panic_hook::set_once();

	mount_to_body(move |cx| view! { cx, <App/> });
}
