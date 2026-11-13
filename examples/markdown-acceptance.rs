#![forbid(unsafe_code)]

#[path = "markdown_acceptance/edits.rs"]
mod edits;
#[path = "markdown_acceptance/native.rs"]
mod native;
#[path = "markdown_acceptance/pty.rs"]
mod pty;
#[path = "markdown_acceptance/support.rs"]
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
        Some("--native") if args.len() == 2 => native::run(&args[1]),
        Some("--pty") if args.len() == 4 => pty::run(&args[1], &args[2], &args[3]),
        _ => Err("expected --native ROOT or --pty ROOT EDITOR WRAPPER".into()),
    }
}
