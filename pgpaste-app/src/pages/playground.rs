use leptos::{component, view, IntoView, Scope};

#[component]
pub(crate) fn Playground(cx: Scope) -> impl IntoView {
	view! { cx,
		<div class="section is-large">
			<h1>"You are playground"</h1>
		</div>
	}
}
