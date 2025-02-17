//! Implementation of the `read` subcommand

use std::borrow::Cow;

use pgpaste_api_types::{Visibility, api::ReadResponse};
use reqwest::{
	StatusCode, Url,
	blocking::Client,
	header::{self, HeaderValue},
};

use crate::{
	args::ReadArgs,
	config::Config,
	crypto::{ReceiveHelper, decrypt, verify},
};

#[allow(clippy::needless_pass_by_value)]
/// Read a paste from the server
pub(crate) fn read(args: ReadArgs, config: &Config) -> eyre::Result<()> {
	let paste = get_paste(config.server.clone(), &args.slug, &args)?;

	let helper = ReceiveHelper::new(&config.private_keys, &config.public_keys)?;

	let content = match paste.visibility {
		Visibility::Public => verify(&paste.inner, helper)?,
		Visibility::Protected | Visibility::Private => decrypt(&paste.inner, helper)?,
	};

	let content = match paste.mime {
		text if text == mime::TEXT_PLAIN => String::from_utf8_lossy(&content),
		_ => Cow::Owned(format!("{:?}", &content)),
	};

	log::info!("Your paste content:");
	log::info!("{content}");

	Ok(())
}

/// Get a paste from the server
fn get_paste(mut server: Url, slug: &str, _args: &ReadArgs) -> eyre::Result<ReadResponse> {
	let client = Client::default();

	server.set_path(&format!("/api/paste/{slug}"));

	let response = client.get(server).send()?;

	match response.status() {
		StatusCode::OK => {
			if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
				if HeaderValue::from_str(mime::APPLICATION_MSGPACK.as_ref())? != content_type {
					eyre::bail!("Invalid content type");
				}
			};

			Ok(rmp_serde::from_slice(&response.bytes()?)?)
		}
		StatusCode::NOT_FOUND => eyre::bail!("Paste not found"),
		code => eyre::bail!("Unknown error: {}, {}", code, response.text()?),
	}
}
