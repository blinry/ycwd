// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
// SPDX-FileCopyrightText: 2025 Joshix <joshix@asozial.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ops::Deref, path::PathBuf};

use procfs::{process, ProcError, ProcResult};

type Depth = usize;
type Frontier = Vec<(Depth, Process)>;

pub struct CwdProcess {
    proc: Process,
    cwd: PathBuf,
}

impl CwdProcess {
    pub fn into_cwd(self) -> PathBuf {
        self.cwd
    }

    #[allow(dead_code)]
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    #[allow(dead_code)]
    pub fn process(&self) -> &Process {
        &self.proc
    }
}

pub struct Process {
    proc: process::Process,
}

impl Process {
    pub fn new(pid: u32) -> ProcResult<Process> {
        let proc = process::Process::new(pid as i32)?;

        Ok(Process { proc })
    }

    pub fn is_tty(&self) -> bool {
        match self.proc.stat() {
            Ok(stat) => stat.tty_nr != 0,
            Err(_) => false,
        }
    }

    #[inline]
    fn add_children_to_stack(&self, depth: Depth, frontier: &mut Frontier) -> ProcResult<()> {
        for task in self.tasks()? {
            let children = task?.children()?;
            frontier.reserve(children.len());
            for child in children {
                match Process::new(child) {
                    Ok(child) => frontier.push((depth, child)),
                    Err(err) => eprintln!("Could not get child of {}: {err}", self.pid()),
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn find_deepest_leaf(
        self,
        depth: Depth,
        frontier: &mut Frontier,
        deepest_leaf: &mut Option<(Depth, ProcResult<CwdProcess>)>,
    ) {
        if !self.is_tty() {
            // this can happen when forking of a terminal
            // e.g. some random deeply-nested gcc process
            // we only want to get cwds of procs with tty
            eprintln!("Ignoring process {} (not tty)", self.pid());
            return;
        }
        // add the children to the stack
        if let Err(err) = self.add_children_to_stack(depth + 1, frontier) {
            eprintln!("Error while getting children of {}: {err}", self.pid());
        }
        // the current maximum depth
        let max_depth = deepest_leaf.as_ref().map(|(d, _)| *d).unwrap_or(0);
        if depth <= max_depth {
            // depth isn't greater than max_depth
            // self is not a leaf, can be ignored
            return;
        }
        let max_is_ok = deepest_leaf
            .as_ref()
            .map(|(_, r)| r.is_ok())
            .unwrap_or(false);

        // query the cwd of self
        let cwd: ProcResult<_> = self.try_into();
        if cwd.is_ok() || !max_is_ok {
            // could read cwd or max wasn't ok
            *deepest_leaf = Some((depth, cwd));
        }
    }

    pub fn into_deepest_leaf(self) -> ProcResult<CwdProcess> {
        let mut frontier: Frontier = vec![];
        if let Err(error) = self.add_children_to_stack(1, &mut frontier) {
            eprintln!("Could not get children: {error}");
            // return self as we couldn't get the children
            return self.try_into();
        };
        // the cwd of the process with the maximum depth
        let mut deepest_leaf: Option<(Depth, ProcResult<CwdProcess>)> = None;
        // loop over all the children in the stack
        while let Some((depth, child)) = frontier.pop() {
            child.find_deepest_leaf(depth, &mut frontier, &mut deepest_leaf);
        }
        // return the cwd ProcResult with the maximum depth or fallback to self
        deepest_leaf
            .map(|(_, cwd_result)| cwd_result)
            .unwrap_or_else(|| self.try_into())
    }
}

impl Deref for Process {
    type Target = process::Process;

    fn deref(&self) -> &Self::Target {
        &self.proc
    }
}

impl From<CwdProcess> for Process {
    fn from(value: CwdProcess) -> Self {
        value.proc
    }
}

impl TryFrom<Process> for CwdProcess {
    type Error = ProcError;

    fn try_from(proc: Process) -> Result<Self, Self::Error> {
        let cwd = proc.proc.cwd()?;
        Ok(CwdProcess { cwd, proc })
    }
}
