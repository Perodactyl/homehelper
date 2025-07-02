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
use remote::{EwwInfo, RemoteRequest, RemoteResponse};

mod submap;
use submap::show_binds_in_submap;

pub mod remote;

#[derive(Debug, Clone, clap::Args)]
pub struct Arguments {
    #[arg(short, long, action = clap::ArgAction::Set, default_value_t = true)]
    submap: bool,
}

static DAEMON_SOCKET: LazyLock<String> = LazyLock::new(|| Daemon::socket_path().unwrap());

enum StepStatusUpdate {
	KeepActive,
	Done,
}

///A temporary, non-blocking stage in the main loop, often used for certain client requests.
trait MainLoopStep: std::fmt::Debug {
	fn step(&mut self) -> Result<StepStatusUpdate> {
		Ok(StepStatusUpdate::KeepActive)
	}
	fn on_event(&mut self, event: &Event) -> Result<()> {
		Ok(())
	}
	///Used to catch errors, potentially to relay them to connected clients. Returns an Error if
	///the main loop must break its processing for one iteration. If it returns an Error, this
	///entry will also be removed.
	fn on_error(&mut self, error: anyhow::Error) -> Result<StepStatusUpdate> {
		Err(error)
	}
}

#[derive(Debug)]
struct EwwInfoStep {
	socket: UnixStream
} impl EwwInfoStep {
	fn send_update(&mut self) -> Result<()> {
		let monitors = hyprctl::monitors()?;
		let update: EwwInfo = EwwInfo(hyprctl::workspaces()?.into_iter()
			.map(|w| remote::EwwWorkspace::new(&monitors, w))
			.collect()
		);
		ciborium::into_writer(&update, &mut self.socket)?;
		
		Ok(())
	}
} impl MainLoopStep for EwwInfoStep {
	fn on_event(&mut self, event: &Event) -> Result<()> {
		match event {
			Event::FocusedMon { .. } |
			Event::Workspace { .. } |
			Event::CreateWorkspace { .. } |
			Event::DestroyWorkspace { .. } |
			Event::RenameWorkspace { .. } |
			Event::ActiveSpecial { .. } => {
				self.send_update()?;
			},
			_ => {},
		}

		Ok(())
	}
	fn on_error(&mut self, error: anyhow::Error) -> Result<StepStatusUpdate> {
		let _ = ciborium::into_writer(&RemoteResponse::Error(format!("{error:#}")), &mut self.socket);
		Ok(StepStatusUpdate::Done)
	}
}

#[derive(Debug)]
struct SubmapContentEntry {
    panel: Option<Child>,
}
impl MainLoopStep for SubmapContentEntry {
	fn on_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Submap { name } => {
                if let Some(name) = name {
                    if let Some(mut child) = self.panel.take() {
                        let _ = child.kill();
                    }
                    self.panel = Some(show_binds_in_submap(&name)?);
                } else if let Some(mut child) = self.panel.take() {
                    let _ = child.kill();
                }
            },
            _ => {}
        }

		Ok(())
	}
}

#[derive(Debug)]
pub struct Daemon {
    options: Arguments,
    socket2: hyprctl::Socket2,
    home_helper_socket: UnixListener,
	entries: Vec<Box<dyn MainLoopStep>>
}
impl Daemon {
    fn new(options: Arguments) -> Result<Self> {
        let addr = &*DAEMON_SOCKET;
        println!("Opening socket at {addr}");
        let socket = UnixListener::bind(addr)?;
        socket.set_nonblocking(true)?;

        let socket2 = hyprctl::Socket2::new()?;
		let mut entries: Vec<Box<dyn MainLoopStep>> = vec![];
		if options.submap {
			entries.push(Box::new(SubmapContentEntry { panel: None }));
		}
        Ok(Daemon {
            options,
            socket2,
            home_helper_socket: socket,
			entries,
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
		let mut i = 0;
		while i < self.entries.len() {
			let entry = &mut self.entries[i];
			let mut should_increment = true;
			match entry.step() {
				Ok(StepStatusUpdate::Done) => {
					self.entries.remove(i);
					should_increment = false;
				},
				Ok(StepStatusUpdate::KeepActive) => {},
				Err(e) => {
					match entry.on_error(e) {
						Ok(StepStatusUpdate::KeepActive) => {},
						Ok(StepStatusUpdate::Done) => {
							self.entries.remove(i);
							should_increment = false;
						}
						Err(e) => {
							self.entries.remove(i);
							return Err(e);
						}
					}
				}
			}
			if should_increment {
				i += 1;
			}
		}

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
			RemoteRequest::Workspaces => RemoteResponse::Workspaces(hyprctl::workspaces()?),
			RemoteRequest::Monitors => RemoteResponse::Monitors(hyprctl::monitors()?),
			_ => unimplemented!(),
		})
	}

	fn listener_handle_socket(&mut self, mut socket: UnixStream) -> Result<()> {
		let request: RemoteRequest = ciborium::from_reader(&mut socket)?;
		match request {
			RemoteRequest::ListenEww => {
				socket.set_nonblocking(true)?;
				let mut step = EwwInfoStep { socket };
				step.send_update()?;
				self.entries.push(Box::new(step));
			},
			r => match self.listener_handle_request(r) {
				Ok(response) => ciborium::into_writer(&response, &mut socket)?,
				Err(e) => {
					log_error(&e);
					ciborium::into_writer(&RemoteResponse::Error(format!("{e:#}")), &mut socket)?;
				},
			}
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
		let event = event?;
		let mut i = 0;
		while i < self.entries.len() {
			let entry = &mut self.entries[i];
			let mut should_increment = true;
			match entry.on_event(&event) {
				Ok(()) => {},
				Err(e) => match entry.on_error(e) {
					Ok(StepStatusUpdate::KeepActive) => {},
					Ok(StepStatusUpdate::Done) => {
						self.entries.remove(i);
						should_increment = false;
					}
					Err(e) => {
						self.entries.remove(i);
						return Err(e);
					}
				}
			}
			if should_increment {
				i += 1;
			}
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
