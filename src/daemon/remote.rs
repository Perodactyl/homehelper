use std::{io, os::unix::net::UnixStream};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::hyprctl::{workspaces::Workspace, Monitor};

use super::DAEMON_SOCKET;

macro_rules! impl_request {
	{
		$(fn $name:ident -> $out:ty:
			$rq:expr => $key:ident in $rs:pat),+
	} => {
		$(pub fn $name(&mut self) -> Result<$out> {
			#[allow(unreachable_patterns)]
			match self.send_and_get_response(&$rq)? {
				RemoteResponse::Error(e) => Err(anyhow!(e)),
				$rs => Ok($key),
				_ => panic!("Response was not of expected type"),
			}
		})+
    };
}

///Connection to a remote daemon
pub struct Remote {
    socket: UnixStream,
}
impl Remote {
    pub fn new() -> Result<Self> {
		let socket = match UnixStream::connect(&*DAEMON_SOCKET) {
			Ok(s) => s,
			Err(e) if e.kind() == io::ErrorKind::NotFound => {
				bail!("Socket file not found. Is the daemon running?");
			},
			e => e?,
		};
        Ok(Remote {
            socket,
		})
    }
    fn send_and_get_response(&mut self, request: &RemoteRequest) -> Result<RemoteResponse> {
        ciborium::into_writer(request, &mut self.socket)?;
        Ok(ciborium::from_reader(&mut self.socket)?)
    }
    impl_request! {
        fn workspaces -> Vec<Workspace>:
            RemoteRequest::Workspaces => out in RemoteResponse::Workspaces(out),
		fn monitors -> Vec<Monitor>:
			RemoteRequest::Monitors => out in RemoteResponse::Monitors(out)
    }
}

pub fn launch(arguments: Arguments) -> Result<()> {
    match arguments.mode {
        RunMode::Workspaces => {
            println!("{}", serde_json::to_string(&Remote::new()?.workspaces()?)?)
        },
        RunMode::Monitors => {
            println!("{}", serde_json::to_string(&Remote::new()?.monitors()?)?)
        },
    }
    Ok(())
}

#[derive(Debug, Clone, clap::Args)]
pub struct Arguments {
    #[arg(value_enum)]
    mode: RunMode,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum RunMode {
    Workspaces,
	Monitors,
}

///Message sent to the daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteRequest {
    Workspaces,
	Monitors,
}

///Message received from the daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteResponse {
	Error(String),
    Workspaces(Vec<Workspace>),
	Monitors(Vec<Monitor>),
}
