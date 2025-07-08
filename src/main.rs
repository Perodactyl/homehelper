#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Error, Result};
use clap::Parser;

mod daemon;
pub mod hyprctl; //pub silences an error in daemon::commands::prelude

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

pub fn log_error(error: &Error) {
	use hyprctl::{NotifyIcon,Color};
	use std::time::Duration;

	eprintln!("{error:#}");
	let _ = hyprctl::notify(
		NotifyIcon::Error,
		Duration::from_secs(5),
		Color::Rgb(255, 0, 0),
		&format!("HomeHelper Error: {error}"),
	);
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Daemon(args) => {
            daemon::Daemon::launch(args)?;
        }
        Command::Remote(args) => {
            daemon::remote::launch(args)?;
        }
    }
    Ok(())
}
