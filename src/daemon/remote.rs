use std::os::unix::net::UnixStream;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::hyprctl::workspaces::Workspace;

use super::DAEMON_SOCKET;

macro_rules! impl_request {
	{
		$(fn $name:ident -> $out:ty:
			$rq:expr => $key:ident in $rs:pat),+
	} => {
		$(pub fn $name(&mut self) -> Result<$out> {
			#[allow(unreachable_patterns)]
			match self.send_and_get_response(&$rq)? {
				$rs => Ok($key),
				_ => panic!("Response was not of expected type"),
			}
		})+
    };
}

///Connection to a remote daemon
pub struct Remote {
	socket: UnixStream
} impl Remote {
	pub fn new() -> Result<Self> {
		Ok(Remote {
			socket: UnixStream::connect(&*DAEMON_SOCKET)?,
		})
	}
	fn send_and_get_response(&mut self, request: &RemoteRequest) -> Result<RemoteResponse> {
		ciborium::into_writer(request, &mut self.socket)?;
		Ok(ciborium::from_reader(&mut self.socket)?)
	}
	impl_request! {
		fn workspaces -> Vec<Workspace>:
			RemoteRequest::Workspaces => out in RemoteResponse::Workspaces(out)
	}
}

pub fn launch(arguments: Arguments) -> Result<()> {
	match arguments.mode {
		RunMode::Workspaces => println!("{}", serde_json::to_string(&Remote::new()?.workspaces()?)?)
	}
	Ok(())
}

#[derive(Debug, Clone, clap::Args)]
pub struct Arguments {
	#[arg(value_enum)]
	mode: RunMode
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum RunMode {
	Workspaces,
}

///Message sent to the daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteRequest {
	Workspaces
}

///Message received from the daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteResponse {
	Workspaces(Vec<Workspace>)
}

