use crate::{
	args::CreateArgs,
	config::Config,
	crypto::{encrypt, protect, sign},
	ToEyreError,
};
use async_compat::Compat;
use pgpaste_api_types::{
	api::{CreateBody, CreateResponse},
	Visibility,
};
use reqwest::{blocking::Client, header, Method, StatusCode, Url};
use rpassword::prompt_password;
use sequoia_net::{KeyServer, Policy};
use smol::block_on;
use std::borrow::Cow;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create(args: CreateArgs, config: &Config) -> eyre::Result<()> {
	let key = config
		.keys
		.clone()
		.ok_or(eyre::eyre!("no signing cert found"))?;
	let content = args.content()?;

	let bytes = match args.mode {
		Visibility::Public => sign(&content, &key)?,
		Visibility::Private => {
			// TODO: get recipient key by handle using local keyring

			let recipient_key = match &args.recipient {
				Some(handle) => {
					let mut key_server =
						KeyServer::keys_openpgp_org(Policy::Encrypted).to_eyre()?;

					let cert = block_on(Compat::new(async {
						key_server.get(handle.clone()).await.to_eyre()
					}))?;

					Cow::Owned(cert)
				}
				None => Cow::Borrowed(&key),
			};

			encrypt(&content, &key, &recipient_key)?
		}
		Visibility::Protected => {
			let pass = prompt_password("Password: ")?;
			protect(&content, &key, &pass)?
		}
	};

	let res = post(config.server.clone(), bytes, &args.slug, &args)?;

	println!("Your paste is available with the slug `{}`", res.slug);

	Ok(())
}

fn post(
	mut server: Url,
	content: Vec<u8>,
	slug: &Option<String>,
	args: &CreateArgs,
) -> eyre::Result<CreateResponse> {
	let client = Client::default();

	let query = CreateBody {
		slug: slug.clone(),
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
