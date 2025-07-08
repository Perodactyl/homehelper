use std::os::unix::net::UnixStream;
use anyhow::Result;
use crate::daemon::Daemon;

pub mod prelude {
	#![allow(unused_imports)]

	pub use serde::{Deserialize, Serialize};
	pub use anyhow::{anyhow, bail, Result};

	pub use crate::hyprctl::prelude::*;
	pub use crate::{send, recv};
	pub use crate::daemon::Daemon;

	pub use super::{HandleDaemon, HandleRemote};
	pub use crate::daemon::{MainLoopStep, StepState};
	pub use std::os::unix::net::{UnixStream, UnixListener};
}

mod hyprctl;
mod listen_eww;

pub mod implementors {
	pub use super::hyprctl::*;
	pub use super::listen_eww::*;
}

#[macro_export]
macro_rules! send {
    ($socket:expr, $data:expr) => {
        ciborium::into_writer(&$data, &mut $socket)
    };
}

#[macro_export]
macro_rules! recv {
    ($socket:expr) => {
        ciborium::from_reader(&mut $socket)
    };
}

pub trait HandleRemote {
	fn remote(s: UnixStream) -> Result<()>;
}

pub trait HandleDaemon {
	fn daemon(d: &mut Daemon, s: UnixStream) -> Result<()>;
}

macro_rules! command_enum {
	{ $($name:ident),* $(,)* } => {
		#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, clap::ValueEnum)]
		pub enum Command {
			$($name),*
		}
		impl Command {
			pub fn dispatch_remote(self, socket: UnixStream) -> Result<()> {
				match self {
					$(Command::$name => <$name as HandleRemote>::remote(socket)),*
				}
			}
			pub fn dispatch_daemon(self, daemon: &mut Daemon, socket: UnixStream) -> Result<()> {
				match self {
					$(Command::$name => <$name as HandleDaemon>::daemon(daemon, socket)),*
				}
			}
		}
	}
}

#[allow(clippy::wildcard_imports)]
use implementors::*;

command_enum! {
	Workspaces,
	Monitors,
	ListenEww,
}
