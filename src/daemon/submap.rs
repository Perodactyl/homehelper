use std::process::{Child, Command, Stdio};

use anyhow::Result;

use crate::hyprctl::{self, Bind};

pub fn open_kitty(cmd: &str, lines: usize, longest_line: usize) -> Result<Child> {
    let mut c = Command::new("kitty");

	//This is why we can't have rustfmt
    let args = [
        "+kitten", "panel",
		"--edge", "center-sized",
        "--layer", "top",
        "--lines", &lines.to_string(),
        "--columns", &longest_line.to_string(),
        "--app-id", "homehelper-submap",
        "sh", "-c", cmd,
    ];

    c.args(args);
    c.stdout(Stdio::null());
    c.stderr(Stdio::null());
    Ok(c.spawn()?)
}

pub fn show_binds(binds: &[Bind]) -> Result<Child> {
    let mut longest_bind_name = 0;
    for bind in binds {
        longest_bind_name = longest_bind_name.max(bind.key.len());
    }
	let mut longest_line = 0;

    let bind_str = binds
        .iter()
        .map(|b| {
            let mut spacing = String::with_capacity(longest_bind_name + 1);

            while b.key.len() + spacing.len() <= longest_bind_name {
                spacing.push(' ');
            }

            let out = format!("{}{spacing}{}", b.key, b.description.as_ref().unwrap());
			longest_line = longest_line.max(out.len());
			out
        })
        .collect::<Vec<_>>()
        .join("\n");

    let cmd = format!("printf '\x1b[?25l{bind_str}'; sleep infinity");
    let cp = open_kitty(&cmd, binds.len(), longest_line)?;

    Ok(cp)
}

pub fn show_binds_in_submap(name: &str) -> Result<Child> {
    let all_binds = hyprctl::binds()?;
    let next_binds: Vec<Bind> = all_binds
        .into_iter()
        .filter(|b| {
            if let Some(submap) = &b.submap {
                *submap == name && b.description.is_some()
            } else {
                false
            }
        })
        .collect();

    show_binds(&next_binds)
}
