//! `pgpaste` command line interface

use std::{fmt::Display, io};

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use config::Config;
use eyre::Context;

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

/// Compat trait to interop between eyre and sequoia v1.x (anyhow) errors
pub(crate) trait ToEyreError<T> {
	/// Convert to eyre error
	fn to_eyre(self) -> eyre::Result<T>;

	/// Convert to eyre error and wrap with a message
	#[inline]
	fn to_wrap_err<D>(self, msg: D) -> eyre::Result<T>
	where
		Self: Sized,
		D: Display + Send + Sync + 'static,
	{
		self.to_eyre().wrap_err(msg)
	}
}

impl<T> ToEyreError<T> for sequoia_openpgp::Result<T> {
	fn to_eyre(self) -> eyre::Result<T> {
		self.map_err(|err| eyre::eyre!(Box::new(err)))
	}
}
