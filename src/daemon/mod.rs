use std::{
    os::unix::net::{UnixListener, UnixStream},
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock,
    }, time::{Duration, Instant},
};

use crate::{hyprctl::{self, Event}, log_error};
use anyhow::Result;

mod submap;
use submap::show_binds_in_submap;

pub mod remote;
pub mod commands;

#[derive(Debug, Clone, clap::Args)]
pub struct Arguments {
    #[arg(short, long, action = clap::ArgAction::Set, default_value_t = true)]
    submap: bool,
}

static DAEMON_SOCKET: LazyLock<String> = LazyLock::new(|| Daemon::socket_path().unwrap());

pub enum StepState {
	KeepActive,
	Done,
}

///A temporary, non-blocking stage in the main loop, often used for certain client requests.
pub trait MainLoopStep: std::fmt::Debug {
	fn step(&mut self) -> Result<StepState> {
		Ok(StepState::KeepActive)
	}
	#[allow(unused_variables)]
	fn on_event(&mut self, event: &Event) -> Result<StepState> {
		Ok(StepState::KeepActive)
	}
	///Used to catch errors, potentially to relay them to connected clients. Returns an Error if
	///the main loop must break its processing for one iteration. If it returns an Error, this
	///entry will also be removed.
	fn on_error(&mut self, error: anyhow::Error) -> Result<StepState> {
		Err(error)
	}
}

#[derive(Debug)]
struct SubmapContentEntry {
    panel: Option<Child>,
} impl MainLoopStep for SubmapContentEntry {
	fn on_event(&mut self, event: &Event) -> Result<StepState> {
        if let Event::Submap { name } = event {
			if let Some(name) = name {
				if let Some(mut child) = self.panel.take() {
					let _ = child.kill();
				}
				self.panel = Some(show_binds_in_submap(name)?);
			} else if let Some(mut child) = self.panel.take() {
				let _ = child.kill();
			}
		}

		Ok(StepState::KeepActive)
	}
}

#[derive(Debug)]
pub struct Daemon {
    options: Arguments,
    socket2: hyprctl::Socket2,
    home_helper_socket: UnixListener,
	pub steps: Vec<Box<dyn MainLoopStep>>
} impl Daemon {
    fn new(options: Arguments) -> Result<Self> {
        let addr = &*DAEMON_SOCKET;
        println!("Opening socket at {addr}");
        let socket = UnixListener::bind(addr)?;
        socket.set_nonblocking(true)?;

        let socket2 = hyprctl::Socket2::new()?;
		let steps: Vec<Box<dyn MainLoopStep>> = vec![];
        Ok(Daemon {
            options,
            socket2,
            home_helper_socket: socket,
			steps,
        })
    }
    pub fn socket_path() -> Result<String> {
        let runtime = std::env::var("XDG_RUNTIME_DIR")?;
        Ok(format!("{runtime}/homehelper.sock"))
    }

    pub fn launch(options: Arguments) -> Result<()> {
        let mut d = Self::new(options)?;
		if d.options.submap {
			d.steps.push(Box::new(SubmapContentEntry { panel: None }));
		}

        let must_exit = Arc::new(AtomicBool::new(false));
        let thread_must_exit = Arc::clone(&must_exit);

        ctrlc::set_handler(move || {
            thread_must_exit.store(true, Ordering::Relaxed);
        })?;

		let mut last_step = Instant::now();
        loop {
            d.step()?;
            if must_exit.load(Ordering::Relaxed) {
                break;
            }
			let this_step = Instant::now();
			if this_step.duration_since(last_step) < Duration::from_millis(25) {
				std::thread::sleep(Duration::from_millis(25) - this_step.duration_since(last_step));
			}
			last_step = this_step;
        }

        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        self.hyprctl_step()?;
        self.listener_step()?;
		let mut i = 0;
		while i < self.steps.len() {
			let entry = &mut self.steps[i];
			let mut should_increment = true;
			match entry.step() {
				Ok(StepState::Done) => {
					self.steps.remove(i);
					should_increment = false;
				},
				Ok(StepState::KeepActive) => {},
				Err(e) => {
					match entry.on_error(e) {
						Ok(StepState::KeepActive) => {},
						Ok(StepState::Done) => {
							self.steps.remove(i);
							should_increment = false;
						}
						Err(e) => {
							self.steps.remove(i);
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

	fn listener_handle_socket(&mut self, mut socket: UnixStream) -> Result<()> {
		let request: commands::Command = ciborium::from_reader(&mut socket)?;
		request.dispatch_daemon(self, socket)?;

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
		while i < self.steps.len() {
			let entry = &mut self.steps[i];
			let mut should_increment = true;
			match entry.on_event(&event) {
				Ok(StepState::KeepActive) => {},
				Ok(StepState::Done) => {
					self.steps.remove(i);
					should_increment = false;
				},
				Err(e) => match entry.on_error(e) {
					Ok(StepState::KeepActive) => {},
					Ok(StepState::Done) => {
						self.steps.remove(i);
						should_increment = false;
					}
					Err(e) => {
						self.steps.remove(i);
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
} impl Drop for Daemon {
    fn drop(&mut self) {
        let addr = &*DAEMON_SOCKET;
        println!("Closing socket at {addr}");
        let _ = std::fs::remove_file(addr);
    }
}
