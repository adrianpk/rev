use std::io;
use std::time::{Duration, Instant};

use rev::command::{Command, GutterCommand, Motion, Scroll};
use rev::session::Session;

use super::{Result, emit, require};

fn draw(session: &mut Session) -> Result<Duration> {
    let started = Instant::now();
    session.execute(Command::Scroll(Scroll::PreviousLine));
    let frame = session.frame()?;
    frame.write(&mut io::sink())?;
    let elapsed = started.elapsed();
    let numbers = frame
        .rows()
        .iter()
        .filter(|row| row.text().contains('│'))
        .count();
    require(numbers == 22, "visible gutter row count changed")?;
    require(
        frame.rows().len() == 24,
        "projection grew with document lines",
    )?;
    Ok(elapsed)
}

fn measure(session: &mut Session, name: &str, command: GutterCommand) -> Result<()> {
    session.execute(Command::Gutter(command));
    session.execute(Command::Motion(Motion::BufferEnd));
    let mut samples = Vec::new();
    for _ in 0..64 {
        samples.push(draw(session)?);
    }
    emit(
        name,
        samples,
        "\"logical_lines\":100001,\"frames\":64,\"number_rows_per_frame\":22,\"viewport_rows\":22",
    )
}

pub(super) fn run(session: &mut Session) -> Result<()> {
    measure(session, "gutter_absolute", GutterCommand::Absolute)?;
    measure(session, "gutter_relative", GutterCommand::Relative)
}
