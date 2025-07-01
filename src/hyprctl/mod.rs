use std::io::{Read, Write};
use std::os::unix;
use unix::net::UnixStream;

use anyhow::{bail, Result};
use once_cell::sync::Lazy;

pub mod notify;
pub use notify::*;
pub mod binds;
pub use binds::*;
pub mod workspaces;
pub mod socket2;
pub use socket2::*;


pub static SOCKET1: Lazy<String> = Lazy::new(|| {
	let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();
	let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
	format!("{runtime}/hypr/{his}/.socket.sock")
});

pub static SOCKET2: Lazy<String> = Lazy::new(|| {
	let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();
	let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
	format!("{runtime}/hypr/{his}/.socket2.sock")
});

fn send_command(command: &[u8]) -> Result<String> {
	let mut sock = UnixStream::connect(&*SOCKET1)?;
	sock.write_all(command)?;
	let mut result = vec![];
	sock.read_to_end(&mut result)?;
	let result_str = String::from_utf8(result)?;

	Ok(result_str)
}

pub fn expect_ok(result: String) -> Result<()> {
	if result == "ok" {
		Ok(())
	} else {
		bail!("Command returned: {result}");
	}
}


