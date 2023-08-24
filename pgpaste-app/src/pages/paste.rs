use leptos::{
	component, create_local_resource, create_resource, view, IntoView, Scope, SignalWith, Suspense,
};
use leptos_router::use_params_map;
use pgpaste_api_types::Paste;
use sequoia_openpgp_wasm::{parse::Parse, Message};

#[cfg(feature = "ssr")]
async fn contact_data_(id: &str) -> eyre::Result<Vec<u8>> {
	let path = format!("http://localhost:3000/api/paste/{}", id);
	let res = reqwest::get(path).await?;

	if res.status() == 200 {
		Ok(res.bytes().await?.to_vec())
	} else {
		let error = res.text().await?;
		eyre::bail!("Error while fetching: {error}")
	}
}

#[cfg(not(feature = "ssr"))]
// #[cfg(any(feature = "hydrate", feature = "csr"))]
async fn contact_data_(id: &str) -> eyre::Result<Vec<u8>> {
	use gloo_net::http::Request;

	let path = format!("http://localhost:3000/api/paste/{}", id);
	let res = Request::get(&path).send().await?;

	Ok(res.binary().await?)
}

async fn contact_data(id: String) -> eyre::Result<Paste> {
	let body = contact_data_(&id).await?;

	Ok(rmp_serde::from_slice(&body)?)
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
	let data = create_local_resource(
		cx,
		move || params.with(|p| p.get("slug").cloned().unwrap_or_default()),
		contact_data,
	);

	view! { cx,
		<div class="section is-large">
		<Suspense fallback=move || view! { cx, <p>"Loading paste..."<Spinner /></p> }>
			{move || {
				data.read(cx).map(|paste| paste.map_or_else(
					|| view! { cx, <pre>"Error"</pre> }.into_view(cx),
					|paste| view! { cx, <Display paste /> }.into_view(cx))
				)
			}}
		</Suspense>
		</div>
	}
}

#[component]
fn Spinner(cx: Scope) -> impl IntoView {
	view! { cx,
		<span>"Look I'm spinning"</span>
	}
}
