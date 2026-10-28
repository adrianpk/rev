use super::*;
use rev::{
    command::{Command, OrgCommand},
    session::Session,
    view::Size,
};
/// A real native first-frame output failure must clear the armed timer before
/// returning control to the existing terminal restoration owner.
pub(super) fn failure(terminal: &mut Terminal<Fault>) -> io::Result<()> {
    let mut session = Session::empty(Size {
        width: 80,
        height: 24,
    })
    .map_err(io::Error::other)?;
    session.execute(Command::Org(OrgCommand::TimerSet));
    session.handle(crossterm::event::Event::Paste("0:00:03".into()));
    session.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ),
    ));
    if session.remaining_input(std::time::Instant::now()).is_none() {
        return Err(io::Error::other("timer fixture did not arm"));
    }
    let result = session.run_terminal(terminal);
    if session.remaining_input(std::time::Instant::now()).is_some() {
        return Err(io::Error::other("timer survived native output failure"));
    }
    eprintln!("timer-owner-clean");
    result
}

pub(super) fn pattern(mode: &str) -> Option<&'static [u8]> {
    (mode == "org-timer-io").then_some(b"T-")
}
