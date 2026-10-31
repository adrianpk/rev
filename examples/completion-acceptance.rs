#![forbid(unsafe_code)]

#[path = "../tests/support/language_assets.rs"]
mod assets;
#[path = "../tests/configuration/fixture.rs"]
mod fixture;
#[path = "completion_acceptance/native.rs"]
mod native;
#[path = "completion_acceptance/pty.rs"]
mod pty;
#[path = "completion_acceptance/resources.rs"]
mod resources;
#[path = "completion_acceptance/support.rs"]
mod support;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    assert_eq!(args.len(), 3, "expected fixture, binary and PTY wrapper");
    // This producer runs before constructing any editor. Runtime only loads
    // these precompiled libraries; test-only Python is not bundled into rev.
    let libraries = assets::libraries();
    native::run(&args[0], &libraries)?;
    resources::run(&args[0], &args[1])?;
    pty::run(&args[0], &args[1], &args[2])?;
    println!(
        "PASS native completion acceptance: ten parsers, Org, external assets, checked edits, bounded work and Linux PTY"
    );
    Ok(())
}
