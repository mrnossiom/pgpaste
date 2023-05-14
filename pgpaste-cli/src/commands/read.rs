use crate::{args::ReadArgs, config::Config};
use reqwest::{blocking::Client, StatusCode, Url};
use sequoia_openpgp::{parse::Parse, Message};

pub(crate) fn read(args: &ReadArgs, config: &Config) -> eyre::Result<()> {
	// TODO
	let _key = config.keys.clone().ok_or(eyre::eyre!(""))?;

	let res = get(config.server.clone(), &args.slug, args)?;

	println!("Your paste content:");

	println!("{:?}", res);

	Ok(())
}

fn get(mut server: Url, slug: &str, _args: &ReadArgs) -> eyre::Result<Message> {
	let client = Client::default();

	server.set_path(&format!("/api/paste/{}", slug));

	let response = client.get(server).send()?;

	let bytes = match response.status() {
		StatusCode::OK => response.bytes()?,
		StatusCode::NOT_FOUND => eyre::bail!("Paste not found"),
		_ => eyre::bail!("Unknown error"),
	};

	Message::from_bytes(&bytes).map_err(|err| eyre::eyre!(Box::new(err)))
}
