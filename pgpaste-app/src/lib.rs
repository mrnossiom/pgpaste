//! # The pgpaste web interface

#![warn(
	missing_docs,
	// clippy::missing_docs_in_private_items,
	clippy::print_literal,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo,
	rustdoc::broken_intra_doc_links
)]
#![allow(clippy::redundant_pub_crate, clippy::module_name_repetitions)]
#![cfg_attr(not(feature = "ssr"), allow(clippy::future_not_send))]

use leptos::{component, view, IntoView, Scope};
use leptos_meta::{provide_meta_context, Stylesheet};
use leptos_router::{Route, Router, Routes};

mod components;
pub mod error;
mod pages;
mod utils;

use crate::{
	components::{Footer, Nav},
	pages::{About, Home, NotFound, Paste, Playground, Settings},
};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
	provide_meta_context(cx);

	view! { cx,
		<Stylesheet href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css" />
		<Stylesheet href="/static/reset.css" />

		<Router fallback=move |cx| view! { cx, <NotFound /> }>
			<Nav />

			<main>
			<Routes>
				<Route path="" view=Home />
				<Route path="about" view=About />
				<Route path="paste/:slug" view=Paste />
				<Route path="playground" view=Playground />
				<Route path="settings" view=Settings />
			</Routes>
			</main>
		</Router>

		<Footer />
	}
}
