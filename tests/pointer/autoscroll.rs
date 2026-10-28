use std::time::{Duration, Instant};

use rev::command::{Command, Motion, Split};
use rev::display::Position;
use rev::session::{Action, Session};

use super::scroll::{editor, lines};
use super::selection::{drag, range, release};

pub(super) const TICK: Duration = Duration::from_millis(100);

pub(super) fn arm(session: &mut Session, now: Instant) {
    session.handle_at(super::focus::press(Position { column: 7, row: 0 }), now);
    session.handle_at(drag(7, 10), now);
}

// An outside drag extends the original canonical mark, pauses on reentry and
// retains the finished region without changing source revision or history.
#[test]
fn edge_lifetime() {
    let mut session = editor(&lines(40));
    let stamp = session.buffer().unwrap().stamp();
    let now = Instant::now();
    arm(&mut session, now);
    assert_eq!(range(&session), None);
    assert_eq!(
        session.expire_input(now + TICK - Duration::from_nanos(1)),
        Action::Wait
    );
    assert_eq!(session.expire_input(now + TICK), Action::Redraw);
    assert_eq!(session.view().top, 3);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        12
    );
    assert_eq!(range(&session), Some(1..97));
    session.handle_at(drag(7, 3), now + TICK);
    let before = session.view().clone();
    assert_eq!(session.expire_input(now + TICK * 20), Action::Wait);
    assert_eq!(session.view(), &before);
    session.handle_at(drag(7, 11), now + TICK * 20);
    session.handle_at(release(7, 11), now + TICK * 21);
    let region = range(&session);
    assert_eq!(session.expire_input(now + TICK * 100), Action::Wait);
    assert_eq!(session.view(), &before);
    assert_eq!(range(&session), region);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// A source below another pane can scroll upward; the pointer's row belongs to
// the neighboring surface but capture and all scrolling remain with its source.
#[test]
fn above_source() {
    let mut session = editor(&lines(40));
    session.execute(Command::Split(Split::Below));
    let source = session.windows().order()[1];
    let peer = session.windows().order()[0];
    session.select_view(source).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.frame().unwrap();
    let other = session.windows().get(peer).unwrap().clone();
    let top = session.view().top;
    let now = Instant::now();
    session.handle_at(super::focus::press(Position { column: 7, row: 6 }), now);
    session.handle_at(drag(7, 5), now);
    assert_eq!(session.expire_input(now + TICK), Action::Redraw);
    assert_eq!(session.view().top, top - 3);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        top - 3
    );
    assert_eq!(session.windows().selected_id(), source);
    assert_eq!(session.windows().get(peer).unwrap(), &other);
    assert!(range(&session).is_some());
}
