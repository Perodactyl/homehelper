use super::{send_command, expect_ok};
use std::fmt::Display;
use anyhow::Result;
use std::time::Duration;

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum NotifyIcon {
	NONE = -1,
	WARNING = 0,
	INFO = 1,
	HINT = 2,
	ERROR = 3,
	CONFUSED = 4,
	OK = 5,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
	RGB(u8, u8, u8),
	RGBA(u8, u8, u8, u8),
} impl Display for Color {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	    match self {
			Color::RGB(r, g, b) => write!(f, "rgb({r},{g},{b})"),
			Color::RGBA(r, g, b, a) => write!(f, "rgba({r},{g},{b},{a})")
		}
	}
}

pub fn notify(icon: NotifyIcon, time: Duration, color: Color, message: &str) -> Result<()> {
	expect_ok(send_command(format!("/notify {} {} {color} {message}", icon as i8, time.as_millis()).as_bytes())?)
}

