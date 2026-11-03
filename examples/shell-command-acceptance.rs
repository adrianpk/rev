#![forbid(unsafe_code)]

#[path = "shell_command_acceptance/metrics.rs"]
mod metrics;
#[path = "shell_command_acceptance/native.rs"]
mod native;
#[path = "shell_command_acceptance/pty.rs"]
mod pty;
#[path = "shell_command_acceptance/resources.rs"]
mod resources;
#[path = "shell_command_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = args.first().and_then(|arg| arg.to_str());
    let root = PathBuf::from(args.get(1).ok_or("missing owned fixture root")?);
    match mode {
        Some("--native") => native::run(&root),
        Some("--resources") => resources::run(&root),
        Some("--pty") => terminals(&args[1..]),
        _ => Err("expected --native, --resources or --pty and an owned fixture root".into()),
    }
}

fn terminals(args: &[std::ffi::OsString]) -> Result<()> {
    let root = PathBuf::from(args.first().ok_or("missing fixture root")?);
    let binary = PathBuf::from(args.get(1).ok_or("missing editor binary")?);
    let wrapper = PathBuf::from(args.get(2).ok_or("missing PTY wrapper")?);
    pty::run(&root, &binary, &wrapper)
}
