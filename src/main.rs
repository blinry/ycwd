// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
// SPDX-FileCopyrightText: 2025 Joshix <joshix@asozial.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use procfs::{process::Process as ProcProcess, ProcError, ProcResult};
use std::path::PathBuf;

struct ProcessWithDepth {
    depth: usize,
    process: ProcProcess,
}

impl ProcessWithDepth {
    fn valid_cwd(&self) -> ProcResult<Option<PathBuf>> {
        if !self.connected_to_terminal()? {
            return Ok(None);
        }
        Ok(self.process.cwd().ok())
    }

    fn connected_to_terminal(&self) -> ProcResult<bool> {
        Ok(self.process.stat()?.tty_nr != 0)
    }

    fn children(&self) -> ProcResult<Vec<ProcessWithDepth>> {
        let mut children = vec![];
        for task in self.process.tasks()? {
            for child in task?.children()? {
                children.push(ProcessWithDepth {
                    depth: self.depth + 1,
                    process: ProcProcess::new(child as i32)?,
                });
            }
        }
        Ok(children)
    }
}

struct ProcessesWithDepth(Vec<ProcessWithDepth>);

impl ProcessesWithDepth {
    fn init(pid: i32) -> ProcResult<Self> {
        Ok(Self(vec![ProcessWithDepth {
            depth: 0,
            process: ProcProcess::new(pid)?,
        }]))
    }

    fn crawl_all_children(&mut self) -> ProcResult<()> {
        let mut frontier = std::mem::take(&mut self.0);
        while let Some(process) = frontier.pop() {
            frontier.extend(process.children()?);
            self.0.push(process);
        }
        Ok(())
    }

    fn sort_by_depth_descending(&mut self) {
        self.0.sort_by(|a, b| b.depth.cmp(&a.depth));
    }

    fn first_valid_cwd(self) -> ProcResult<PathBuf> {
        self.0
            .iter()
            .find_map(|p| p.valid_cwd().transpose())
            .ok_or_else(|| ProcError::Other("No suitable process found".to_string()))?
    }
}

fn get_cwd() -> ProcResult<PathBuf> {
    let pid = std::env::args()
        .nth(1)
        .ok_or("Provide a process ID as the first argument")?
        .parse()?;

    let mut processes = ProcessesWithDepth::init(pid)?;
    processes.crawl_all_children()?;
    processes.sort_by_depth_descending();
    processes.first_valid_cwd()
}

fn get_cwd_with_fallbacks() -> PathBuf {
    match get_cwd() {
        Ok(path) => return path,
        Err(error) => eprintln!("Could not get cwd, using fallback: {error}"),
    }

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
