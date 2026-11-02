#![forbid(unsafe_code)]

#[path = "../tests/support/language_assets.rs"]
mod assets;
#[path = "../tests/configuration/fixture.rs"]
mod fixture;
#[path = "lsp_acceptance/mod.rs"]
mod lsp;
#[path = "completion_acceptance/native.rs"]
mod native;
#[path = "completion_acceptance/pty.rs"]
mod native_pty;
#[path = "completion_acceptance/resources.rs"]
mod resources;
#[path = "completion_acceptance/support.rs"]
mod support;

#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.first().is_some_and(|arg| arg == "--real") {
        return lsp::real::exercise(args.get(1).ok_or("missing isolated Rust root")?);
    }
    assert_eq!(args.len(), 3, "expected fixture, editor and PTY wrapper");
    let libraries = assets::libraries();
    lsp::real::run(&args[0])?;
    lsp::controlled::run(&args[0])?;
    native::run(&args[0], &libraries)?;
    resources::run(&args[0], &args[1])?;
    native_pty::run(&args[0], &args[1], &args[2])?;
    lsp::pty::run(&args[0], &args[1], &args[2])?;
    println!(
        "PASS joint LSP/native acceptance: real unsaved Rust semantics, resolved edits, source switching, bounded ownership and Linux PTY"
    );
    Ok(())
}
