#![forbid(unsafe_code)]

use std::process::ExitCode;
use std::{io, path::PathBuf};

fn main() -> ExitCode {
    match argument().and_then(|path| rev::run(path.as_deref())) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rev: {error}");
            ExitCode::FAILURE
        }
    }
}

fn argument() -> io::Result<Option<PathBuf>> {
    let mut arguments = std::env::args_os().skip(1);
    let path = arguments.next().map(PathBuf::from);
    if arguments.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: rev [file]",
        ));
    }
    Ok(path)
}
