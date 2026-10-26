#![forbid(unsafe_code)]

pub mod buffer;
pub mod command;
pub mod coordinates;
pub mod display;
pub mod input;
pub mod session;
pub mod terminal;
pub mod view;

use std::io;

use terminal::Terminal;

/// Runs the terminal session and reports errors after attempting restoration.
pub fn run() -> io::Result<()> {
    let mut terminal = Terminal::acquire(io::stdout())?;
    let result = terminal.enter().and_then(|()| session::run(&mut terminal));
    let cleanup = terminal.restore();
    match (result, cleanup) {
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; terminal restoration: {cleanup}"),
        )),
        (result, cleanup) => result.and(cleanup),
    }
}
