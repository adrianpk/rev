#![forbid(unsafe_code)]

pub mod ai;
pub mod bookmark;
pub mod buffer;
pub mod command;
mod completion;
pub mod config;
pub mod coordinates;
pub mod display;
pub mod file;
pub mod gutter;
pub mod input;
mod jump;
mod kill;
mod location;
pub mod lsp;
pub mod org;
pub mod panel;
pub mod pointer;
mod process;
pub mod rectangle;
pub mod search;
pub mod session;
mod shell;
pub mod syntax;
pub mod terminal;
pub mod tree;
pub mod view;
pub mod window;

#[cfg(test)]
#[path = "../tests/configuration/fixture.rs"]
mod fixture;

use std::io;
use std::path::Path;

use terminal::Terminal;

/// Runs the terminal session and reports errors after attempting restoration.
pub fn run(path: Option<&Path>) -> io::Result<()> {
    let mut terminal = Terminal::acquire(io::stdout())?;
    let result = terminal
        .enter()
        .and_then(|()| session::run(&mut terminal, path));
    let cleanup = terminal.restore();
    match (result, cleanup) {
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; terminal restoration: {cleanup}"),
        )),
        (result, cleanup) => result.and(cleanup),
    }
}
