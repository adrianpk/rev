#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use rev::buffer::Buffers;
use rev::command::{Command, Split};
use rev::coordinates::Scalar;
use rev::session::{Action, Session};
use rev::view::{Size, View};
use rev::window::Layout;
use unicode_width::UnicodeWidthStr;

fn editor(text: &str, size: Size) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(text).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = Scalar(text.chars().count().min(2));
    view.column = Some(5);
    Session::new(buffers, view, size).unwrap()
}

fn size(width: u16, height: u16) -> Size {
    Size { width, height }
}

fn split(session: &mut Session, direction: Split) {
    assert_eq!(session.execute(Command::Split(direction)), Action::Redraw);
    assert_eq!(session.echo(), "");
}

fn bounded(session: &mut Session, size: Size) {
    let frame = session.frame().unwrap();
    let expected = if size.width == 0 {
        0
    } else {
        usize::from(size.height)
    };
    assert_eq!(frame.rows().len(), expected);
    for row in frame.rows() {
        assert_eq!(row.text().width(), usize::from(size.width));
        assert!(row.row() < size.height);
        assert!(!row.text().contains('\x1b'));
    }
    if let Some(cursor) = frame.cursor() {
        assert!(cursor.column < size.width && cursor.row < size.height);
    }
}

// Each split allocates one distinct view, copies its complete initial navigation
// state, shares the canonical buffer and retains the original selected identity.
#[test]
fn split_state() {
    for direction in [Split::Below, Split::Right] {
        let dimensions = size(31, 13);
        let mut session = editor("abcdef\nsecond", dimensions);
        let original = session.windows().selected_id();
        let before = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        split(&mut session, direction);
        assert_eq!(session.windows().selected_id(), original);
        assert_eq!(session.windows().iter().len(), 2);
        for (_, view) in session.windows().iter() {
            assert_eq!(*view, before);
            assert_eq!(view.point.stamp, stamp);
        }
        let ids = session
            .windows()
            .iter()
            .map(|(id, _)| id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), 2);
        bounded(&mut session, dimensions);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Below/right in either order produces the requested nested geometry, with
// separate status rows, a right divider and one reserved global echo row.
#[test]
fn mixed_layout() {
    for first in [Split::Below, Split::Right] {
        let mut session = editor("abc", size(31, 13));
        let original = session.windows().selected_id();
        split(&mut session, first);
        let second = if first == Split::Below {
            Split::Right
        } else {
            Split::Below
        };
        split(&mut session, second);
        assert_eq!(session.windows().selected_id(), original);
        let Layout::Split {
            direction,
            first: child,
            ..
        } = session.windows().layout()
        else {
            panic!("root split");
        };
        assert_eq!(*direction, first);
        assert!(matches!(**child, Layout::Split { .. }));
        let geometry = session.windows().geometry(size(31, 13)).unwrap();
        let actual = geometry
            .panes()
            .iter()
            .map(|(_, a)| (a.column(), a.row(), a.width(), a.height()))
            .collect::<Vec<_>>();
        let expected = if first == Split::Below {
            vec![(0, 0, 15, 6), (16, 0, 15, 6), (0, 6, 31, 6)]
        } else {
            vec![(0, 0, 15, 6), (0, 6, 15, 6), (16, 0, 15, 12)]
        };
        assert_eq!(actual, expected);
        assert_eq!(geometry.borders().len(), 1);
        assert_eq!(session.windows().iter().len(), 3);
        bounded(&mut session, size(31, 13));
    }
}

// A normal pane requires three text columns and text+status rows. Admission
// includes dividers/echo and rejects before changing layout, focus or view state.
#[test]
fn fit_admission() {
    let cases = [
        ("below exact", 3, 5, Split::Below, true),
        ("below short", 3, 4, Split::Below, false),
        ("right exact", 7, 3, Split::Right, true),
        ("right narrow", 6, 3, Split::Right, false),
        ("narrow text", 2, 8, Split::Below, false),
        ("short text", 20, 2, Split::Right, false),
        ("zero", 0, 0, Split::Below, false),
        ("maximum", u16::MAX, u16::MAX, Split::Right, true),
    ];
    for (name, width, height, direction, fits) in cases {
        let mut session = editor("text", size(width, height));
        let original = session.windows().selected_id();
        let layout = session.windows().layout().clone();
        let before = session.view().clone();
        session.execute(Command::Split(direction));
        assert_eq!(session.windows().selected_id(), original, "{name}");
        assert_eq!(*session.view(), before, "{name}");
        if fits {
            assert_eq!(session.windows().iter().len(), 2, "{name}");
        } else {
            assert_eq!(session.windows().iter().len(), 1, "{name}");
            assert_eq!(*session.windows().layout(), layout, "{name}");
            assert!(session.echo().starts_with("Split does not fit"));
        }
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
