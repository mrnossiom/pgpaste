use crate::{args::ReadArgs, config::Config};
use pgpaste_api_types::{api::ReadResponse, Visibility};
use reqwest::{blocking::Client, StatusCode, Url};
use sequoia_openpgp::{parse::Parse, Message};

pub(crate) fn read(args: &ReadArgs, config: &Config) -> eyre::Result<()> {
	// TODO
	let _key = config.keys.clone().ok_or(eyre::eyre!(""))?;

	let paste = get(config.server.clone(), &args.slug, args)?;
	let cert = Message::from_bytes(&paste.inner).map_err(|err| eyre::eyre!(Box::new(err)))?;

	let content = match paste.visibility {
		Visibility::Public => {
			println!("This paste is public, anyone can read it.");

			let mut content = String::new();

			for packet in cert.descendants() {
				if let sequoia_openpgp::Packet::Literal(literal) = packet {
					println!("Found literal packet");
					content = String::from_utf8(literal.body().to_vec())?;
					break;
				}
			}

			content
		}
		Visibility::Protected => {
			println!("This paste is unlisted, only people with the link can read it.");

			todo!()
		}
		Visibility::Private => {
			println!("This paste is private, only you can read it.");

			todo!()
		}
	};

	println!("Your paste content:");
	println!("{content:?}");

	Ok(())
}

fn get(mut server: Url, slug: &str, _args: &ReadArgs) -> eyre::Result<ReadResponse> {
	let client = Client::default();

	server.set_path(&format!("/api/paste/{slug}"));

	let response = client.get(server).send()?;

	// TODO
	if response
		.headers()
		.get("content-type")
		.ok_or(eyre::eyre!("No content type header"))?
		.to_str()?
		!= "application/msgpack"
	{
		eyre::bail!("Invalid content type");
	}

	match response.status() {
		StatusCode::OK => Ok(rmp_serde::from_slice(&response.bytes()?)?),
		StatusCode::NOT_FOUND => eyre::bail!("Paste not found"),
		code => eyre::bail!("Unknown error: {}, {}", code, response.text()?),
	}
}
