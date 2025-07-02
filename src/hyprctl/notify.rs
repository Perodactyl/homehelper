#![allow(clippy::many_single_char_names)] // f (formatter) and rgba (in colors) are understood

use super::{expect_ok, send_command};
use anyhow::Result;
use std::fmt::Display;
use std::time::Duration;

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum NotifyIcon {
    None = -1,
    Warning = 0,
    Info = 1,
    Hint = 2,
    Error = 3,
    Confused = 4,
    Ok = 5,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Rgb(r, g, b) => write!(f, "rgb({r},{g},{b})"),
            Color::Rgba(r, g, b, a) => write!(f, "rgba({r},{g},{b},{a})"),
        }
    }
}

pub fn notify(icon: NotifyIcon, time: Duration, color: Color, message: &str) -> Result<()> {
    expect_ok(&send_command(
        format!(
            "/notify {} {} {color} {message}",
            icon as i8,
            time.as_millis()
        )
        .as_bytes(),
    )?)
}
