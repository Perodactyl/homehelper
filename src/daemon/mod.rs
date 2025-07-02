use std::{
    os::unix::net::{UnixListener, UnixStream},
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock,
    },
};

use crate::{hyprctl::{self, Event}, log_error};
use anyhow::Result;
use remote::{RemoteRequest, RemoteResponse};

mod submap;
use submap::show_binds_in_submap;

pub mod remote;

#[derive(Debug, Clone, clap::Args)]
pub struct Arguments {
    #[arg(short, long, action = clap::ArgAction::Set, default_value_t = true)]
    submap: bool,
}

static DAEMON_SOCKET: LazyLock<String> = LazyLock::new(|| Daemon::socket_path().unwrap());

#[derive(Debug)]
pub struct Daemon {
    panel: Option<Child>,
    options: Arguments,
    socket2: hyprctl::Socket2,
    home_helper_socket: UnixListener,
}
impl Daemon {
    fn new(options: Arguments) -> Result<Self> {
        let addr = &*DAEMON_SOCKET;
        println!("Opening socket at {addr}");
        let socket = UnixListener::bind(addr)?;
        socket.set_nonblocking(true)?;

        let socket2 = hyprctl::Socket2::new()?;
        Ok(Daemon {
            panel: None,
            options,
            socket2,
            home_helper_socket: socket,
        })
    }
    pub fn socket_path() -> Result<String> {
        let runtime = std::env::var("XDG_RUNTIME_DIR")?;
        Ok(format!("{runtime}/homehelper.sock"))
    }

    pub fn launch(options: Arguments) -> Result<()> {
        let mut d = Self::new(options)?;

        let must_exit = Arc::new(AtomicBool::new(false));
        let thread_must_exit = Arc::clone(&must_exit);

        ctrlc::set_handler(move || {
            thread_must_exit.store(true, Ordering::Relaxed);
        })?;

        loop {
            d.step()?;
            if must_exit.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        self.hyprctl_step()?;
        self.listener_step()?;

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn hyprctl_step(&mut self) -> Result<()> {
        let events: Vec<Result<Event>> = (&mut self.socket2).collect();

        for event in events {
            match self.handle_event(event) {
                Ok(()) => {}
                Err(e) => log_error(&e),
            }
        }

        Ok(())
    }

	fn listener_handle_request(&mut self, request: RemoteRequest) -> Result<RemoteResponse> {
		Ok(match request {
			RemoteRequest::Workspaces => RemoteResponse::Workspaces(hyprctl::workspaces::workspaces()?),
			RemoteRequest::Monitors => RemoteResponse::Monitors(hyprctl::monitors()?),
		})
	}

	fn listener_handle_socket(&mut self, mut socket: UnixStream) -> Result<()> {
		let request: RemoteRequest = ciborium::from_reader(&mut socket)?;
		match self.listener_handle_request(request) {
			Ok(response) => ciborium::into_writer(&response, &mut socket)?,
			Err(e) => {
				log_error(&e);
				ciborium::into_writer(&RemoteResponse::Error(format!("{e:#}")), &mut socket)?
			},
		}

		Ok(())
	}

    fn listener_step(&mut self) -> Result<()> {
        match self.home_helper_socket.accept() {
            Ok((s, _)) => {
				match self.listener_handle_socket(s) {
					Ok(()) => {},
					Err(e) => log_error(&e),
				}
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => Err(e)?,
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Result<Event>) -> Result<()> {
        match event? {
            Event::Submap { name } if self.options.submap => {
                if let Some(name) = name {
                    if let Some(mut child) = self.panel.take() {
                        let _ = child.kill();
                    }
                    self.panel = Some(show_binds_in_submap(&name)?);
                } else if let Some(mut child) = self.panel.take() {
                    let _ = child.kill();
                }
            }
            _ => {}
        }

        Ok(())
    }
}
impl Drop for Daemon {
    fn drop(&mut self) {
        let addr = &*DAEMON_SOCKET;
        println!("Closing socket at {addr}");
        let _ = std::fs::remove_file(addr);
    }
}
