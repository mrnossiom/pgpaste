//! Implementation of the `create` subcommand

use crate::{
	args::CreateArgs,
	config::Config,
	crypto::{encrypt, protect, sign, SendHelper},
};
use eyre::ContextCompat;
use pgpaste_api_types::{
	api::{CreateBody, CreateResponse},
	Visibility,
};
use reqwest::{blocking::Client, header, Method, StatusCode, Url};
use rpassword::prompt_password;

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

	let bytes = match args.mode {
		Visibility::Public => sign(&content, &helper)?,
		Visibility::Private => {
			let recipient = args
				.recipient
				.clone()
				.or_else(|| config.default_key.clone())
				.wrap_err("no recipient specified")?;

			encrypt(&content, &helper, recipient)?
		}
		Visibility::Protected => {
			let paste_password = prompt_password("Password: ")?;
			protect(&content, &helper, &paste_password)?
		}
	};

	let res = post_paste(config.server.clone(), bytes, &args.slug, &args)?;

	println!("Your paste is available with the slug `{}`", res.slug);

	Ok(())
}

/// Post a paste to the server
fn post_paste(
	mut server: Url,
	content: Vec<u8>,
	slug: &Option<String>,
	args: &CreateArgs,
) -> eyre::Result<CreateResponse> {
	let client = Client::default();

	let query = CreateBody {
		slug: slug.clone(),
		mime: args.mime.clone().unwrap_or(mime::TEXT_PLAIN),
		visibility: args.mode,
		burn_in: args.burn_in()?,
		burn_after_read: args.burn_after_read,
		inner: content,
	};

	server.set_path("/api/paste");

	let method = if args.overwrite {
		Method::PUT
	} else {
		Method::POST
	};

	let response = client
		.request(method, server)
		.header(header::CONTENT_TYPE, "application/msgpack")
		.body(rmp_serde::to_vec(&query)?)
		.send()?;

	let response = match response.status() {
		StatusCode::CREATED => rmp_serde::from_slice::<CreateResponse>(&response.bytes()?)?,
		StatusCode::CONFLICT => eyre::bail!("Paste name already exists"),
		code => eyre::bail!("Unknown error: {}, {}", code, response.text()?),
	};

	Ok(response)
}
