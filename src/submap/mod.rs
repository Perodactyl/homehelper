use std::process::{Child, Command, Stdio};

use anyhow::Result;

use crate::hyprctl::{self, Bind};

pub fn open_kitty(cmd: &str, lines: usize) -> Result<Child> {
	let mut c = Command::new("kitty");
	let args = [
		"+kitten", "panel",
		"--edge", "none",
		"--layer", "top",
		"--margin-top", "40",
		"--margin-bottom", "10",
		"--margin-right", "10",
		"--lines", &lines.to_string(),
		"--columns", "70",
		"--app-id", "homehelper-submap",
		"sh", "-c", cmd
	];

	c.args(&args);
	c.stdout(Stdio::null());
	c.stderr(Stdio::null());
	Ok(c.spawn()?)
}

pub fn show_binds(binds: Vec<Bind>) -> Result<Child> {
	let mut longest_bind_name = 0;
	for bind in &binds {
		longest_bind_name = longest_bind_name.max(bind.key.len());
	}

	let bind_str = binds.iter().map(|b| {
		let mut spacing = String::with_capacity(longest_bind_name+1);

		while b.key.len() + spacing.len() <= longest_bind_name {
			spacing.push(' ');
		}

		format!("{}{spacing}{}", b.key, b.description.as_ref().unwrap())
	}).collect::<Vec<_>>().join("\n");

	let cmd = format!("printf '\x1b[?25l{bind_str}'; sleep infinity");
	let cp = open_kitty(&cmd, binds.len())?;

	Ok(cp)
}

pub fn show_binds_in_submap(name: String) -> Result<Child> {
	let all_binds = hyprctl::binds()?;
	let next_binds: Vec<Bind> = all_binds.into_iter()
		.filter(|b| {
			if let Some(submap) = &b.submap {
				*submap == name && b.description.is_some()
			} else {
				false
			}
		})
		.collect();

	show_binds(next_binds)
}
