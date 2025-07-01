use serde::Deserialize;
use super::send_command;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
struct BindInternal {
	locked: bool,
	mouse: bool,
	release: bool,
	repeat: bool,
	#[serde(rename = "longPress")]
	long_press: bool,
	non_consuming: bool,
	has_description: bool,
	modmask: u32,
	submap: String,
	key: String,
	keycode: u32,
	catch_all: bool,
	description: String,
	dispatcher: String,
	arg: String,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Bind {
	pub locked: bool,
	pub mouse: bool,
	pub release: bool,
	pub repeat: bool,
	pub long_press: bool,
	pub non_consuming: bool,
	pub catch_all: bool,
	pub modmask: u32,
	pub submap: Option<String>,
	pub key: String,
	pub keycode: u32,
	pub description: Option<String>,
	pub action: (String, String),
} impl From<BindInternal> for Bind {
	fn from(value: BindInternal) -> Self {
	    Bind {
			locked: value.locked,
			mouse: value.mouse,
			release: value.release,
			repeat: value.repeat,
			long_press: value.long_press,
			non_consuming: value.non_consuming,
			catch_all: value.catch_all,
			modmask: value.modmask,
			submap: if value.submap.len() > 0 { Some(value.submap) } else { None },
			key: value.key,
			keycode: value.keycode,
			description: if value.has_description { Some(value.description) } else { None },
			action: (value.dispatcher, value.arg),
		}
	}
}

pub fn binds() -> Result<Vec<Bind>> {
	Ok(
		serde_json::from_str::<Vec<BindInternal>>(&send_command(b"j/binds")?)?.into_iter()
			.map(|v| <BindInternal as Into<Bind>>::into(v))
			.collect()
	)
}
