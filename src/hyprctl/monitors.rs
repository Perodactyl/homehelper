use super::send_command;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorWorkspace {
	pub id: i32,
	pub name: String,
}


#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorReserved {
	pub top: u32,
	pub left: u32,
	pub bottom: u32,
	pub right: u32,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum Transform {
	Normal        = 0,
	Rotate90      = 1,
	Rotate180     = 2,
	Rotate270     = 3,
	Flip          = 4,
	FlipRotate90  = 5,
	FlipRotate180 = 6,
	FlipRotate270 = 7,
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MonitorInternal {
    id: i32,
    name: String,
	description: String,
	make: String,
	model: String,
	serial: String,
	width: u32,
	height: u32,
	active_workspace: MonitorWorkspace,
	special_workspace: MonitorWorkspace,
	reserved: MonitorReserved,
	scale: f32,
	transform: Transform,
	focused: bool,
	dpms_status: bool,
	#[serde(rename = "vrr")]
	variable_refresh_rate: bool,
	solitary: String,
	actively_tearing: bool,
	direct_scanout_to: String,
	disabled: bool,
	current_format: String,
	mirror_of: String,
	available_modes: Vec<String>,
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: i32,
    pub name: String,
	pub description: String,
	pub make: String,
	pub model: String,
	pub serial: String,
	pub width: u32,
	pub height: u32,
	pub active_workspace: MonitorWorkspace,
	pub special_workspace: Option<MonitorWorkspace>,
	pub reserved: MonitorReserved,
	pub scale: f32,
	pub transform: Transform,
	pub focused: bool,
	pub dpms_status: bool,
	pub variable_refresh_rate: bool,
	pub actively_tearing: bool,
	pub direct_scanout_to: String,
	pub enabled: bool,
	pub current_format: String,
	pub mirror_of: Option<String>,
	pub available_modes: Vec<String>,
} impl From<MonitorInternal> for Monitor {
	fn from(value: MonitorInternal) -> Self {
	    Monitor {
			id: value.id,
			name: value.name,
			description: value.description,
			make: value.make,
			model: value.model,
			serial: value.serial,
			width: value.width,
			height: value.height,
			active_workspace: value.active_workspace,
			special_workspace: if value.special_workspace.name.is_empty() { None } else { Some(value.special_workspace) },
			reserved: value.reserved,
			scale: value.scale,
			transform: value.transform,
			focused: value.focused,
			dpms_status: value.dpms_status,
			variable_refresh_rate: value.variable_refresh_rate,
			actively_tearing: value.actively_tearing,
			direct_scanout_to: value.direct_scanout_to,
			enabled: !value.disabled,
			current_format: value.current_format,
			mirror_of: if value.mirror_of.is_empty() { None } else { Some(value.mirror_of) },
			available_modes: value.available_modes,
		}
	}
}

#[allow(unused)]
pub fn monitors() -> Result<Vec<Monitor>> {
    Ok(
		serde_json::from_str::<Vec<MonitorInternal>>(&send_command(b"j/monitors")?)?.into_iter()
			.map(<Monitor as From<MonitorInternal>>::from)
			.collect()
	)
}
