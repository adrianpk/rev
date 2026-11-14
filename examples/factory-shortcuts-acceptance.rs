#![forbid(unsafe_code)]

#[path = "../tests/configuration/fixture.rs"]
mod fixture;
#[path = "factory_shortcuts_acceptance/matrix.rs"]
mod matrix;
#[path = "shell_command_acceptance/metrics.rs"]
mod metrics;
#[path = "factory_shortcuts_acceptance/native.rs"]
mod native;
#[path = "factory_shortcuts_acceptance/pty.rs"]
mod pty;
#[path = "../tests/git_integration/repository_fixture.rs"]
mod repository;
#[path = "factory_shortcuts_acceptance/support.rs"]
mod support;
#[path = "../tests/support/surface.rs"]
mod surface;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--native") if args.len() == 2 => run_native(),
        Some("--journey") if args.len() == 5 => pty::editing(
            &args[1],
            &args[2],
            &args[3],
            args[4].to_str().ok_or("invalid profile")?,
        ),
        Some("--git") if args.len() == 3 => pty::git(&args[1], &args[2]),
        _ => Err(
            "expected --native ROOT, --journey ROOT EDITOR WRAPPER PROFILE or --git EDITOR WRAPPER"
                .into(),
        ),
    }
}

fn run_native() -> Result<()> {
    matrix::run()?;
    native::run()?;
    println!("MEASURE native process threads={}", metrics::threads()?);
    Ok(())
}
