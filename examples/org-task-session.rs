#![forbid(unsafe_code)]

//! Owned read-only task fixture using the production session event loop.
//! The normal executable has no read-only startup option; this host configures
//! the existing canonical buffer admission flag before handing it to Session.
use std::error::Error;
use std::io;
use std::path::PathBuf;

use rev::buffer::Buffers;
use rev::config::Startup;
use rev::session::Session;
use rev::terminal::Terminal;
use rev::view::{Size, View};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let path = PathBuf::from(arguments.next().ok_or("expected owned Org source path")?);
    if arguments.next().is_some() {
        return Err("expected one owned Org source path".into());
    }
    let mut session = readonly(&path)?;
    let mut terminal = Terminal::acquire(io::stdout())?;
    let result = terminal
        .enter()
        .and_then(|()| session.run_terminal(&mut terminal));
    let cleanup = terminal.restore();
    result.and(cleanup)?;
    Ok(())
}

fn readonly(path: &std::path::Path) -> Result<Session, Box<dyn Error>> {
    let startup = Startup::provision();
    if let Some(diagnostic) = startup.diagnostic {
        return Err(diagnostic.to_string().into());
    }
    let mut buffers = Buffers::with_settings(startup.snapshot.clone())?;
    let id = buffers.visit(path)?;
    buffers.get_mut(id)?.set_readonly(true);
    let view = View::with_settings(buffers.get(id)?, &startup.snapshot);
    let (width, height) = crossterm::terminal::size()?;
    let session = Session::new(buffers, view, Size { width, height })?;
    Ok(session)
}
