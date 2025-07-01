use std::{io::{BufRead, BufReader}, os::unix::net::UnixStream};

use anyhow::{bail, Result};

use super::SOCKET2;

type WindowAddress = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScreenCastOwner {
	Monitor,
	Window
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveWindow {
	pub class: String,
	pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSpecial {
	pub id: i32,
	pub name: String,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Event {
	Workspace { name: String },
	WorkspaceV2 { id: i32, name: String },
	FocusedMon { workspace_name: String, monitor_name: String },
	FocusedMonV2 { workspace_id: i32, monitor_name: String },
	ActiveWindow { window: Option<ActiveWindow> },
	ActiveWindowV2 { window_address: Option<WindowAddress> },
	Fullscreen { active: bool },
	MonitorRemoved { name: String },
	MonitorRemovedV2 { id: i32, name: String, description: String },
	MonitorAdded { name: String },
	MonitorAddedV2 { id: i32, name: String, description: String },
	CreateWorkspace { name: String },
	CreateWorkspaceV2 { id: i32, name: String },
	DestroyWorkspace { name: String },
	DestroyWorkspaceV2 { id: i32, name: String },
	MoveWorkspace { workspace_name: String, monitor_name: String },
	MoveWorkspaceV2 { workspace_id: i32, workspace_name: String, monitor_name: String },
	RenameWorkspace { id: i32, new_name: String },
	ActiveSpecial { workspace_name: Option<String>, monitor_name: String },
	ActiveSpecialV2 { workspace: Option<ActiveSpecial>, monitor_name: String },
	ActiveLayout { keyboard_name: String, layout_name: String },
	OpenWindow { window_address: WindowAddress, workspace_name: String, window_class: String, window_title: String },
	CloseWindow { window_address: WindowAddress },
	MoveWindow { window_address: WindowAddress, workspace_name: String },
	MoveWindowV2 { window_address: WindowAddress, workspace_id: i32, workspace_name: String },
	OpenLayer { namespace: String },
	CloseLayer { namespace: String },
	Submap { name: Option<String> },
	ChangeFloatingMode { window_address: WindowAddress, floating: bool },
	Urgent { window_address: WindowAddress },
	ScreenCast { active: bool, owner: ScreenCastOwner },
	WindowTitle { window_address: WindowAddress },
	WindowTitleV2 { window_address: WindowAddress, window_title: String },
	ToggleGroup { exists: bool, window_addresses: Vec<WindowAddress> },
	MoveIntoGroup { window_address: WindowAddress },
	MoveOutOfGroup { window_address: WindowAddress },
	IgnoreGroupLock { active: bool },
	LockGroups { locked: bool },
	ConfigReloaded,
	Pin { window_address: WindowAddress, pinned: bool },
	Minimized { window_address: WindowAddress, state: bool },
	Bell { window_address: WindowAddress },
	Custom { name: String, params: Vec<String> },
}

macro_rules! params {
    ($in:ident => $a:ident) => {
		let $a = $in;
    };
	//Second comma looks bad but prevents ambiguity (even though it really shouldn't be ambiguous
	//since $last is required but $p isn't)
	($in:ident => $a:ident,, $b:ident) => {
		let Some(($a, $b)) = $in.split_once(',') else { bail!("Not enough params") };
	};
	//Second comma looks bad but prevents ambiguity (even though it really shouldn't be ambiguous
	//since $last is required but $p isn't)
	($in:ident => $first:ident,, $($p:ident),*,, $last:ident) => {
		let ( $first, $($p),*, $last ) = {
			let Some(($first, params)) = $in.split_once(',') else { bail!("Not enough params") };
			$(
				let Some(($p, params)) = params.split_once(',') else { bail!("Not enough params") };
			)*
			let $last = params;
			( $first, $($p),*, $last )
		};
	};
}

pub fn poll_events() -> Result<impl Iterator<Item = Result<Event>>> {
	let stream = UnixStream::connect(&*SOCKET2)?;
	let buf_reader = BufReader::new(stream);
	Ok(buf_reader
		.lines()
		.map(|l| -> Result<Event> {
			let l = l?;

			let Some((event_name, params)) = l.split_once(">>") else { bail!("Could not find separator in {l}") };
			Ok(match event_name {
				"workspace" => Event::Workspace {
					name: String::from(params),
				},
				"workspacev2" => {
					params!(params => id,, name);
					Event::WorkspaceV2 {
						id: id.parse()?,
						name: String::from(name),
					}
				},
				"focusedmon" => {
					params!(params => monitor_name,, workspace_name);
					Event::FocusedMon {
						workspace_name: String::from(workspace_name),
						monitor_name: String::from(monitor_name),
					}
				},
				"focusedmonv2" => {
					params!(params => monitor_name,, workspace_id);
					Event::FocusedMonV2 {
						workspace_id: workspace_id.parse()?,
						monitor_name: String::from(monitor_name),
					}
				},
				"activewindow" => {
					params!(params => class,, title);
					if class.len() == 0 {
						Event::ActiveWindow {
							window: None
						}
					} else {
						Event::ActiveWindow {
							window: Some(ActiveWindow {
								class: String::from(class),
								title: String::from(title),
							}),
						}
					}
				},
				"activewindowv2" => {
					params!(params => window_address);
					if window_address.len() == 0 {
						Event::ActiveWindowV2 {
							window_address: None,
						}
					} else {
						Event::ActiveWindowV2 {
							window_address: Some(u64::from_str_radix(window_address, 16)?),
						}
					}
				},
				"fullscreen" => Event::Fullscreen {
					active: params == "1"
				},
				"monitorremoved" => Event::MonitorRemoved {
					name: String::from(params),
				},
				"monitorremovedv2" => {
					params!(params => monitor_id,, monitor_name,, monitor_desc);
					Event::MonitorRemovedV2 {
						id: monitor_id.parse()?,
						name: String::from(monitor_name),
						description: String::from(monitor_desc),
					}
				},
				"monitoradded" => Event::MonitorAdded {
					name: String::from(params),
				},
				"monitoraddedv2" => {
					params!(params => monitor_id,, monitor_name,, monitor_desc);
					Event::MonitorAddedV2 {
						id: monitor_id.parse()?,
						name: String::from(monitor_name),
						description: String::from(monitor_desc),
					}
				},
				"createworkspace" => Event::CreateWorkspace {
					name: String::from(params),
				},
				"createworkspacev2" => {
					params!(params => id,, name);
					Event::CreateWorkspaceV2 {
						id: id.parse()?,
						name: String::from(name),
					}
				},
				"destroyworkspace" => Event::DestroyWorkspace {
					name: String::from(params),
				},
				"destroyworkspacev2" => {
					params!(params => id,, name);
					Event::DestroyWorkspaceV2 {
						id: id.parse()?,
						name: String::from(name),
					}
				},
				"moveworkspace" => {
					params!(params => workspace_name,, monitor_name);
					Event::MoveWorkspace {
						workspace_name: String::from(workspace_name),
						monitor_name: String::from(monitor_name),
					}
				},
				"moveworkspacev2" => {
					params!(params => workspace_id,, workspace_name,, monitor_name);
					Event::MoveWorkspaceV2 {
						workspace_id: workspace_id.parse()?,
						workspace_name: String::from(workspace_name),
						monitor_name: String::from(monitor_name),
					}
				},
				"renameworkspace" => {
					params!(params => id,, new_name);
					Event::RenameWorkspace {
						id: id.parse()?,
						new_name: String::from(new_name),
					}
				},
				"activespecial" => {
					params!(params => workspace_name,, monitor_name);
					Event::ActiveSpecial {
						monitor_name: String::from(monitor_name),
						workspace_name: if workspace_name.len() > 0 { Some(String::from(workspace_name)) } else { None }
					}
				},
				"activespecialv2" => {
					params!(params => workspace_id,, workspace_name,, monitor_name);
					if workspace_id.len() == 0 {
						Event::ActiveSpecialV2 {
							workspace: None,
							monitor_name: String::from(monitor_name),
						}
					} else {
						Event::ActiveSpecialV2 {
							workspace: Some(ActiveSpecial {
								id: workspace_id.parse()?,
								name: String::from(workspace_name),
							}),
							monitor_name: String::from(monitor_name),
						}
					}
				},
				"activelayout" => {
					params!(params => keyboard_name,, layout_name);
					Event::ActiveLayout {
						keyboard_name: String::from(keyboard_name),
						layout_name: String::from(layout_name),
					}
				},
				"openwindow" => {
					params!(params => window_address,, workspace_name, window_class,, window_title);
					Event::OpenWindow {
						window_address: u64::from_str_radix(window_address, 16)?,
						workspace_name: String::from(workspace_name),
						window_class: String::from(window_class),
						window_title: String::from(window_title),
					}
				},
				"closewindow" => {
					params!(params => window_address);
					Event::CloseWindow {
						window_address: u64::from_str_radix(window_address, 16)?,
					}
				},
				"movewindow" => {
					params!(params => window_address,, workspace_name);
					Event::MoveWindow {
						window_address: u64::from_str_radix(window_address, 16)?,
						workspace_name: String::from(workspace_name),
					}
				},
				"movewindowv2" => {
					params!(params => window_address,, workspace_id,, workspace_name);
					Event::MoveWindowV2 {
						window_address: u64::from_str_radix(window_address, 16)?,
						workspace_id: workspace_id.parse()?,
						workspace_name: String::from(workspace_name),
					}
				},
				"openlayer" => {
					params!(params => namespace);
					Event::OpenLayer {
						namespace: String::from(namespace),
					}
				},
				"closelayer" => {
					params!(params => namespace);
					Event::CloseLayer {
						namespace: String::from(namespace),
					}
				},
				"submap" => Event::Submap {
					name: if params.len() > 0 { Some(String::from(params)) } else { None },
				},
				"changefloatingmode" => {
					params!(params => window_address,, floating);
					Event::ChangeFloatingMode {
						window_address: u64::from_str_radix(window_address, 16)?,
						floating: floating == "1",
					}
				},
				"urgent" => {
					params!(params => window_address);
					Event::Urgent {
						window_address: u64::from_str_radix(window_address, 16)?,
					}
				},
				"screencast" => {
					params!(params => state,, owner);
					Event::ScreenCast {
						active: state == "1",
						owner: if owner == "1" { ScreenCastOwner::Window } else { ScreenCastOwner::Monitor },
					}
				},
				"windowtitle" => {
					params!(params => window_address);
					Event::WindowTitle {
						window_address: u64::from_str_radix(window_address, 16)?,
					}
				},
				"windowtitlev2" => {
					params!(params => window_address,,window_title);
					Event::WindowTitleV2 {
						window_address: u64::from_str_radix(window_address, 16)?,
						window_title: String::from(window_title),
					}
				},
				"togglegroup" => {
					params!(params => state,, addresses);
					let mut output = vec![];
					for addr in addresses.split(',') {
						output.push(u64::from_str_radix(addr, 16)?);
					}
					Event::ToggleGroup {
						exists: state == "1",
						window_addresses: output,
					}
				},
				"moveintogroup" => {
					params!(params => window_address);
					Event::MoveIntoGroup {
						window_address: u64::from_str_radix(window_address, 16)?
					}
				},
				"moveoutofgroup" => {
					params!(params => window_address);
					Event::MoveOutOfGroup {
						window_address: u64::from_str_radix(window_address, 16)?
					}
				},
				"ignoregrouplock" => {
					params!(params => state);
					Event::IgnoreGroupLock {
						active: state == "1",
					}
				},
				"lockgroups" => {
					params!(params => state);
					Event::LockGroups {
						locked: state == "1",
					}
				},
				"configreloaded" => Event::ConfigReloaded,
				"pin" => {
					params!(params => window_address,, state);
					Event::Pin {
						window_address: u64::from_str_radix(window_address, 16)?,
						pinned: state == "1",
					}
				},
				"minimized" => {
					params!(params => window_address,, state);
					Event::Minimized {
						window_address: u64::from_str_radix(window_address, 16)?,
						state: state == "1",
					}
				},
				"bell" => {
					params!(params => window_address);
					Event::Bell {
						window_address: u64::from_str_radix(window_address, 16)?,
					}
				},
				name => Event::Custom {
					name: String::from(name),
					params: params.split(',').map(String::from).collect()
				}
			})

		})
	)
}
