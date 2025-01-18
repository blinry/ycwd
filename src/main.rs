// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

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
    fn new(pid: u32) -> ProcessTree {
        let p = procfs::process::Process::new(pid as i32).expect("Process should exist");
        let node = Process {
            cwd: p.cwd().expect("Process should have a cwd"),
            tty: p.stat().expect("Process should hav status info").tty_nr,
        };

        let t = p
            .task_main_thread()
            .expect("Process should have main thread");
        let c_pids = t.children().expect("Task should have children");
        let children = c_pids
            .iter()
            .map(|&c_pid| ProcessTree::new(c_pid))
            .collect();
        ProcessTree { node, children }
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

fn main() {
    let t = ProcessTree::new(
        std::env::args()
            .nth(1)
            .expect("PID should be provided as first argument")
            .parse()
            .expect("PID should be a number"),
    );
    let leaves = t.leaf_nodes_with_tty();

    // Print cwd of first leaf.
    println!("{}", leaves[0].cwd.display());
}
