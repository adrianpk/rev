#![forbid(unsafe_code)]

#[path = "../tests/configuration/fixture.rs"]
mod fixture;
#[path = "git_integration_acceptance/journeys.rs"]
mod journeys;
#[path = "git_integration_acceptance/matrix.rs"]
mod matrix;
#[path = "shell_command_acceptance/metrics.rs"]
mod metrics;
#[path = "git_integration_acceptance/native.rs"]
mod native;
#[path = "git_integration_acceptance/pty.rs"]
mod pty;
#[path = "../tests/git_integration/support.rs"]
mod repository;
#[path = "git_integration_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use support::Result;

fn main() -> Result<()> {
    let mode = std::env::args().nth(1).ok_or("missing acceptance mode")?;
    match mode.as_str() {
        "--native" => native::run(),
        "--pty" => {
            let binary = std::env::args_os().nth(2).ok_or("missing editor binary")?;
            let wrapper = std::env::args_os().nth(3).ok_or("missing PTY wrapper")?;
            pty::run(
                std::path::Path::new(&binary),
                std::path::Path::new(&wrapper),
            )
        }
        _ => Err("expected --native or --pty".into()),
    }
}
