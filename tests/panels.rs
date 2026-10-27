#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::command::{Command, Split};
use rev::display::Position;
use rev::panel::{Composition, Panel};
use rev::session::Session;
use rev::view::{Size, View};
use rev::window::Area;

fn area(column: u16, row: u16, width: u16, height: u16) -> Area {
    Area::new(column, row, width, height).unwrap()
}

fn position(column: u16, row: u16) -> Position {
    Position { column, row }
}

fn editor() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\t界e\u{301}\r\nlast").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

fn nested(first: Split, second: Split, peer: bool) -> Session {
    let mut session = editor();
    session.execute(Command::Split(first));
    if peer {
        session.select_view(session.windows().order()[1]).unwrap();
    }
    session.execute(Command::Split(second));
    session
}

// Status consumes exactly the final row of the supplied outer bounds. Empty
// surfaces remain representable and cannot produce a document cursor.
#[test]
fn panel_bounds() {
    for (width, height, reserve, rows) in [
        (7, 4, true, 3),
        (7, 4, false, 4),
        (1, 1, true, 0),
        (0, 4, true, 3),
        (7, 0, true, 0),
    ] {
        let outer = area(4, 3, width, height);
        let panel = Panel::new(outer, reserve, true);
        assert_eq!(panel.outer(), outer);
        assert_eq!(panel.content(), area(4, 3, width, rows));
        assert_eq!(
            panel.status(),
            (reserve && height > 0).then(|| area(4, 3 + rows, width, 1))
        );
        assert!(panel.active());
        assert!(!Panel::new(outer, reserve, false).active());
        assert_eq!(
            panel.position(position(0, 0)).is_some(),
            width > 0 && rows > 0
        );
    }
}

// Translation and clipping use content bounds rather than outer/status bounds.
// Offscreen, reversed and empty spans never escape through native arithmetic.
#[test]
fn cell_clipping() {
    let panel = Panel::new(area(4, 3, 6, 3), true, false);
    assert_eq!(panel.position(position(5, 1)), Some(position(9, 4)));
    assert_eq!(panel.position(position(6, 0)), None);
    assert_eq!(panel.position(position(0, 2)), None);
    let cases = [
        (0, 0..6, Some((3, 4..10))),
        (1, 2..u16::MAX, Some((4, 6..10))),
        (2, 0..3, None),
        (0, 6..9, None),
        (0, 3..3, None),
        (0, 5..2, None),
    ];
    for (row, columns, expected) in cases {
        assert_eq!(panel.clip(row, columns), expected);
    }
}

// Half-open endpoints admit native maxima without wrapping. Empty endpoints
// are valid bounds, but a real cell beyond either axis is rejected.
#[test]
fn native_bounds() {
    assert!(Area::new(1, 0, u16::MAX, 1).is_none());
    assert!(Area::new(0, 1, 1, u16::MAX).is_none());
    assert!(Area::new(u16::MAX, u16::MAX, 0, 0).is_some());
    let panel = Panel::new(area(0, 0, u16::MAX, u16::MAX), true, false);
    assert_eq!(
        panel.position(position(u16::MAX - 1, u16::MAX - 2)),
        Some(position(u16::MAX - 1, u16::MAX - 2))
    );
    assert_eq!(panel.position(position(0, u16::MAX - 1)), None);
}

// Both nesting directions and branches meet the common separator rather than
// painting independent pane frames. Endpoints stay straight at the outer edge.
#[test]
fn shared_junctions() {
    for (first, second, peer, junction) in [
        (Split::Right, Split::Below, false, '┤'),
        (Split::Right, Split::Below, true, '├'),
        (Split::Below, Split::Right, false, '┴'),
        (Split::Below, Split::Right, true, '┬'),
    ] {
        let session = nested(first, second, peer);
        let composition = Composition::new(session.windows(), area(0, 0, 15, 13), 0).unwrap();
        assert_eq!(composition.separator(position(7, 5)), Some(junction));
        assert_eq!(composition.separator(position(0, 0)), None);
        assert_eq!(composition.separator(position(14, 11)), None);
        assert_eq!(composition.separator(position(7, 12)), None);
        assert_eq!(composition.borders().len(), 2);
    }
}

// Aligned child boundaries form one crossing at the parent separator. The
// compact spans leave all status and prompt cells outside separator ownership.
#[test]
fn cross_junction() {
    let mut session = editor();
    session.execute(Command::Split(Split::Right));
    let right = session.windows().order()[1];
    session.execute(Command::Split(Split::Below));
    session.select_view(right).unwrap();
    session.execute(Command::Split(Split::Below));
    let composition = Composition::new(session.windows(), area(0, 0, 15, 13), 0).unwrap();
    assert_eq!(composition.separator(position(7, 5)), Some('┼'));
    assert_eq!(composition.separator(position(7, 0)), Some('│'));
    assert_eq!(composition.separator(position(0, 5)), Some('─'));
    assert_eq!(composition.borders().len(), 3);
    assert_eq!(composition.panels().len(), 4);
}

// Completion content and its separator use only surplus rows after the full
// editing minimum and prompt. A one-row surplus cannot steal an editing row.
#[test]
fn auxiliary_rows() {
    let session = editor();
    for (height, requested, rows, editing) in [
        (3, 8, 0, 2),
        (4, 8, 0, 3),
        (5, 8, 1, 2),
        (8, 2, 2, 4),
        (8, 0, 0, 7),
        (u16::MAX, u16::MAX, u16::MAX - 4, 2),
    ] {
        let composition =
            Composition::new(session.windows(), area(2, 0, 15, height), requested).unwrap();
        assert_eq!(composition.panels()[0].1.outer(), area(2, 0, 15, editing));
        assert_eq!(composition.prompt(), area(2, height - 1, 15, 1));
        assert_eq!(
            composition.auxiliary().map(|panel| panel.content()),
            (rows > 0).then(|| area(2, editing + 1, 15, rows))
        );
        assert_eq!(composition.borders().len(), usize::from(rows > 0));
    }
}
