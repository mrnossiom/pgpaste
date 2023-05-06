use reqwest::blocking::{multipart::Form, Client};
use serde_json::{Map, Value};

use crate::args::CreateArgs;
use std::{fs::read_to_string, io::stdin, path::PathBuf};

pub(crate) fn create(args: &CreateArgs) -> eyre::Result<()> {
	let content = if let Some(file) = Option::<PathBuf>::None {
		read_to_string(file)?
	} else if atty::isnt(atty::Stream::Stdin) {
		stdin().lines().collect::<Result<Vec<_>, _>>()?.join("\n")
	} else {
		eyre::bail!("You need to provide content either via stdin or a file.");
	};

	let res = if args.private {
		todo!()
	} else {
		post(&content, args)?
	};

	println!("{}", res);

	Ok(())
}

fn post(content: &str, args: &CreateArgs) -> eyre::Result<String> {
	let client = Client::default();

	let form = Form::new()
		.text(
			"meta",
			serde_json::to_string(&Value::Object(Map::default()))?,
		)
		.text("content", content.to_owned());

	let response = client.post(&args.content).multipart(form).send()?;

	Ok(response.text()?)
}
