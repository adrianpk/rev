#![forbid(unsafe_code)]

#[path = "integrated_terminal_acceptance/native.rs"]
mod native;
#[path = "integrated_terminal_acceptance/support.rs"]
mod support;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let root = PathBuf::from(args.get(1).ok_or("missing owned fixture root")?);
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--native") => native::run(&root),
        _ => Err("expected --native and an owned fixture root".into()),
    }
}
