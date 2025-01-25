// SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later
mod process_tree;

use std::{
    ffi::OsString,
    io::{stdout, Write},
    path::PathBuf,
};

use process_tree::Process;
use procfs::ProcResult;

fn get_path() -> ProcResult<PathBuf> {
    let t = Process::new(
        std::env::args()
            .nth(1)
            .ok_or("First argument is required")?
            .parse()?,
    )?;

    t.into_deepest_leaf().map(|proc| proc.into_cwd())
}

fn get_path_with_fallbacks() -> OsString {
    match get_path() {
        Ok(path) => return path.into(),
        Err(error) => eprintln!("Could not get cwd: {error}"),
    };

    match std::env::var_os("HOME") {
        Some(home) => return home,
        None => eprintln!("HOME not set"),
    }

    "/".into()
}

fn main() -> std::io::Result<()> {
    let path = get_path_with_fallbacks();
    stdout().write_all(path.as_encoded_bytes())?;
    println!();
    Ok(())
}
