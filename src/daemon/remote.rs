use super::DAEMON_SOCKET;
use anyhow::{bail, Result};
use std::{io, os::unix::net::UnixStream};

fn connect() -> Result<UnixStream> {
	match UnixStream::connect(&*DAEMON_SOCKET) {
		Ok(s) => Ok(s),
		Err(e) if e.kind() == io::ErrorKind::NotFound => {
			bail!("Socket file not found. Is the daemon running?");
		},
		Err(e) => bail!(e),
	}
}

pub fn launch(arguments: Arguments) -> Result<()> {
	let mut socket = connect()?;
	ciborium::into_writer(&arguments.command, &mut socket)?;
	arguments.command.dispatch_remote(socket)?;
    Ok(())
}

#[derive(Debug, Clone, clap::Args)]
pub struct Arguments {
    #[arg(value_enum)]
    command: super::commands::Command,
}
