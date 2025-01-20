// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ops::Deref, path::PathBuf};

use procfs::{
    process::{Process, Task},
    ProcError, ProcResult,
};

#[derive(Debug)]
pub struct CwdProcessTree {
    tree: ProcessTree,
    cwd: PathBuf,
}

#[derive(Debug)]
pub struct ProcessTree {
    proc: Process,
    task: Task,
}

impl ProcessTree {
    pub fn new(pid: u32) -> ProcResult<ProcessTree> {
        let proc = Process::new(pid as i32)?;

        let task = proc.task_main_thread()?;

        Ok(ProcessTree { proc, task })
    }

    fn children(&self) -> ProcResult<impl IntoIterator<Item = ProcResult<ProcessTree>>> {
        Ok(self.task.children()?.into_iter().map(ProcessTree::new))
    }

    pub fn into_deepest_leaf(self) -> ProcResult<CwdProcessTree> {
        fn deepest_leaf(depth: usize, tree: ProcessTree) -> (usize, ProcResult<CwdProcessTree>) {
            let children = tree.children();
            let mut max: Option<(usize, ProcResult<CwdProcessTree>)> = None;
            match children {
                Ok(children) => {
                    for child in children {
                        match child {
                            Ok(child) => {
                                let leaf = deepest_leaf(depth + 1, child);
                                match leaf.1.as_ref() {
                                    Ok(_) => match max.as_ref() {
                                        Some(value) => {
                                            if value.0 < leaf.0 {
                                                max = Some(leaf);
                                            }
                                        }
                                        None => max = Some(leaf),
                                    },
                                    Err(err) => eprintln!("Could not go deeper: {err}"),
                                }
                            }
                            Err(err) => eprintln!("Could not get child: {err}"),
                        }
                    }
                }
                Err(err) => eprintln!("Could not get children: {err}"),
            }

            match max {
                Some(max) => max,
                None => (depth, tree.try_into()),
            }
        }

        deepest_leaf(0, self).1
    }
}

impl From<CwdProcessTree> for ProcessTree {
    fn from(value: CwdProcessTree) -> Self {
        value.tree
    }
}

impl Deref for CwdProcessTree {
    type Target = ProcessTree;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl From<CwdProcessTree> for PathBuf {
    fn from(value: CwdProcessTree) -> Self {
        value.cwd
    }
}

impl TryFrom<ProcessTree> for CwdProcessTree {
    type Error = ProcError;

    fn try_from(tree: ProcessTree) -> Result<Self, Self::Error> {
        let cwd = tree.proc.cwd()?;
        Ok(CwdProcessTree { cwd, tree })
    }
}
