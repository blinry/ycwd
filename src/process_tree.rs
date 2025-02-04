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
        let mut stack: Vec<(usize, ProcResult<Process>)> = match self.children() {
            Ok(children) => children.into_iter().map(|proc| (1, proc)).collect(),
            Err(error) => {
                eprintln!("Could not get children: {error}");
                return self.try_into();
            }
        };
        let mut max: Option<(usize, ProcResult<CwdProcess>)> = None;
        while let Some((depth, child)) = stack.pop() {
            match child {
                Ok(child) => {
                    if !child.is_tty() {
                        eprintln!("Ignoring process {} (not tty)", child.proc.pid());
                        continue;
                    }
                    match child.children() {
                        Ok(children) => {
                            stack.extend(children.into_iter().map(|proc| (depth + 1, proc)));
                        }
                        Err(err) => eprintln!("Could not go deeper: {err}"),
                    }
                    let max_depth = max.as_ref().map(|(d, _)| *d).unwrap_or(0);
                    let max_is_ok = max.as_ref().map(|(_, r)| r.is_ok()).unwrap_or(false);
                    if depth > max_depth {
                        let cwd_child: ProcResult<CwdProcess> = child.try_into();
                        if cwd_child.is_ok() || !max_is_ok {
                            max = Some((depth, cwd_child));
                        }
                    }
                }
                Err(err) => eprintln!("Could not get child: {err}"),
            }
        }

        max.map(|(_, proc_result)| proc_result)
            .unwrap_or_else(|| self.try_into())
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
