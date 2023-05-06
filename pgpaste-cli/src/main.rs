use clap::Parser;

mod args;
mod commands;

use crate::args::{Commands, PGPasteArgs};

fn main() -> eyre::Result<()> {
	let args = PGPasteArgs::parse();

	match args.command {
		Commands::Create(create_args) => commands::create(&create_args)?,
		Commands::Read(read_args) => commands::read(&read_args)?,
	}

	Ok(())
}
