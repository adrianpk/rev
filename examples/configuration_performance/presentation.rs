use std::io;
use std::ops::ControlFlow;
use std::path::Path;
use std::time::Instant;

use rev::buffer::Buffers;
use rev::buffer::edit::{History, Replacement};
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

pub(super) fn redraw(root: &Path) -> Result<()> {
    let mut session = session(root)?;
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
    let stamp = session.buffer()?.stamp();
    let state = session.buffer()?.state();
    barrier()?;
    let mut samples = Vec::new();
    for _ in 0..64 {
        let start = Instant::now();
        let frame = session.frame()?;
        frame.write(&mut io::sink())?;
        samples.push(start.elapsed());
        require(
            frame
                .rows()
                .iter()
                .map(|row| row.text())
                .eq(rows.iter().map(String::as_str)),
            "retained panel canvas changed",
        )?;
        require(frame.cursor().is_some(), "prompt cursor missing")?;
    }
    require(
        (session.buffer()?.stamp(), session.buffer()?.state()) == (stamp, state),
        "draw mutated document",
    )?;
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

pub(super) fn syntax(root: &Path) -> Result<()> {
    let mut buffers = Buffers::with_settings(graph::settings(root)?)?;
    let id = buffers.visit(&root.join("work/cutoff.rs"))?;
    let buffer = buffers.get_mut(id)?;
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
    barrier()?;
    let mut samples = Vec::new();
    for _ in 0..32 {
        let start = Instant::now();
        require(
            progress(buffer) == 0 && buffer.mode() == "Text",
            "oversize parser executed",
        )?;
        samples.push(start.elapsed());
    }
    buffer.navigate(History::Undo, &mut [], None)?;
    let admitted = progress(buffer);
    require(
        admitted > 0 && buffer.mode() == "Rust",
        "below-cutoff positive parser control missing",
    )?;
    require(progress(buffer) == 0, "unchanged parser repeated work")?;
    require(!buffer.is_modified(), "cutoff undo lost saved identity")?;
    emit(
        "syntax",
        samples,
        &format!(
            "\"cutoff_bytes\":65536,\"oversize_callbacks\":0,\"unchanged_callbacks\":0,\"admitted_callbacks\":{admitted}"
        ),
    )
}
