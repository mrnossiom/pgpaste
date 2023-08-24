use http::status::StatusCode;
use leptos::{component, create_rw_signal, view, Errors, For, IntoView, RwSignal, Scope};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum App {
	/// The page asked doesn't exist or was moved
	#[error("Not Found")]
	NotFound,
}

impl App {
	/// Retrieves the status code associated to the error
	#[must_use]
	pub const fn status_code(&self) -> StatusCode {
		match self {
			Self::NotFound => StatusCode::NOT_FOUND,
		}
	}
}

/// Displays errors to the user
#[component]
#[track_caller]
pub fn ErrorTemplate(
	cx: Scope,
	#[prop(optional)] outside_errors: Option<Errors>,
	#[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
	let errors = outside_errors
		.map(|err| create_rw_signal(cx, err))
		.xor(errors)
		.expect("either `outside_errors` or `errors` MUST be provided")();

	let errors: Vec<App> = errors
		.into_iter()
		.filter_map(|(_k, v)| v.downcast_ref::<App>().cloned())
		.collect();

	log::debug!("Errors: {errors:#?}");

	// (Server-side) We set the response code to the first error's status code
	#[cfg(feature = "ssr")]
	{
		use {leptos::use_context, leptos_axum::ResponseOptions};

		if let Some(response) = use_context::<ResponseOptions>(cx) {
			response.set_status(errors[0].status_code());
		}
	}

	view! { cx,
		<h1>{if errors.len() > 1 {"Errors"} else {"Error"}}</h1>
		<For
			each=move || {errors.clone().into_iter().enumerate()}
			key=|(index, _error)| *index
			view=move |cx, (_index, error)| { view! { cx,
				<h2>{error.status_code().to_string()}</h2>
				<p>"Error: " {error.to_string()}</p>
			}}
		/>
	}
}
