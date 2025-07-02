use std::io::{Read, Write};
use std::os::unix;
use std::sync::LazyLock;
use unix::net::UnixStream;

use anyhow::{bail, Result};

pub mod notify;
pub use notify::*;
pub mod binds;
pub use binds::*;
pub mod socket2;
pub mod workspaces;
pub use socket2::*;
pub mod monitors;
pub use monitors::*;

pub static SOCKET1: LazyLock<String> = LazyLock::new(|| {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();
    let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    format!("{runtime}/hypr/{his}/.socket.sock")
});

pub static SOCKET2: LazyLock<String> = LazyLock::new(|| {
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

fn expect_ok(result: &str) -> Result<()> {
    if result == "ok" {
        Ok(())
    } else {
        bail!("Command returned: {result}");
    }
}

#[allow(unused)]
pub fn reload() -> Result<()> {
	expect_ok(&send_command(b"/reload")?)
}
