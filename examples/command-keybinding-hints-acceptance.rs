#![forbid(unsafe_code)]

#[path = "command_keybinding_hints_acceptance/contexts.rs"]
mod contexts;
#[path = "command_keybinding_hints_acceptance/native.rs"]
mod native;
#[path = "command_keybinding_hints_acceptance/presentation.rs"]
mod presentation;
#[path = "command_keybinding_hints_acceptance/pty.rs"]
mod pty;
#[allow(
    dead_code,
    reason = "Reuse existing native session and process helpers."
)]
#[path = "completion_acceptance/support.rs"]
mod support;
#[allow(dead_code, reason = "Only command-row colors are exercised here.")]
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
            &args[1],
            &args[2],
            &args[3],
            args[4].to_str().ok_or("invalid profile")?,
        ),
        _ => Err("expected --native ROOT or --journey ROOT EDITOR WRAPPER PROFILE".into()),
    }
}
