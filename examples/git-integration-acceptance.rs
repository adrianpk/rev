#![forbid(unsafe_code)]

#[path = "../tests/configuration/fixture.rs"]
mod fixture;
#[path = "git_integration_acceptance/journeys.rs"]
mod journeys;
#[path = "git_integration_acceptance/lifetime.rs"]
mod lifetime;
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
#[path = "git_integration_acceptance/resources.rs"]
mod resources;
#[path = "git_integration_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

use support::Result;

fn main() -> Result<()> {
    let mode = std::env::args().nth(1).ok_or("missing acceptance mode")?;
    match mode.as_str() {
        "--native" => native::run(),
        "--resources" => resources::run(),
        "--lifetime" => lifetime::run(),
        "--pty" | "--startup" => {
            let binary = std::env::args_os().nth(2).ok_or("missing editor binary")?;
            let wrapper = std::env::args_os().nth(3).ok_or("missing PTY wrapper")?;
            let binary = std::path::Path::new(&binary);
            let wrapper = std::path::Path::new(&wrapper);
            if mode == "--startup" {
                return pty::startup(binary, wrapper);
            }
            pty::run(binary, wrapper)
        }
        _ => Err("expected --native, --resources, --lifetime, --startup or --pty".into()),
    }
}
