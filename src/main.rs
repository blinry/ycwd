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

fn print_path<T: AsRef<OsStr>>(path: T) -> std::io::Result<()> {
    stdout().write_all(path.as_ref().as_encoded_bytes())?;
    println!();
    Ok(())
}

fn get_path() -> Result<PathBuf, Box<dyn Error>> {
    let t = ProcessTree::new(
        std::env::args()
            .nth(1)
            .ok_or("First argument is required.")?
            .parse()?,
    )?;
    let leafs = t.leaf_nodes_with_tty();

    let first_leaf = leafs.first().ok_or("Could not get first leaf node")?;

    Ok(first_leaf.cwd.clone())
}

fn main() -> std::io::Result<()> {
    match get_path() {
        Ok(path) => print_path(path),
        Err(error) => {
            eprintln!("Could not get cwd: {error}");
            if let Some(home) = std::env::var_os("HOME") {
                print_path(home)
            } else {
                eprintln!("HOME not set");
                print_path("/")
            }
        }
    }
}
