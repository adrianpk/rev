use std::io;
use std::ops::ControlFlow;
use std::path::Path;
use std::time::{Duration, Instant};

use rev::buffer::edit::{History, Replacement};
use rev::buffer::{Buffer, BufferId, Buffers};
use rev::command::{Command, Split, TreeCommand};
use rev::session::Session;
use rev::view::{Size, View};
use tree_sitter::{ParseOptions, ParseState};

use super::{Result, barrier, emit, graph, require};

fn session(root: &Path) -> Result<Session> {
    let mut buffers = Buffers::with_settings(graph::settings(root)?)?;
    let id = buffers.visit(&root.join("work/main.rs"))?;
    let view = View::new(buffers.get(id)?);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 30,
        },
    )?;
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Split(Split::Below));
    session.execute(Command::Tree(TreeCommand::Focus));
    session.execute(Command::Tree(TreeCommand::Expand));
    session.execute(Command::KeyboardQuit);
    session.execute(Command::FindFile);
    Ok(session)
}

fn canvas(session: &mut Session) -> Result<Vec<String>> {
    let frame = session.frame()?;
    let rows: Vec<_> = frame
        .rows()
        .iter()
        .map(|row| row.text().to_owned())
        .collect();
    require(
        rows.iter().any(|row| row.contains('界')),
        "Unicode panel fixture missing",
    )?;
    require(
        frame.rows().iter().any(|row| !row.syntax().is_empty()),
        "visible syntax missing",
    )?;
    require(
        session.windows().iter().len() == 3,
        "nested shared panes missing",
    )?;
    Ok(rows)
}

fn draw(session: &mut Session, rows: &[String]) -> Result<Duration> {
    let start = Instant::now();
    let frame = session.frame()?;
    frame.write(&mut io::sink())?;
    let elapsed = start.elapsed();
    require(
        frame
            .rows()
            .iter()
            .map(|row| row.text())
            .eq(rows.iter().map(String::as_str)),
        "retained panel canvas changed",
    )?;
    require(frame.cursor().is_some(), "prompt cursor missing")?;
    Ok(elapsed)
}

fn identity(session: &Session) -> Result<(rev::buffer::Stamp, rev::buffer::State)> {
    let buffer = session.buffer()?;
    Ok((buffer.stamp(), buffer.state()))
}

pub(super) fn redraw(root: &Path) -> Result<()> {
    let mut session = session(root)?;
    let rows = canvas(&mut session)?;
    let before = identity(&session)?;
    barrier()?;
    let mut samples = Vec::new();
    for _ in 0..64 {
        samples.push(draw(&mut session, &rows)?);
    }
    require(identity(&session)? == before, "draw mutated document")?;
    emit(
        "redraw",
        samples,
        "\"panes\":3,\"columns\":100,\"rows\":30,\"frames\":64",
    )
}

fn progress(buffer: &mut rev::buffer::Buffer) -> usize {
    let mut calls = 0;
    let mut callback = |_: &ParseState| {
        calls += 1;
        ControlFlow::Continue(())
    };
    buffer.parse_with(ParseOptions::new().progress_callback(&mut callback));
    calls
}

fn cutoff_buffer(root: &Path) -> Result<(Buffers, BufferId)> {
    let mut buffers = Buffers::with_settings(graph::settings(root)?)?;
    let id = buffers.visit(&root.join("work/cutoff.rs"))?;
    Ok((buffers, id))
}

fn exceed(buffer: &mut Buffer) -> Result<()> {
    require(
        buffer.len_bytes() == 65536 && buffer.mode() == "Rust",
        "exact cutoff not admitted",
    )?;
    let change = Replacement {
        source: buffer.span(buffer.len_chars()..buffer.len_chars())?,
        text: "x".into(),
    };
    let prepared = buffer.prepare(&[change], &[], None)?;
    buffer.publish(prepared, &mut [])?;
    Ok(())
}

fn oversize(buffer: &mut Buffer) -> Result<Vec<Duration>> {
    let mut samples = Vec::new();
    for _ in 0..32 {
        let start = Instant::now();
        require(
            progress(buffer) == 0 && buffer.mode() == "Text",
            "oversize parser executed",
        )?;
        samples.push(start.elapsed());
    }
    Ok(samples)
}

fn restore(buffer: &mut Buffer) -> Result<usize> {
    buffer.navigate(History::Undo, &mut [], None)?;
    let admitted = progress(buffer);
    require(
        admitted > 0 && buffer.mode() == "Rust",
        "below-cutoff positive parser control missing",
    )?;
    require(progress(buffer) == 0, "unchanged parser repeated work")?;
    require(!buffer.is_modified(), "cutoff undo lost saved identity")?;
    Ok(admitted)
}

pub(super) fn syntax(root: &Path) -> Result<()> {
    let (mut buffers, id) = cutoff_buffer(root)?;
    let buffer = buffers.get_mut(id)?;
    exceed(buffer)?;
    barrier()?;
    let samples = oversize(buffer)?;
    let admitted = restore(buffer)?;
    emit(
        "syntax",
        samples,
        &format!(
            "\"cutoff_bytes\":65536,\"oversize_callbacks\":0,\"unchanged_callbacks\":0,\"admitted_callbacks\":{admitted}"
        ),
    )
}
