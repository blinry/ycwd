// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later
mod process_tree;

use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use process_tree::ProcessTree;
use procfs::ProcResult;

fn get_path() -> ProcResult<PathBuf> {
    let t = ProcessTree::new(
        std::env::args()
            .nth(1)
            .ok_or("First argument is required")?
            .parse()?,
    )?;

    t.into_deepest_leaf().map(|proc| proc.into_cwd())
}

fn main() -> std::io::Result<()> {
    let path = match get_path() {
        Ok(path) => path.into(),
        Err(error) => {
            eprintln!("Could not get cwd: {error}");
            if let Some(home) = std::env::var_os("HOME") {
                home
            } else {
                eprintln!("HOME not set");
                "/".into()
            }
        }
    };
    stdout().write_all(path.as_encoded_bytes())?;
    println!();
    Ok(())
}
