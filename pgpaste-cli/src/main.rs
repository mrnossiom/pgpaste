#![warn(
	// clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	// clippy::cargo,
	rustdoc::broken_intra_doc_links
)]
#![allow(clippy::redundant_pub_crate)]

use clap::Parser;
use config::Config;

mod args;
mod commands;
mod config;

use crate::args::{Commands, PGPasteArgs};

fn main() -> eyre::Result<()> {
	let args = PGPasteArgs::parse();
	let config = Config::new(&args)?;

	match &args.command {
		Commands::Create(create_args) => commands::create(create_args, &config)?,
		Commands::Read(read_args) => commands::read(read_args, &config)?,
	}

	Ok(())
}
