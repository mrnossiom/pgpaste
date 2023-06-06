use crate::{
	args::ReadArgs,
	config::Config,
	crypto::{decrypt, verify},
};
use pgpaste_api_types::{api::ReadResponse, Visibility};
use reqwest::{blocking::Client, header::HeaderValue, StatusCode, Url};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn read(args: ReadArgs, config: &Config) -> eyre::Result<()> {
	let key = config
		.keys
		.clone()
		.ok_or(eyre::eyre!("no signing cert found"))?;

	let paste = get(config.server.clone(), &args.slug, &args)?;

	let content = match paste.visibility {
		Visibility::Public => verify(&paste.inner, key)?,
		Visibility::Protected | Visibility::Private => decrypt(&paste.inner, key)?,
	};

	println!("Your paste content:");
	println!("{content:?}");

	Ok(())
}

fn get(mut server: Url, slug: &str, _args: &ReadArgs) -> eyre::Result<ReadResponse> {
	let client = Client::default();

	server.set_path(&format!("/api/paste/{slug}"));

	let response = client.get(server).send()?;

	match response.status() {
		StatusCode::OK => {
			if let Some(content_type) = response.headers().get("content-type") {
				if HeaderValue::from_str("application/msgpack")? != content_type {
					eyre::bail!("Invalid content type");
				}
			};

			Ok(rmp_serde::from_slice(&response.bytes()?)?)
		}
		StatusCode::NOT_FOUND => eyre::bail!("Paste not found"),
		code => eyre::bail!("Unknown error: {}, {}", code, response.text()?),
	}
}
