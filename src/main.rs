use std::{process::Child, time::Duration};

use anyhow::Result;
use hyprctl::{notify, poll_events, Color, Event, NotifyIcon};
use submap::show_binds_in_submap;

mod hyprctl;
mod submap;

struct EventHandler {
	panel: Option<Child>,
} impl EventHandler {
	fn new() -> Self {
		EventHandler {
			panel: None
		}
	}
	fn run(&mut self) {
		for event in poll_events().unwrap() {
			match self.handle_event(event) {
				Ok(_) => {},
				Err(e) => {
					eprintln!("{e}");
					let _ = notify(
						NotifyIcon::ERROR,
						Duration::from_secs(5),
						Color::RGB(255, 0, 0),
						&format!("HomeHelper Error: {e}"),
					);
				}
			}
		}
	}
	fn handle_event(&mut self, event: Result<Event>) -> Result<()> {
		match event? {
			Event::Custom { .. } => {},
			Event::Submap { name } => {
				if let Some(name) = name {
					if let Some(mut child) = self.panel.take() {
						let _ = child.kill();
					}
					self.panel = Some(show_binds_in_submap(name)?);
				} else {
					if let Some(mut child) = self.panel.take() {
						let _ = child.kill();
					}
				}
			},
			e => println!("{e:#?}"),
		}

		Ok(())
	}
}

fn main() {
	let mut eh = EventHandler::new();
	eh.run();
}
