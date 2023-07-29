use leptos::{component, view, IntoView, Scope};

#[component]
pub(crate) fn Settings(cx: Scope) -> impl IntoView {
	view! { cx,
		<div class="section is-large">
			<h1>"You are settings"</h1>
		</div>
	}
}
