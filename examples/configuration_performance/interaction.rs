#[path = "interaction/gutter.rs"]
mod gutter;
#[path = "interaction/pointer.rs"]
mod pointer;

use std::path::Path;

use rev::buffer::Buffers;
use rev::command::{Command, TreeCommand};
use rev::session::Session;
use rev::view::{Size, View};

use super::{Result, barrier, emit, graph, require};

fn session(root: &Path) -> Result<Session> {
    let settings = graph::settings(root)?;
    let mut buffers = Buffers::with_settings(settings.clone())?;
    let id = buffers.visit(&root.join("work/large.txt"))?;
    let view = View::with_settings(buffers.get(id)?, &settings);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )?;
    session.execute(Command::Tree(TreeCommand::Toggle));
    require(
        session.buffer()?.len_lines() == 100001,
        "large line index missing",
    )?;
    Ok(session)
}

pub(super) fn run(root: &Path, kind: &str) -> Result<()> {
    let mut session = session(root)?;
    let stamp = session.buffer()?.stamp();
    barrier()?;
    match kind {
        "gutter" => gutter::run(&mut session)?,
        "drag" => pointer::run(&mut session)?,
        _ => return Err("unknown interaction workload".into()),
    }
    unchanged(&session, stamp)
}

fn unchanged(session: &Session, stamp: rev::buffer::Stamp) -> Result<()> {
    let buffer = session.buffer()?;
    require(buffer.stamp() == stamp, "interaction changed source")?;
    require(buffer.history().is_empty(), "interaction changed history")
}
