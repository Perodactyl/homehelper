use std::io;

use super::prelude::*;
use hyprctl::{Monitor, Workspace};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EwwWorkspace {
	//Used for display
	index: Option<String>,
	name: Option<String>,
	icon: Option<String>,

	//Used to highlight active workspace on each monitor
	active_on: Option<String>,

	//Changes behavior
	is_special: bool,
	//Workspace name with `special:` stripped, used in `hyprctl dispatch togglespecialworkspace`
	special_name: String,

	//used in `hyprctl dispatch workspace`
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
		} else if let Ok(parts) = serde_json::from_str::<[String; 3]>(&workspace.name) {
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

pub struct ListenEww;
impl ListenEww {
} impl HandleDaemon for ListenEww {
	fn daemon(d: &mut Daemon, s: UnixStream) -> Result<()> {
		d.steps.push(Box::new(ListenEwwStep { socket: s }));
		Ok(())
	}
} impl HandleRemote for ListenEww {
	fn remote(mut s: UnixStream) -> Result<()> {
		loop {
			let update: ListenEwwMessage = recv!(s)?;
			match update {
				Ok(u) => println!("{}", serde_json::to_string(&u)?),
				Err(e) => bail!(e),
			}
		}
	}
}

type ListenEwwMessage = Result<Vec<EwwWorkspace>, String>;

#[derive(Debug)]
struct ListenEwwStep {
	socket: UnixStream
} impl ListenEwwStep {
	fn send_update(&mut self) -> Result<()> {
		let monitors = hyprctl::monitors()?;
		let update: Vec<EwwWorkspace> = hyprctl::workspaces()?.into_iter()
			.map(|w| EwwWorkspace::new(&monitors, w))
			.collect();

		send!(self.socket, ListenEwwMessage::Ok(update))?;
		Ok(())
	}
}

impl MainLoopStep for ListenEwwStep {
	fn on_event(&mut self, event: &Event) -> Result<StepState> {
		if let
			Event::FocusedMon { .. } |
			Event::Workspace { .. } |
			Event::CreateWorkspace { .. } |
			Event::DestroyWorkspace { .. } |
			Event::RenameWorkspace { .. } |
			Event::ActiveSpecial { .. } = event
		{
			if let Err(e) = self.send_update() &&
				let Some(e) = e.downcast_ref::<std::io::Error>() &&
				e.kind() == io::ErrorKind::BrokenPipe
			{
				return Ok(StepState::Done);
			}
		}

		Ok(StepState::KeepActive)
	}
	fn on_error(&mut self, error: anyhow::Error) -> Result<StepState> {
		send!(self.socket, ListenEwwMessage::Err(format!("{error:#}")))?;

		Ok(StepState::Done)
	}
}
