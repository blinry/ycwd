// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    error::Error,
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

fn fallible_main() -> Result<(), Box<dyn Error>> {
    let t = ProcessTree::new(
        std::env::args()
            .nth(1)
            .ok_or_else(|| "First argument is required.".to_string())?
            .parse()?,
    )?;
    let leaves = t.leaf_nodes_with_tty();

    // Print cwd of first leaf.
    println!("{}", leaves[0].cwd.display());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(error) = fallible_main() {
        eprintln!("Could not get cwd: {error}");
        if let Some(home) = std::env::var_os("HOME") {
            stdout().write_all(home.as_encoded_bytes())?;
            println!();
        }
    }

    Ok(())
}
