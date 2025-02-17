//! Implementation of the `create` subcommand

use eyre::ContextCompat;
use pgpaste_api_types::{
	Visibility,
	api::{CreateBody, CreateResponse},
};
use reqwest::{Method, StatusCode, Url, blocking::Client, header};
use rpassword::prompt_password;

use crate::{
	args::CreateArgs,
	config::Config,
	crypto::{SendHelper, encrypt, protect, sign},
};

// TODO: fix the need to enter the password two times for private pastes

#[allow(clippy::needless_pass_by_value)]
/// Create a paste on the server
pub(crate) fn create(args: CreateArgs, config: &Config) -> eyre::Result<()> {
	let content = args.content()?;
	let helper = SendHelper::new(
		&config
			.default_key
			.clone()
			.wrap_err("you need to choose a key")?,
		&config.private_keys,
		&config.public_keys,
	)?;

	let message = match args.mode {
		Visibility::Public => sign(&content, &helper)?,
		Visibility::Private => {
			let recipient = args
				.recipient
				.clone()
				.or_else(|| config.default_key.clone())
				.wrap_err("no recipient specified")?;

			encrypt(&content, &helper, recipient, args.sign_private)?
		}
		Visibility::Protected => {
			let paste_password = prompt_password("Password: ")?;
			protect(&content, &helper, &paste_password, args.sign_private)?
		}
	};

	#[cfg(debug_assertions)]
	if let Some(path) = &args.dump_message {
		std::fs::write(path, &message)?;
		log::debug!("Wrote message to {}", path.display());
	}

	let query = CreateBody {
		slug: args.slug.clone(),
		mime: args.mime.clone().unwrap_or(mime::TEXT_PLAIN),
		visibility: args.mode,
		burn_in: args.burn_in()?,
		burn_after_read: args.burn_after_read,
		message,
	};

	let query = rmp_serde::to_vec(&query)?;

	let signed_query = sign(&query, &helper)?;

	let res = post_paste(&config.server, &args, signed_query)?;

	log::info!("Your paste is available with the slug `{}`", res.slug);

	Ok(())
}

/// Post a paste to the server
fn post_paste(server: &Url, args: &CreateArgs, query: Vec<u8>) -> eyre::Result<CreateResponse> {
	let client = Client::default();

	let method = if args.overwrite {
		Method::PUT
	} else {
		Method::POST
	};

	let response = client
		.request(method, server.join("/api/paste")?)
		.header(header::CONTENT_TYPE, "application/pgp-signature")
		.body(query)
		.send()?;

	let response = match response.status() {
		StatusCode::CREATED => rmp_serde::from_slice::<CreateResponse>(&response.bytes()?)?,
		StatusCode::CONFLICT => eyre::bail!("Paste name already exists"),
		code => eyre::bail!("Unknown error: {}, {}", code, response.text()?),
	};

	Ok(response)
}
