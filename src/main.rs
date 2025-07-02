#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use clap::Parser;

mod hyprctl;
mod daemon;

#[derive(Debug, Clone, clap::Parser)]
#[command(version, about, propagate_version = true)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Command {
	Daemon(daemon::Arguments),
	Remote(daemon::remote::Arguments),
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	match cli.command {
		Command::Daemon(args) => {
			daemon::Daemon::launch(args)?;
		},
		Command::Remote(args) => {
			daemon::remote::launch(args)?;
		}
	}
	Ok(())
}
