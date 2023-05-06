use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub(crate) struct PGPasteArgs {
	#[clap(subcommand)]
	pub(crate) command: Commands,

	#[clap(long, default_value = "http://localhost:3000")]
	pub(crate) server: String,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
	Create(CreateArgs),
	Read(ReadArgs),
}

#[derive(Debug, Args)]
pub(crate) struct CreateArgs {
	/// A complex argument.
	///
	/// - If the argument begins with @, it will be interpreted as a file path.
	/// - If the argument is empty, it will be read from stdin.
	///
	/// TODO : actually you need to implement this
	pub(crate) content: String,

	#[clap(long)]
	pub(crate) private: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ReadArgs {
	pub(crate) slug: String,
}
