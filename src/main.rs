#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    match rev::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rev: {error}");
            ExitCode::FAILURE
        }
    }
}
