#![forbid(unsafe_code)]

#[path = "integrated_terminal_acceptance/native.rs"]
mod native;
#[path = "integrated_terminal_acceptance/support.rs"]
mod support;

#[path = "integrated_terminal_acceptance/failure.rs"]
mod failure;
#[path = "shell_command_acceptance/metrics.rs"]
mod metrics;
#[path = "integrated_terminal_acceptance/pty.rs"]
mod pty;
#[path = "integrated_terminal_acceptance/resources.rs"]
mod resources;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;
use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let root = PathBuf::from(args.get(1).ok_or("missing owned fixture root")?);
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--native") => native::run(&root),
        Some("--resources") => resources::run(
            &root,
            &PathBuf::from(args.get(2).ok_or("missing editor binary")?),
        ),
        Some("--host-error") => failure::host(&root),
        Some("--pty") => pty::run(
            &root,
            &PathBuf::from(args.get(2).ok_or("missing editor binary")?),
            &PathBuf::from(args.get(3).ok_or("missing PTY wrapper")?),
        ),
        _ => Err("expected --native, --host-error or --pty and owned paths".into()),
    }
}
