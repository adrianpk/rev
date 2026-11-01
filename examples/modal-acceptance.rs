#![forbid(unsafe_code)]

#[path = "modal_acceptance/native.rs"]
mod native;
#[path = "modal_acceptance/pty.rs"]
mod pty;
#[path = "modal_acceptance/resources.rs"]
mod resources;
#[path = "modal_acceptance/semantic.rs"]
mod semantic;
#[path = "completion_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/pty.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args
        .first()
        .is_some_and(|arg| arg.to_string_lossy().starts_with("--"))
    {
        return focused(&args);
    }
    assert_eq!(args.len(), 3, "expected fixture, editor and PTY wrapper");
    let started = std::time::Instant::now();
    native::run(&args[0])?;
    resources::run(&args[0], &args[1])?;
    semantic::run(&args[0])?;
    terminals(&args)?;
    println!(
        "PASS modal acceptance: isolated profiles, native edits/selections/history, installed Rust semantics, bounded work and restored Linux PTYs"
    );
    Ok(())
}

fn focused(args: &[PathBuf]) -> Result<()> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--native") => native::run(args.get(1).ok_or("missing native fixture")?),
        Some("--semantic") => semantic::exercise(args.get(1).ok_or("missing owned Rust root")?),
        Some("--pty") => terminals(&args[1..]),
        _ => Err("unknown acceptance mode".into()),
    }
}

fn terminals(args: &[PathBuf]) -> Result<()> {
    assert_eq!(args.len(), 3, "expected fixture, editor and PTY wrapper");
    terminal::run(&args[0], &args[1], &args[2])?;
    pty::run(&args[0], &args[1], &args[2])
}
