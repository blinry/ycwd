// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    error::Error,
    ffi::OsStr,
    io::{stdout, Write},
    path::PathBuf,
};

#[derive(Debug)]
struct Process {
    cwd: PathBuf,
    tty: i32,
}

#[derive(Debug)]
struct ProcessTree {
    node: Process,
    children: Vec<ProcessTree>,
}

impl ProcessTree {
    fn new(pid: u32) -> Result<ProcessTree, Box<dyn Error>> {
        let p = procfs::process::Process::new(pid as i32)?;
        let node = Process {
            cwd: p.cwd()?,
            tty: p.stat()?.tty_nr,
        };

        let t = p.task_main_thread()?;
        let c_pids = t.children()?;
        let children = c_pids
            .into_iter()
            .map(ProcessTree::new)
            .collect::<Result<_, _>>()?;

        Ok(ProcessTree { node, children })
    }

    fn leaf_nodes_with_tty(&self) -> Vec<&Process> {
        let leaves: Vec<&Process> = self
            .children
            .iter()
            .flat_map(|c| c.leaf_nodes_with_tty())
            .collect();

        if !leaves.is_empty() {
            return leaves;
        }

        if self.node.tty != 0 {
            vec![&self.node]
        } else {
            vec![]
        }
    }
}

fn print_path<T: AsRef<OsStr>>(path: T) {
    stdout()
        .write_all(path.as_ref().as_encoded_bytes())
        .expect("printing doesn't fail");
    println!();
}

fn fallible_main() -> Result<(), Box<dyn Error>> {
    let t = ProcessTree::new(
        std::env::args()
            .nth(1)
            .ok_or("First argument is required.")?
            .parse()?,
    )?;
    let leaves = t.leaf_nodes_with_tty();

    // Print cwd of first leaf.
    print_path(&leaves.first().ok_or("Could not get first leaf node")?.cwd);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(error) = fallible_main() {
        eprintln!("Could not get cwd: {error}");
        if let Some(home) = std::env::var_os("HOME") {
            print_path(home);
        }
    }

    Ok(())
}
