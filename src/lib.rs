#![forbid(unsafe_code)]

pub mod buffer;
pub mod command;
pub mod coordinates;
pub mod input;
pub mod terminal;

use std::io::{self, Write};

use command::Command;
use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event,
    terminal::{Clear, ClearType},
};
use input::{Input, Outcome};
use terminal::Terminal;

/// Runs the terminal session and reports errors after attempting restoration.
pub fn run() -> io::Result<()> {
    let mut terminal = Terminal::acquire(io::stdout())?;
    let result = terminal.enter().and_then(|()| session(&mut terminal));
    let cleanup = terminal.restore();
    match (result, cleanup) {
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; terminal restoration: {cleanup}"),
        )),
        (result, cleanup) => result.and(cleanup),
    }
}

fn session(terminal: &mut Terminal<impl Write>) -> io::Result<()> {
    terminal
        .writer()
        .write_all(b"\x1b[2J\x1b[Hrev\r\nC-x C-c: exit\r\n")?;
    terminal.writer().flush()?;
    let mut input = Input::default();
    loop {
        let outcome = input.handle(event::read()?);
        if outcome == Outcome::Command(Command::Quit) {
            return Ok(());
        }
        feedback(terminal.writer(), &outcome)?;
    }
}

fn feedback(writer: &mut impl Write, outcome: &Outcome) -> io::Result<()> {
    let message = match outcome {
        Outcome::Prefix(prefix) => prefix.label(),
        Outcome::Message(message) => message,
        Outcome::Command(Command::KeyboardQuit) => "Quit",
        _ => return Ok(()),
    };
    writer
        .queue(MoveTo(0, 2))?
        .queue(Clear(ClearType::CurrentLine))?;
    writer.write_all(message.as_bytes())?;
    writer.flush()
}
