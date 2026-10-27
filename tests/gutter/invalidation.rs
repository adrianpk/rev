use crossterm::event::Event;
use rev::command::{Command, History, Motion};
use rev::coordinates::Scalar;
use rev::gutter::{Change, Mode, Projection};
use rev::session::Session;
use rev::view::{Size, View};
use rev::window::Windows;

use super::{Buffers, Composition, area};

fn project(session: &Session) -> Projection {
    let composition = Composition::new(session.windows(), area(0, 0, 80, 24), 0).unwrap();
    composition
        .gutter(
            session.windows(),
            session.buffer().unwrap(),
            session.windows().selected_id(),
        )
        .unwrap()
        .unwrap()
}

// Logical metadata includes the editable empty line and CRLF as one separator;
// tabs, Unicode width and combining clusters do not invent source lines.
#[test]
fn logical_metadata() {
    for (name, source, lines) in [
        ("empty", "", 1),
        ("lf", "\n", 2),
        ("crlf", "\r\n", 2),
        ("unicode", "界\te\u{301}", 1),
        ("mixed", "界\r\n\n", 3),
        ("no final newline", "a\r\nb", 2),
    ] {
        let mut session = super::lifecycle::editor(source);
        session.execute(Command::Motion(Motion::BufferEnd));
        let projection = project(&session);
        assert_eq!(
            (projection.lines, projection.current),
            (lines, lines - 1),
            "{name}"
        );
        assert_eq!(projection.source, session.buffer().unwrap().stamp());
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// A real edit crosses 9999/10000 before a new projection; undo/redo recompute
// coherent widths with fresh source stamps rather than retaining stale geometry.
#[test]
fn edit_invalidation() {
    let mut session = super::lifecycle::editor(&"\n".repeat(9_998));
    session.execute(Command::Motion(Motion::BufferEnd));
    let before = project(&session);
    let saved = session.buffer().unwrap().state();
    let stale = Windows::new(session.view().clone()).unwrap();
    assert_eq!((before.lines, before.geometry.digits()), (9_999, 4));
    session.handle(Event::Paste("\n".to_owned()));
    let edited = project(&session);
    assert_eq!((edited.lines, edited.geometry.digits()), (10_000, 5));
    assert_ne!(edited.source, before.source);
    assert_eq!(edited.current, 9_999);
    assert_eq!(
        edited.geometry.text().column(),
        before.geometry.text().column()
    );
    assert_eq!(
        edited.geometry.text().width() + 1,
        before.geometry.text().width()
    );
    let composition = Composition::new(&stale, area(0, 0, 80, 24), 0).unwrap();
    assert!(
        composition
            .gutter(&stale, session.buffer().unwrap(), stale.selected_id())
            .is_err()
    );
    session.execute(Command::History(History::Undo));
    let undone = project(&session);
    assert_eq!((undone.lines, undone.geometry.digits()), (9_999, 4));
    assert_eq!(undone.geometry, before.geometry);
    assert_eq!(session.buffer().unwrap().state(), saved);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    let redone = project(&session);
    assert_eq!(redone.geometry, edited.geometry);
    assert_ne!(redone.source, edited.source);
    assert_eq!(redone.policy, before.policy);
}

// Relative origin movement changes only the view snapshot; width depends on the
// complete source count and resetting overrides invalidates presentation alone.
#[test]
fn view_invalidation() {
    let mut session = super::lifecycle::editor(&"\n".repeat(100));
    let mut relative = session.view().clone();
    relative.gutter.apply(Change::Mode(Mode::Relative));
    let mut windows = Windows::new(relative).unwrap();
    let id = windows.selected_id();
    let point = windows.selected().point;
    let composition = Composition::new(&windows, area(0, 0, 80, 24), 0).unwrap();
    let first = composition
        .gutter(&windows, session.buffer().unwrap(), id)
        .unwrap()
        .unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    let mut last_view = session.view().clone();
    last_view.gutter.apply(Change::Mode(Mode::Relative));
    let last = Windows::new(last_view).unwrap();
    let last_composition = Composition::new(&last, area(0, 0, 80, 24), 0).unwrap();
    let end = last_composition
        .gutter(&last, session.buffer().unwrap(), last.selected_id())
        .unwrap()
        .unwrap();
    assert_eq!((first.current, end.current), (0, 100));
    assert_eq!(first.geometry, end.geometry);
    assert_eq!(first.source, end.source);
    assert!(windows.change_gutter(id, Change::Reset).unwrap());
    let reset = composition
        .gutter(&windows, session.buffer().unwrap(), id)
        .unwrap()
        .unwrap();
    assert_ne!(reset.policy, first.policy);
    assert!(!reset.policy.visible);
    assert_eq!(reset.geometry.numbers(), None);
    assert_eq!(reset.source, first.source);
    assert_eq!(windows.selected().point, point);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Wrong sources/positions and deleted identities cannot produce labels; tiny
// composition suspends projection while retaining the requested view policy.
#[test]
fn admission_and_resize() {
    let mut session = super::lifecycle::editor("界\tvalue\r\nlast");
    let before = session.view().clone();
    for (width, height, visible) in [(80, 24, true), (2, 2, false), (0, 0, false), (80, 24, true)] {
        session.handle(Event::Resize(width, height));
        let composition =
            Composition::workspace(session.windows(), session.tree(), Size { width, height }, 0);
        let projection = composition.and_then(|c| {
            c.gutter(
                session.windows(),
                session.buffer().unwrap(),
                session.windows().selected_id(),
            )
            .unwrap()
        });
        assert_eq!(projection.is_some(), visible);
        assert_eq!(session.view(), &before);
    }
    rejected_sources(&session);
}

fn rejected_sources(session: &Session) {
    let other = Buffers::new().unwrap();
    let mut view = View::new(session.buffer().unwrap());
    let mut windows = Windows::new(view.clone()).unwrap();
    windows
        .split(
            rev::window::Split::Right,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap();
    let closed = windows.selected_id();
    let composition = Composition::new(&windows, area(0, 0, 80, 24), 0).unwrap();
    assert!(
        composition
            .gutter(
                &windows,
                other.get(other.initial()).unwrap(),
                windows.selected_id()
            )
            .is_err()
    );
    view.point.position = Scalar(session.buffer().unwrap().len_chars() + 1);
    let invalid = Windows::new(view).unwrap();
    let invalid_composition = Composition::new(&invalid, area(0, 0, 80, 24), 0).unwrap();
    assert!(
        invalid_composition
            .gutter(&invalid, session.buffer().unwrap(), invalid.selected_id())
            .is_err()
    );
    assert!(
        composition
            .gutter(&windows, session.buffer().unwrap(), invalid.selected_id())
            .is_err()
    );
    windows.delete_selected().unwrap();
    assert!(
        composition
            .gutter(&windows, session.buffer().unwrap(), closed)
            .is_err()
    );
}
