use serde::Deserialize;
use super::send_command;
use anyhow::Result;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
	pub id: i32,
	pub name: String,
	pub monitor: String,
	#[serde(rename = "monitorID")]
	pub monitor_id: u32,
	pub windows: u32,
	#[serde(rename = "hasfullscreen")]
	pub has_fullscreen: bool,

	#[serde(rename = "lastwindow")]
	pub last_window: String,
	#[serde(rename = "lastwindowtitle")]
	pub last_window_title: String,
	#[serde(rename = "ispersistent")]
	pub is_persistent: bool,
}

#[allow(unused)]
pub fn workspaces() -> Result<Vec<Workspace>> {
	Ok(serde_json::from_str(&send_command(b"j/workspaces")?)?)
}
