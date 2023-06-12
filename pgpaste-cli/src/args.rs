//! CLI definition

use clap::{value_parser, Args, Parser, Subcommand};
use clap_complete::Shell;
use duration_human::DurationHuman;
use mime::Mime;
use pgpaste_api_types::Visibility;
use sequoia_openpgp::{crypto::Password, KeyHandle};
use std::{
	io::{stdin, stdout, IsTerminal},
	path::PathBuf,
	time::Duration,
};

/// A `PasteBin` like service that lays on encryption
#[derive(Debug, Parser)]
#[clap(name = "pgpaste", author, version, about)]
#[clap(arg_required_else_help = true)]
pub(crate) struct PGPasteArgs {
	/// Which command to execute
	#[clap(subcommand)]
	pub(crate) command: Option<Commands>,

	/// The server URL to use
	#[clap(long)]
	pub(crate) server: Option<String>,

	/// Path to the config file
	#[clap(long)]
	pub(crate) config: Option<PathBuf>,

	/// Whether the output should be less verbose
	#[clap(long)]
	pub(crate) quiet: bool,

	/// Generate completion script for the given shell
	#[clap(long, value_parser = value_parser!(Shell))]
	pub(crate) generate: Option<Shell>,
}
/// Commands
#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
	/// Create a new paste
	Create(CreateArgs),

	/// Read an existing paste
	Read(ReadArgs),
}

/// Arguments to create a new paste
#[derive(Debug, Args)]
pub(crate) struct CreateArgs {
	/// The slug of the paste to create
	#[clap(long, short)]
	pub(crate) slug: Option<String>,

	/// The content of the paste
	#[clap(long, short, group = "message_content")]
	content: Option<String>,

	/// The file to read the content from
	#[clap(long, group = "message_content")]
	file: Option<PathBuf>,

	// TODO: guess mime type
	/// The mime type of the content
	#[clap(long, value_parser = parsers::to_mime)]
	pub(crate) mime: Option<Mime>,

	/// The visibility of the paste
	#[clap(long, short, value_parser = parsers::to_api_visibility)]
	pub(crate) mode: Visibility,

	/// The longevity of the paste
	#[clap(long, group = "time", value_parser = parsers::to_duration_human)]
	lifetime: Option<DurationHuman>,
	/// When should the paste be burned
	#[clap(long, group = "time", value_parser = parsers::to_do::<Option<String>>)]
	burn_date: Option<String>,
	/// Whether the paste should be burned after reading
	#[clap(long)]
	pub(crate) burn_after_read: bool,

	/// The recipient of the paste in case of a private paste
	#[clap(long, value_parser = parsers::to_key_handle)]
	pub(crate) recipient: Option<KeyHandle>,

	// TODO: implement obscuring recipients
	/// Whether the recipient(s) should be public in the encryption process
	/// using wildcard key
	#[clap(long)]
	pub(crate) obscure: bool,

	/// Overwrite an existing paste with the same slug and from the same account
	#[clap(long)]
	pub(crate) overwrite: bool,
}

impl CreateArgs {
	/// Get the content of the paste from different sources
	pub(crate) fn content(&self) -> eyre::Result<String> {
		let content = if let Some(content) = &self.content {
			content.clone()
		} else if let Some(file) = &self.file {
			std::fs::read_to_string(file)?
		} else if stdout().is_terminal() {
			std::io::read_to_string(stdin())?
		} else {
			eyre::bail!("I could not get paste content by a `--file`, a `--content` or stdin.")
		};

		Ok(content)
	}

	/// Get the lifetime of the paste
	#[allow(clippy::unnecessary_wraps)]
	pub(crate) fn burn_in(&self) -> eyre::Result<Option<Duration>> {
		let dur: Option<Duration> = self.lifetime.as_ref().map(Into::into);

		// TODO: implement and handle `burn_date`
		// see if SystemTime is the right type

		Ok(dur)
	}
}

/// Arguments to read an existing paste
#[derive(Debug, Args)]
pub(crate) struct ReadArgs {
	/// The slug of the paste to read
	#[clap(long, short)]
	pub(crate) slug: String,

	/// The password to decrypt the paste
	#[clap(long, short)]
	pub(crate) password: Option<Password>,
}

/// Clap value parsers
mod parsers {
	use duration_human::DurationHuman;
	use mime::Mime;
	use pgpaste_api_types::Visibility;
	use sequoia_openpgp::KeyHandle;

	/// Convert a visibility string to a `Visibility` enum
	pub(crate) fn to_api_visibility(visibility: &str) -> Result<Visibility, String> {
		match visibility {
			"public" => Ok(Visibility::Public),
			"protected" => Ok(Visibility::Protected),
			"private" => Ok(Visibility::Private),
			_ => Err("Available visibilities are `public`, `protected` and `private`".into()),
		}
	}

	/// Convert a duration string to a `DurationHuman` struct
	pub(crate) fn to_duration_human(duration: &str) -> Result<DurationHuman, String> {
		DurationHuman::try_from(duration).map_err(|err| err.to_string())
	}

	/// Convert a key handle string to a `KeyHandle` struct
	pub(crate) fn to_key_handle(handle: &str) -> Result<KeyHandle, String> {
		handle.parse::<KeyHandle>().map_err(|err| err.to_string())
	}

	/// Convert a mime type string to a `Mime` struct
	pub(crate) fn to_mime(mime_type: &str) -> Result<Mime, String> {
		mime_type.parse::<Mime>().map_err(|err| err.to_string())
	}

	/// Convert a duration string to a `Duration` struct
	pub(crate) fn to_do<T>(_duration: &str) -> Result<T, String> {
		panic!()
	}
}
