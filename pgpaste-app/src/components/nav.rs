use leptos::{
	component, create_node_ref, create_signal, html::Input, view, IntoView, NodeRef, Scope,
	SignalUpdate,
};
use leptos_icons::{FiIcon, Icon, SiIcon};
use leptos_router::{use_navigate, NavigateOptions, A};
use stylers::style_str;

const CLASS: (&str, &str) = style_str! {"Nav",
	nav {}
};

#[component]
pub(crate) fn Nav(cx: Scope) -> impl IntoView {
	let (active_burger, set_active_burger) = create_signal(cx, false);

	let input_ref: NodeRef<Input> = create_node_ref(cx);
	let on_submit = move |_| {
		let value = input_ref().unwrap().value();

		if !value.is_empty() {
			let path = format!("/paste/{}", value);
			use_navigate(cx)(&path, NavigateOptions::default()).unwrap();
		}
	};

	// it seems to be a bug in the view! macro
	#[allow(unused_imports)]
	use leptos::IntoClass;

	view! { cx, class = {CLASS.0},
		<style>{CLASS.1}</style>
		<nav class="navbar" role="navigation" aria-label="main navigation">
			<div class="navbar-brand">
				<A class="navbar-item pgpaste-logo" href="/">
					// TODO: exchange name for a logo
					<span class="title">"PGPaste"</span>
				</A>

				<a
					class="navbar-burger"
					class:is-active=active_burger
					on:click=move |_| set_active_burger.update(|b| *b = !*b)
					role="button"
					aria-label="menu"
					aria-expanded="false"
				>
					<span aria-hidden="true"></span>
					<span aria-hidden="true"></span>
					<span aria-hidden="true"></span>
				</a>
			</div>

			<div class="navbar-menu" class:is-active=active_burger>
				<div class="navbar-start"></div>
				<div class="navbar-end">
					// TODO: add useful links
					<a class="navbar-item icon-text" href="https://github.com/MrNossiom/pgpaste">
						<span class="icon">
							<Icon icon=Icon::Si(SiIcon::SiGithub) />
						</span>
						<span>
							"GitHub"
							<Icon width=".75em" height=".75em" icon=Icon::Fi(FiIcon::FiExternalLink) />
						</span>
					</a>
					<A class="navbar-item" href="about">"About"</A>
					<A class="navbar-item" href="playground">"Playground"</A>

					<div class="navbar-item">
						<div class="field is-grouped">
							<div class="control">
								<input
									node_ref=input_ref
									class="input"
									type="text"
									placeholder="Paste ID"
								/>
							</div>
							<div class="control">
								<button class="button is-primary" on:click=on_submit>
									<span class="icon">
										<Icon icon=Icon::Fi(FiIcon::FiSearch) />
									</span>
									<span>"Read"</span>
								</button>
							</div>
						</div>
					</div>
				</div>
			</div>
		</nav>
	}
}
