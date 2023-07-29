use gloo_net::http::Request;
use leptos::{component, create_resource, view, IntoView, Scope, SignalWith, Suspense};
use leptos_router::use_params_map;
use pgpaste_api_types::Paste;
use sequoia_openpgp::{parse::Parse, Message};

async fn contact_data(id: String) -> Option<Paste> {
	let path = format!("https://localhost:3000/api/paste/{}", id);
	let res = Request::get(&path).send().await.unwrap();

	let body = res.binary().await.unwrap();

	rmp_serde::from_slice(&body).ok()
}

#[component]
fn Display(cx: Scope, paste: Paste) -> impl IntoView {
	let unlocked = Message::from_bytes(&paste.inner).unwrap();
	let inner = unlocked.body().unwrap();

	view! { cx,
		<div class="section">
			<h1>"Display "{paste.slug}</h1>
			<p>{String::from_utf8_lossy(inner.body())}</p>
		</div>
	}
}

#[component]
pub(crate) fn Paste(cx: Scope) -> impl IntoView {
	let params = use_params_map(cx);
	let data = create_resource(
		cx,
		move || params.with(|p| p.get("slug").cloned().unwrap_or_default()),
		contact_data,
	);

	view! { cx,
		<div class="section is-large">
		<Suspense fallback=move || view! { cx, <p>"Loading (Suspense Fallback)..."</p> }>
			{move || {
				data.read(cx).map(|paste| match paste {
				None => view! { cx, <pre>"Error"</pre> }.into_view(cx),
				Some(paste) => view! { cx, <Display paste /> }.into_view(cx),
			})}}
		</Suspense>
		</div>
	}
}
