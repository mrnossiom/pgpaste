use leptos::{component, view, IntoView, Scope};
use leptos_router::A;
use stylers::style_str;

const CLASS: (&str, &str) = style_str! {"Home",
	.hero {
		background: rgb(209,253,255);
		background: linear-gradient(140deg, rgba(209,253,255,1) 0%, rgba(253,219,146,1) 100%);
	}
};

#[component]
pub(crate) fn Home(cx: Scope) -> impl IntoView {
	view! { cx, class = {CLASS.0},
		<style>{CLASS.1}</style>
		<section class="hero is-medium">
			<div class="hero-head"></div>
			<div class="hero-body">
				<div class="container has-text-centered">
					<p class="title is-1">"PGPaste"</p>
					<p class="subtitle is-2">"Welcome to a pastebin-like service based on encryption."</p>

					<div class="buttons is-centered">
						<a class="button is-white is-large" target="_blank" href="https://github.com/MrNossiom/pgpaste">"GitHub"</a>
						<A class="button is-primary is-large" href="playground">"Playground"</A>
					</div>

					<p>"Or you can just install the CLI with "<code>"cargo install pgpaste-cli"</code></p>
				</div>
			</div>
			<div class="hero-foot"></div>
		</section>
	}
}
