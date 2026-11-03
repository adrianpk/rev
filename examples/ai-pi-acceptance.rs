#![forbid(unsafe_code)]

#[path = "ai_pi_acceptance/fixture.rs"]
mod fixture;
#[path = "ai_pi_acceptance/live.rs"]
mod live;
#[path = "ai_pi_acceptance/native.rs"]
mod native;
#[path = "ai_pi_acceptance/pty.rs"]
mod pty;
#[path = "ai_pi_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.first().and_then(|s| s.to_str()) {
        Some("--version" | "--mode") => fixture::proxy(&args),
        Some("--native") => native::run(args.get(1).ok_or("missing native root")?),
        Some("--pty") if args.len() == 4 => pty::run(&args[1], &args[2], &args[3]),
        Some("--live") => live::run(args.get(1).ok_or("missing toy root")?),
        _ => Err("expected --native ROOT, --pty ROOT EDITOR WRAPPER or --live ROOT".into()),
    }
}
