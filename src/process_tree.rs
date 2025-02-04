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

    #[inline]
    fn check_children(
        self,
        depth: usize,
        stack: &mut Vec<(usize, ProcResult<Process>)>,
        max: &mut Option<(usize, ProcResult<CwdProcess>)>,
    ) {
        if !self.is_tty() {
            // this can happen when forking of a terminal
            // e.g. some random deeply-nested gcc process
            // we only want to get cwds of procs with tty
            eprintln!("Ignoring process {} (not tty)", self.proc.pid());
            return;
        }
        // add the children to the stack
        match self.children() {
            Ok(children) => stack.extend(children.into_iter().map(|proc| (depth + 1, proc))),
            Err(err) => eprintln!("Could not go deeper: {err}"),
        }
        // the current maximum depth
        let max_depth = max.as_ref().map(|(d, _)| *d).unwrap_or(0);
        if depth <= max_depth {
            // depth isn't greater than max_depth
            // self is not a leaf, can be ignored
            return;
        }
        let max_is_ok = max.as_ref().map(|(_, r)| r.is_ok()).unwrap_or(false);
        // query the cwd of self
        let cwd_child: ProcResult<CwdProcess> = self.try_into();
        if cwd_child.is_ok() || !max_is_ok {
            // could read cwd or max wasn't ok
            *max = Some((depth, cwd_child));
        }
    }

    pub fn into_deepest_leaf(self) -> ProcResult<CwdProcess> {
        let mut stack: Vec<(usize, ProcResult<Process>)> = match self.children() {
            Ok(children) => children.into_iter().map(|proc| (1, proc)).collect(),
            Err(error) => {
                eprintln!("Could not get children: {error}");
                // return self as we couldn't get the children
                return self.try_into();
            }
        };
        // the cwd of the process with the maximum depth
        let mut max: Option<(usize, ProcResult<CwdProcess>)> = None;
        // loop over all the children in the stack
        while let Some((depth, child)) = stack.pop() {
            match child {
                Ok(child) => child.check_children(depth, &mut stack, &mut max),
                Err(err) => eprintln!("Could not get child: {err}"),
            }
        }
        // return the cwd ProcResult with the maximum depth or fallback to self
        max.map(|(_, r)| r).unwrap_or_else(|| self.try_into())
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
