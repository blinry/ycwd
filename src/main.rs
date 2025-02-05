// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
// SPDX-FileCopyrightText: 2025 Joshix <joshix@asozial.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use procfs::{process::Process, ProcError, ProcResult};
use std::path::PathBuf;

fn crawl_children(pid: i32) -> ProcResult<Vec<(usize, Process)>> {
    let mut frontier = vec![(0, Process::new(pid)?)];
    let mut processes: Vec<(usize, Process)> = vec![];
    while let Some((depth, process)) = frontier.pop() {
        for task in process.tasks()? {
            for child in task?.children()? {
                frontier.push((depth + 1, Process::new(child as i32)?));
            }
        }
        processes.push((depth, process));
    }
    Ok(processes)
}

fn get_cwd() -> ProcResult<PathBuf> {
    let pid = std::env::args()
        .nth(1)
        .ok_or("Provide a process ID as the first argument")?
        .parse()?;

    // 1. Construct a vector of (depth, process) tuples of all child processes.
    let mut processes = crawl_children(pid)?;

    // 2. Sort the vector by depth in descending order.
    processes.sort_by(|a, b| b.0.cmp(&a.0));

    // 3. Find the first process that is connected to a tty, and where we can read its cwd.
    for (_, process) in &processes {
        if process.stat()?.tty_nr != 0 {
            if let Ok(cwd) = process.cwd() {
                return Ok(cwd);
            }
        }
    }

    Err(ProcError::Other("No suitable process found".to_string()))
}

fn get_cwd_with_fallbacks() -> PathBuf {
    match get_cwd() {
        Ok(path) => return path,
        Err(error) => eprintln!("Could not get cwd, using fallback: {error}"),
    };

    match std::env::var_os("HOME") {
        Some(home) => return home.into(),
        None => eprintln!("Can't use $HOME as fallback, because it is not set"),
    }

    "/".into()
}

fn main() {
    let path = get_cwd_with_fallbacks();
    println!("{}", path.display());
}
