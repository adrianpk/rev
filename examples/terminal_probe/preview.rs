use super::*;
use rev::{
    command::{Command, OrgCommand},
    session::Session,
    view::Size,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
pub(super) fn failure(terminal: &mut Terminal<Fault>, mode: &str) -> io::Result<()> {
    let source = std::env::args()
        .nth(2)
        .ok_or_else(|| io::Error::other("preview fixture path is required"))?;
    let mut session = Session::empty(Size {
        width: 80,
        height: 24,
    })
    .map_err(io::Error::other)?;
    session.load(Path::new(&source)).map_err(io::Error::other)?;
    settle(&mut session)?;
    session.execute(Command::Org(OrgCommand::LinkPreview));
    settle(&mut session)?;
    let frame = session.frame().map_err(io::Error::other)?;
    if frame.previews().len() != 1 {
        return Err(io::Error::other(format!(
            "preview fixture: {}",
            session.echo()
        )));
    }
    terminal.draw(&frame)?;
    if mode == "org-preview-unwind" {
        panic!("injected preview unwind");
    }
    terminal.restore()
}
fn settle(session: &mut Session) -> io::Result<()> {
    let end = Instant::now() + Duration::from_secs(2);
    loop {
        session.frame().map_err(io::Error::other)?;
        if session.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        if Instant::now() >= end {
            return Err(io::Error::other("preview fixture deadline"));
        }
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}

pub(super) fn pattern(mode: &str) -> Option<&'static [u8]> {
    (mode == "org-preview-io").then_some(b"a=d,d=I")
}
