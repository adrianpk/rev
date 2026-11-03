#![forbid(unsafe_code)]

#[path = "org_heading_acceptance/native.rs"]
mod native;
#[path = "org_heading_acceptance/pty.rs"]
mod pty;
#[path = "org_heading_acceptance/resources.rs"]
mod resources;
#[path = "org_heading_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--native") => native::run(args.get(1).ok_or("missing fixture")?),
        Some("--resources") => resources::run(args.get(1).ok_or("missing fixture")?),
        Some("--pty") if args.len() == 4 => pty::run(&args[1], &args[2], &args[3]),
        _ => Err("expected --native ROOT, --resources ROOT or --pty ROOT EDITOR WRAPPER".into()),
    }
}
