#![forbid(unsafe_code)]

#[path = "lsp_installation_acceptance/fixture.rs"]
mod fixture;
#[path = "lsp_installation_acceptance/native.rs"]
mod native;
#[path = "lsp_installation_acceptance/pty.rs"]
mod pty;
#[path = "lsp_installation_acceptance/semantic.rs"]
mod semantic;
#[path = "lsp_installation_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use std::path::PathBuf;
use support::Result;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.first().and_then(|s| s.to_str()) {
        Some("--fixture") => fixture::prepare(args.get(1).ok_or("missing fixture root")?),
        Some("--native") => native::run(args.get(1).ok_or("missing native root")?),
        Some("--pty") => {
            assert_eq!(args.len(), 4, "expected mode, root, editor and PTY wrapper");
            pty::run(&args[1], &args[2], &args[3])
        }
        _ => Err("expected --fixture, --native or --pty".into()),
    }
}
