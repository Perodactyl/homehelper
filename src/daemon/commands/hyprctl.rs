use super::prelude::*;



pub struct Workspaces;
impl HandleDaemon for Workspaces {
	fn daemon(_: &mut Daemon, mut s: UnixStream) -> Result<()> {
		send!(s, hyprctl::workspaces()?)?;
		Ok(())
	}
}
impl HandleRemote for Workspaces {
	fn remote(mut s: std::os::unix::net::UnixStream) -> Result<()> {
		let workspaces: Vec<hyprctl::Workspace> = recv!(s)?;
		println!("{}", serde_json::to_string(&workspaces)?);
		Ok(())
	}
}



pub struct Monitors;
impl HandleDaemon for Monitors {
	fn daemon(_: &mut Daemon, mut s: UnixStream) -> Result<()> {
	    send!(s, hyprctl::monitors()?)?;
		Ok(())
	}
}
impl HandleRemote for Monitors {
	fn remote(mut s: UnixStream) -> Result<()> {
	    let monitors: Vec<hyprctl::Monitor> = recv!(s)?;
		println!("{}", serde_json::to_string(&monitors)?);
		Ok(())
	}
}
