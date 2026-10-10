//! Reproducible workspace commands. Node is already required to build upstream.
#![forbid(unsafe_code)]

use std::{
    env,
    path::Path,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("workspace root is missing")?;
    let status = Command::new("node")
        .arg(root.join("tools/xtask.mjs"))
        .args(env::args().skip(1))
        .current_dir(root)
        .status()
        .map_err(|error| format!("cannot run Node: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed ({status})"))
    }
}
