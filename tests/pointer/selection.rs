use std::ops::Range;

use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::coordinates::Scalar;
use rev::display::Position;
use rev::session::{Action, Session};

use super::scroll::editor;

pub(super) fn gesture(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

pub(super) fn drag(column: u16, row: u16) -> Event {
    gesture(MouseEventKind::Drag(MouseButton::Left), column, row)
}

pub(super) fn release(column: u16, row: u16) -> Event {
    gesture(MouseEventKind::Up(MouseButton::Left), column, row)
}

pub(super) fn range(session: &Session) -> Option<Range<usize>> {
    session
        .buffer()
        .unwrap()
        .region(session.view().point)
        .unwrap()
        .map(|span| span.range)
}

// Both directions use the existing canonical region. Release can update the
// last checked endpoint and ends capture without editing text or losing mark.
#[test]
fn linear_drag() {
    for (start, target, end, anchor, expected) in [
        ((7, 0), (9, 1), (8, 1), 1, 1..8),
        ((9, 1), (7, 0), (8, 0), 9, 2..9),
    ] {
        let mut session = editor("alpha\nbeta\n");
        let stamp = session.buffer().unwrap().stamp();
        session.handle(super::focus::press(Position {
            column: start.0,
            row: start.1,
        }));
        assert_eq!(range(&session), None);
        assert_eq!(session.handle(drag(target.0, target.1)), Action::Redraw);
        assert_eq!(range(&session), Some(1..9));
        session.handle(release(end.0, end.1));
        assert_eq!(range(&session), Some(expected.clone()));
        assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(anchor)));
        let point = session.view().point;
        assert_eq!(session.handle(drag(6, 0)), Action::Wait);
        assert_eq!(session.view().point, point);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(session.buffer().unwrap().history().is_empty());
        let mut bytes = Vec::new();
        rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
        assert_eq!(bytes, b"alpha\nbeta\n");
        assert!(
            session.frame().unwrap().rows()[0]
                .selection()
                .unwrap()
                .start
                >= 6
        );
    }
}

// A click, a same-cell motion and its release cannot fabricate an empty region.
// A real drag may subsequently shrink back to its anchor through normal motion.
#[test]
fn click_boundary() {
    let mut session = editor("alpha\nbeta\n");
    session.handle(super::focus::press(Position { column: 7, row: 0 }));
    session.handle(drag(7, 0));
    session.handle(release(9, 1));
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(session.buffer().unwrap().mark(), None);
    session.handle(super::focus::press(Position { column: 8, row: 0 }));
    session.handle(drag(10, 0));
    session.handle(drag(8, 0));
    session.handle(release(8, 0));
    assert_eq!(range(&session), Some(2..2));
    assert!(session.buffer().unwrap().history().is_empty());
}
