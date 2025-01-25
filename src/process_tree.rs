// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ops::Deref, path::PathBuf};

use procfs::{process, ProcError, ProcResult};

#[derive(Debug)]
pub struct CwdProcess {
    proc: Process,
    cwd: PathBuf,
}

impl CwdProcess {
    pub fn into_cwd(self) -> PathBuf {
        self.cwd
    }
}

#[derive(Debug)]
pub struct Process {
    proc: process::Process,
}

impl Process {
    pub fn new(pid: u32) -> ProcResult<Process> {
        let proc = process::Process::new(pid as i32)?;

        Ok(Process { proc })
    }

    fn children(&self) -> ProcResult<impl IntoIterator<Item = ProcResult<Process>>> {
        Ok(self
            .proc
            .task_main_thread()?
            .children()?
            .into_iter()
            .map(Process::new))
    }

    pub fn into_deepest_leaf(self) -> ProcResult<CwdProcess> {
        fn deepest_leaf(depth: usize, tree: Process) -> (usize, ProcResult<CwdProcess>) {
            let children = tree.children();
            let mut max: Option<(usize, ProcResult<CwdProcess>)> = None;
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

impl From<CwdProcess> for Process {
    fn from(value: CwdProcess) -> Self {
        value.proc
    }
}

impl Deref for CwdProcess {
    type Target = Process;

    fn deref(&self) -> &Self::Target {
        &self.proc
    }
}

impl TryFrom<Process> for CwdProcess {
    type Error = ProcError;

    fn try_from(proc: Process) -> Result<Self, Self::Error> {
        let cwd = proc.proc.cwd()?;
        Ok(CwdProcess { cwd, proc })
    }
}
