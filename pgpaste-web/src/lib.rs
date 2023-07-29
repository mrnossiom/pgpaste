use leptos::{component, view, IntoView, Scope};
use leptos_router::{Route, Router, Routes};

mod components;
mod pages;

use crate::{
	components::{Footer, Nav},
	pages::{About, Home, NotFound, Paste, Playground, Settings},
};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
	view! { cx,
		<Router fallback=move |cx| view! { cx, <NotFound /> }>
			<Nav />

			<main>
			<Routes>
				<Route
					path=""
					view=move |cx| view! { cx, <Home /> }
				/>
				<Route
					path="about"
					view=move |cx| view! { cx, <About /> }
				/>
				<Route
					path="paste/:slug"
					view=move |cx| view! { cx, <Paste /> }
				/>
				<Route
					path="playground"
					view=move |cx| view! { cx, <Playground /> }
				/>
				<Route
					path="settings"
					view=move |cx| view! { cx, <Settings /> }
				/>
			</Routes>
			</main>
		</Router>

		<Footer />
	}
}
