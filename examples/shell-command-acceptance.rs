#![forbid(unsafe_code)]

#[path = "shell_command_acceptance/native.rs"]
mod native;
#[path = "shell_command_acceptance/support.rs"]
mod support;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = args.first().and_then(|arg| arg.to_str());
    let root = PathBuf::from(args.get(1).ok_or("missing owned fixture root")?);
    match mode {
        Some("--native") => native::run(&root),
        _ => Err("expected --native and an owned fixture root".into()),
    }
}
