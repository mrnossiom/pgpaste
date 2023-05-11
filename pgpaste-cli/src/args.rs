use clap::{Args, Parser, Subcommand};
use pgpaste_api_types::Visibility;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub(crate) struct PGPasteArgs {
	#[clap(subcommand)]
	pub(crate) command: Commands,

	#[clap(long)]
	pub(crate) server: Option<String>,

	#[clap(long)]
	pub(crate) config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
	Create(CreateArgs),
	Read(ReadArgs),
}

#[derive(Debug, Args)]
pub(crate) struct CreateArgs {
	pub(crate) slug: Option<String>,

	#[clap(long, short, group = "content")]
	pub(crate) content: Option<String>,

	#[clap(long, short, group = "content")]
	pub(crate) file: Option<PathBuf>,

	#[clap(long, value_parser = to_api_visibility)]
	pub(crate) mode: Visibility,
}

#[derive(Debug, Args)]
pub(crate) struct ReadArgs {
	pub(crate) slug: String,
}

fn to_api_visibility(visibility: &str) -> Result<Visibility, String> {
	match visibility {
		"public" => Ok(Visibility::Public),
		"protected" => Ok(Visibility::Protected),
		"private" => Ok(Visibility::Private),
		_ => Err("Available visibilities are `public`, `protected` and `private`".into()),
	}
}
