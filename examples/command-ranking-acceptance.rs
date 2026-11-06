#![forbid(unsafe_code)]

#[path = "command_ranking_acceptance/native.rs"]
mod native;
#[path = "command_ranking_acceptance/pty.rs"]
mod pty;
#[allow(
    dead_code,
    reason = "Reuse the native session and Linux process helpers."
)]
#[path = "completion_acceptance/support.rs"]
mod support;
#[allow(
    dead_code,
    reason = "This matrix checks selection colors, not gutter variants."
)]
#[path = "../tests/support/surface.rs"]
mod surface;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--native") if args.len() == 2 => native::run(&args[1]),
        Some("--journey") if args.len() == 5 => pty::journey(
            &args[1], &args[2], &args[3], args[4].to_str().ok_or("invalid profile")?,
        ),
        Some("--paired") if args.len() == 4 => pty::paired(&args[1], &args[2], &args[3]),
        _ => Err("expected --native ROOT, --journey ROOT EDITOR WRAPPER PROFILE or --paired ROOT EDITOR WRAPPER".into()),
    }
}
