#![forbid(unsafe_code)]

#[path = "bookmark_acceptance/journey.rs"]
mod journey;
#[path = "bookmark_acceptance/pty.rs"]
mod pty;
#[path = "bookmark_acceptance/resources.rs"]
mod resources;
#[path = "bookmark_acceptance/support.rs"]
mod support;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    assert_eq!(args.len(), 3, "expected fixture, binary and PTY wrapper");
    journey::run(&args[0])?;
    resources::run(&args[0])?;
    pty::run(&args[0], &args[1], &args[2])?;
    Ok(())
}
