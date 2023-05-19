use clap::{value_parser, Args, Parser, Subcommand};
use clap_complete::Shell;
use duration_human::DurationHuman;
use pgpaste_api_types::Visibility;
use sequoia_openpgp::KeyHandle;
use std::{io::stdin, path::PathBuf, time::Duration};

#[derive(Debug, Parser)]
#[clap(name = "pgpaste", author, version, about)]
#[clap(arg_required_else_help = true)]
pub(crate) struct PGPasteArgs {
	#[clap(subcommand)]
	pub(crate) command: Option<Commands>,

	#[clap(long)]
	pub(crate) server: Option<String>,

	#[clap(long)]
	pub(crate) config: Option<PathBuf>,

	#[clap(long, value_parser = value_parser!(Shell))]
	pub(crate) generate: Option<Shell>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
	Create(CreateArgs),
	Read(ReadArgs),
}

#[derive(Debug, Args)]
pub(crate) struct CreateArgs {
	#[clap(long, short)]
	pub(crate) slug: Option<String>,

	#[clap(long, short, group = "message_content")]
	content: Option<String>,

	#[clap(long, group = "message_content")]
	file: Option<PathBuf>,

	#[clap(long, short, value_parser = parsers::to_api_visibility)]
	pub(crate) mode: Visibility,

	#[clap(long, group = "time", value_parser = parsers::to_duration_human)]
	lifetime: Option<DurationHuman>,
	// TODO
	#[clap(long, group = "time", value_parser = parsers::to_do::<Option<String>>)]
	burn_date: Option<String>,

	#[clap(long)]
	pub(crate) burn_after_read: bool,

	#[clap(long, value_parser = parsers::to_key_handle)]
	pub(crate) recipient: Option<KeyHandle>,

	#[clap(long)]
	pub(crate) overwrite: bool,
}

impl CreateArgs {
	pub(crate) fn content(&self) -> eyre::Result<String> {
		let content = if let Some(content) = &self.content {
			content.clone()
		} else if let Some(file) = &self.file {
			std::fs::read_to_string(file)?
		} else if atty::isnt(atty::Stream::Stdin) {
			std::io::read_to_string(stdin())?
		} else {
			eyre::bail!("I could not get paste content by a `--file`, a `--content` or stdin.")
		};

		Ok(content)
	}

	// TODO: see if SystemTime is the right type
	#[allow(clippy::unnecessary_wraps)]
	pub(crate) fn burn_in(&self) -> eyre::Result<Option<Duration>> {
		let dur: Option<Duration> = self.lifetime.as_ref().map(Into::into);

		// TODO: implement and handle `burn_date`

		Ok(dur)
	}
}

#[derive(Debug, Args)]
pub(crate) struct ReadArgs {
	pub(crate) slug: String,
}

mod parsers {
	use duration_human::DurationHuman;
	use pgpaste_api_types::Visibility;
	use sequoia_openpgp::KeyHandle;

	pub(crate) fn to_api_visibility(visibility: &str) -> Result<Visibility, String> {
		match visibility {
			"public" => Ok(Visibility::Public),
			"protected" => Ok(Visibility::Protected),
			"private" => Ok(Visibility::Private),
			_ => Err("Available visibilities are `public`, `protected` and `private`".into()),
		}
	}

	pub(crate) fn to_duration_human(duration: &str) -> Result<DurationHuman, String> {
		DurationHuman::try_from(duration).map_err(|err| err.to_string())
	}

	pub(crate) fn to_key_handle(handle: &str) -> Result<KeyHandle, String> {
		handle.parse::<KeyHandle>().map_err(|err| err.to_string())
	}

	pub(crate) fn to_do<T>(_duration: &str) -> Result<T, String> {
		panic!()
	}
}
