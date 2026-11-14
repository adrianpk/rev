#![forbid(unsafe_code)]

#[path = "../tests/configuration/fixture.rs"]
mod fixture;
#[path = "factory_shortcuts_acceptance/matrix.rs"]
mod matrix;
#[path = "factory_shortcuts_acceptance/native.rs"]
mod native;
#[path = "factory_shortcuts_acceptance/support.rs"]
mod support;
#[path = "../tests/support/surface.rs"]
mod surface;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--native") if args.len() == 2 => {
            matrix::run()?;
            native::run()
        }
        _ => Err("expected --native ROOT".into()),
    }
}
