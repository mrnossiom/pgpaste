#![warn(
	// clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo,
	rustdoc::broken_intra_doc_links
)]
#![allow(clippy::redundant_pub_crate)]

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use config::Config;
use std::io;

mod args;
mod commands;
mod config;
mod crypto;

use crate::args::{Commands, PGPasteArgs};

fn main() -> eyre::Result<()> {
	pretty_env_logger::init();

	let args = PGPasteArgs::parse();
	let config = Config::new(&args)?;

	if let Some(generator) = args.generate {
		eprintln!("Generating completion file for {generator}...");

		let mut cmd = PGPasteArgs::command();
		let name = cmd.get_name().to_owned();

		generate(generator, &mut cmd, &name, &mut io::stdout());

		return Ok(());
	}

	if let Some(command) = args.command {
		match command {
			Commands::Create(create_args) => commands::create(create_args, &config)?,
			Commands::Read(read_args) => commands::read(read_args, &config)?,
		}
	};

	Ok(())
}

pub(crate) trait ToEyreError<T> {
	fn to_eyre(self) -> eyre::Result<T>;
}

impl<T> ToEyreError<T> for sequoia_openpgp::Result<T> {
	fn to_eyre(self) -> eyre::Result<T> {
		self.map_err(|err| eyre::eyre!(Box::new(err)))
	}
}
