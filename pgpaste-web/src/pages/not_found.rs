use leptos::{component, view, IntoView, Scope};

#[component]
pub(crate) fn NotFound(cx: Scope) -> impl IntoView {
	view! { cx,
		<section class="section is-large">
			<h1 class="title">"404 Not Found"</h1>
			<h2 class="subtitle">
				"The page you are trying to "<strong>"access"</strong>" doesn't exists anymore. Return to the homepage by clicking on our logo."
			</h2>
		</section>
	}
}
