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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EwwWorkspace {
	index: Option<String>,
	name: Option<String>,
	icon: Option<String>,
	active_on: Option<String>,
	is_special: bool,
	special_name: String,
	id: i32,
} impl EwwWorkspace {
	pub fn new(monitors: &Vec<Monitor>, workspace: Workspace) -> Self {
		let mut active_on = None;
		for monitor in monitors {
			if monitor.active_workspace.id == workspace.id {
				active_on = Some(monitor.name.clone());
				break;
			} else if let Some(special) = &monitor.special_workspace && special.id == workspace.id {
				active_on = Some(monitor.name.clone());
				break;
			}
		}
		let is_special = workspace.id < 0;

		if is_special {
			EwwWorkspace {
				active_on,
				is_special,
				index: None,
				icon: Some(match &workspace.name[..] {
					"special:guide" => "󰈹",
					"special:term" => "",
					"special:other" => "",
					"special:music" => "",
					"special:notes" => "",
					"special:testing" => "",
					n => n,
				}.to_string()),
				name: None,
				id: workspace.id,
				special_name: if let Some((_, right)) = workspace.name.split_once(':') {
					right.to_string()
				} else {
					workspace.name
				}
			}
		} else {
			if let Ok(parts) = serde_json::from_str::<[String; 3]>(&workspace.name) {
				let [index, icon, name] = parts;
				EwwWorkspace {
					active_on,
					index: if index.is_empty() {
						None
					} else {
						Some(if index == "#" { workspace.id.to_string() } else { index })
					},
					icon: if icon.is_empty() { None } else { Some(icon) },
					name: if name.is_empty() { None } else { Some(name) },
					is_special,
					id: workspace.id,
					special_name: workspace.name,
				}
			} else {
				EwwWorkspace {
					active_on,
					index: Some(workspace.name.clone()),
					icon: None,
					name: None,
					is_special,
					id: workspace.id,
					special_name: workspace.name,
				}
			}
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EwwInfo(pub Vec<EwwWorkspace>);

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
		RunMode::ListenEww => {
			let mut r = Remote::new()?;
			ciborium::into_writer(&RemoteRequest::ListenEww, &mut r.socket)?;
			loop {
				let update: EwwInfo = ciborium::from_reader(&mut r.socket)?;
				println!("{}", serde_json::to_string(&update)?);
			}
		}
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
	ListenEww,
}

///Message sent to the daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteRequest {
    Workspaces,
	Monitors,
	ListenEww,
}

///Message received from the daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteResponse {
	Error(String),
    Workspaces(Vec<Workspace>),
	Monitors(Vec<Monitor>),
}
