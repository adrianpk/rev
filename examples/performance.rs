#![forbid(unsafe_code)]

use std::error::Error;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, Motion, Scroll, Split, WindowCommand};
use rev::session::{Action, Session};
use rev::view::Size;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn require(condition: bool, message: &str) -> Result<()> {
    if !condition {
        return Err(io::Error::other(message).into());
    }
    Ok(())
}

fn emit(name: &str, mut samples: Vec<Duration>) {
    samples.sort();
    let median = samples[samples.len() / 2].as_secs_f64() * 1_000_000.0;
    let maximum = samples.last().unwrap().as_secs_f64() * 1_000_000.0;
    println!(
        "{{\"workload\":\"{name}\",\"samples\":{},\"median_us\":{median:.3},\"max_us\":{maximum:.3}}}",
        samples.len()
    );
}

fn draw(session: &mut Session) -> Result<()> {
    let frame = session.frame()?;
    require(frame.rows().len() == 24, "missing complete frame")?;
    require(frame.cursor().is_some(), "missing cursor")?;
    require(
        frame.rows().iter().any(|row| !row.syntax().is_empty()),
        "highlighting did not execute",
    )?;
    frame.write(&mut io::sink())?;
    Ok(())
}

fn startup(path: &Path) -> Result<Session> {
    let start = Instant::now();
    let mut session = Session::empty(Size {
        width: 80,
        height: 24,
    })?;
    session.load(path)?;
    require(
        session.buffer()?.len_lines() >= 20_000,
        "fixture is too small",
    )?;
    require(session.buffer()?.mode() == "Rust", "fixture was not parsed")?;
    draw(&mut session)?;
    emit("load", vec![start.elapsed()]);
    Ok(session)
}

fn search(session: &mut Session) -> Result<()> {
    let row = session.buffer()?.len_lines() / 2;
    let needle = format!("item_{row}(");
    let mut samples = Vec::new();
    for _ in 0..4 {
        session.execute(Command::Motion(Motion::BufferStart));
        let start = Instant::now();
        session.execute(Command::SearchForward);
        session.handle(Event::Paste(needle.clone()));
        require(
            session.search_match().is_some(),
            "middle-file search failed",
        )?;
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        draw(session)?;
        samples.push(start.elapsed());
    }
    emit("search", samples);
    Ok(())
}

fn editing(session: &mut Session) -> Result<()> {
    let mut insert = Vec::new();
    let mut delete = Vec::new();
    for _ in 0..32 {
        let before = session.buffer()?.len_chars();
        let start = Instant::now();
        require(
            session.insert('é') == Action::Redraw,
            "insert was not dispatched",
        )?;
        draw(session)?;
        insert.push(start.elapsed());
        require(
            session.buffer()?.len_chars() == before + 1,
            "insert did not publish",
        )?;
        require(
            session.buffer()?.syntax().is_incremental(),
            "insert reparsed without its tree",
        )?;
        let start = Instant::now();
        session.execute(Command::BackwardDeleteChar);
        draw(session)?;
        delete.push(start.elapsed());
        require(
            session.buffer()?.len_chars() == before,
            "delete did not publish",
        )?;
        require(
            session.buffer()?.syntax().is_incremental(),
            "delete reparsed without its tree",
        )?;
    }
    emit("insert", insert);
    emit("delete", delete);
    Ok(())
}

fn scrolling(session: &mut Session) -> Result<()> {
    let mut samples = Vec::new();
    let origin = session.view().top;
    for _ in 0..16 {
        let start = Instant::now();
        session.execute(Command::Scroll(Scroll::PageForward));
        draw(session)?;
        samples.push(start.elapsed());
    }
    require(
        session.view().top > origin,
        "scroll did not advance the viewport",
    )?;
    emit("scroll", samples);
    Ok(())
}

fn splitting(session: &mut Session) -> Result<()> {
    let mut samples = Vec::new();
    let stamp = session.buffer()?.stamp();
    for _ in 0..8 {
        let start = Instant::now();
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Split(Split::Below));
        require(
            session.windows().iter().len() == 3,
            "nested shared splits did not execute",
        )?;
        draw(session)?;
        samples.push(start.elapsed());
        require(
            session
                .windows()
                .iter()
                .all(|(_, view)| view.point.stamp == stamp),
            "split stopped sharing the canonical revision",
        )?;
        session.execute(Command::Window(WindowCommand::Keep));
    }
    emit("split_redraw", samples);
    Ok(())
}

fn highlighting(session: &mut Session) -> Result<()> {
    let mut samples = Vec::new();
    let stamp = session.buffer()?.stamp();
    for _ in 0..16 {
        let start = Instant::now();
        draw(session)?;
        samples.push(start.elapsed());
        require(
            session.buffer()?.stamp() == stamp,
            "projection changed document state",
        )?;
    }
    emit("highlighting", samples);
    Ok(())
}

fn run(path: &Path) -> Result<()> {
    let original = std::fs::read(path)?;
    let mut session = startup(path)?;
    search(&mut session)?;
    editing(&mut session)?;
    scrolling(&mut session)?;
    splitting(&mut session)?;
    highlighting(&mut session)?;
    let mut actual = Vec::new();
    rev::file::write(session.buffer()?, &mut actual)?;
    require(
        actual == original,
        "measured commands changed retained fixture bytes",
    )?;
    require(
        std::fs::read(path)? == original,
        "measurement changed the file",
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let path = std::env::args_os().nth(1).ok_or("expected fixture path")?;
    run(Path::new(&path))
}
