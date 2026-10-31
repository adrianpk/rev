#![forbid(unsafe_code)]

#[path = "wrap_navigation/journey.rs"]
mod journey;
#[path = "wrap_navigation/pty.rs"]
mod pty;

use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    assert_eq!(
        arguments.len(),
        3,
        "expected fixture, binary and PTY wrapper"
    );
    journey::run(&arguments[0])?;
    pty::run(&arguments[0], &arguments[1], &arguments[2])?;
    Ok(())
}
