use leptos::{component, view, IntoView, Scope};

#[component]
pub(crate) fn About(cx: Scope) -> impl IntoView {
	view! { cx,
		<div class="section is-large">
			<h1>"You are about"</h1>
		</div>
	}
}
