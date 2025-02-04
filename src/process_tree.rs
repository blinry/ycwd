// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
// SPDX-FileCopyrightText: 2025 Joshix <joshix@asozial.org>
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

    pub fn is_tty(&self) -> bool {
        match self.proc.stat() {
            Ok(stat) => stat.tty_nr != 0,
            Err(_) => false,
        }
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
                                let (depth, leaf) = deepest_leaf(depth + 1, child);
                                match leaf.as_ref() {
                                    Ok(proc) if proc.is_tty() => match max.as_ref() {
                                        Some((max_depth, saved)) => {
                                            if *max_depth <= depth || saved.is_err() {
                                                max = Some((depth, leaf));
                                            }
                                        }
                                        None => max = Some((depth, leaf)),
                                    },
                                    Ok(proc) => {
                                        eprintln!(
                                            "Ignoring process {} (not tty)",
                                            proc.proc.proc.pid()
                                        )
                                    }
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

        let (_, result) = deepest_leaf(0, self);

        result
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
