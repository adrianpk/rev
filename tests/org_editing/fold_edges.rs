use rev::buffer::edit::Replacement;
use rev::command::History;
use rev::coordinates::Cell;
use rev::display::{Frame, Position};
use rev::window::Area;

use super::folds::{buffer, range};
use super::*;

// Text publication, undo and redo release obsolete offsets in every shared view.
// A native edit into hidden source ends at the real canonical target, with one
// history record and preserved original line separators.
#[test]
fn edit_reveal() {
    let source = "* A\r\nhidden\r\n* B\r\n";
    let (buffers, id) = buffer(source);
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.set_folds(buffer, &[range(buffer, 1..2)]).unwrap();
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 10,
        },
    )
    .unwrap();
    session.execute(Command::Split(Split::Right));
    session.replace(5..11, "changed").unwrap();
    assert_eq!(text(&session), "* A\r\nchanged\r\n* B\r\n");
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        Cell { row: 1, column: 7 }
    );
    for command in [None, Some(History::Undo), Some(History::Redo)] {
        if let Some(history) = command {
            session.execute(Command::History(history));
        }
        session.frame().unwrap();
        let buffer = session.buffer().unwrap();
        for (_, view) in session.windows().iter() {
            assert_eq!(view.point.stamp, buffer.stamp());
            assert_eq!(view.visible_lines(buffer), buffer.len_lines());
        }
    }
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Raw source changes invalidate the view point and fold authority separately.
// Retiring a stale point does not silently requalify old fold ranges; only fresh
// checked source may replace them, and rejected requests leave metadata intact.
#[test]
fn stale_folds() {
    let (mut buffers, id) = buffer("* A\nbody\n* B\n");
    let buffer = buffers.get_mut(id).unwrap();
    let old = range(buffer, 1..2);
    let mut view = View::new(buffer);
    view.set_folds(buffer, std::slice::from_ref(&old)).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "!".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(view.source_row(buffer, 0), None);
    assert_eq!(view.screen_row(buffer, 0), None);
    assert!(
        Frame::project(
            buffer,
            &view,
            Size {
                width: 40,
                height: 8
            },
            ""
        )
        .is_err()
    );
    view.point.stamp = buffer.stamp();
    let before = view.clone();
    assert!(view.set_folds(buffer, &[old]).is_err());
    assert_eq!(view, before);
    assert_eq!(view.visible_lines(buffer), buffer.len_lines());
    assert_eq!(view.source_row(buffer, 1), Some(1));
    view.reveal(
        buffer,
        Size {
            width: 0,
            height: 0,
        },
    )
    .unwrap();
    assert!(!view.clear_folds());
}

// Reveal/cancel in one split keeps another view's requested visibility intact.
// Read-only editing failure cannot consume source or rewrite a fold snapshot.
#[test]
fn peer_visibility() {
    let (mut buffers, id) = buffer("* A\nsecret\n* B\nlast\n");
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(true);
    let mut view = View::new(buffer);
    view.set_folds(buffer, &[range(buffer, 1..2)]).unwrap();
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("secret".into()));
    session.frame().unwrap();
    let buffer = session.buffer().unwrap();
    assert_eq!(session.view().visible_lines(buffer), 5);
    assert_eq!(
        session.windows().get(peer).unwrap().visible_lines(buffer),
        4
    );
    session.execute(Command::KeyboardQuit);
    let before = session.view().clone();
    assert!(session.replace(4..10, "changed").is_err());
    assert_eq!(session.view(), &before);
    assert_eq!(text(&session), "* A\nsecret\n* B\nlast\n");
    assert!(session.buffer().unwrap().history().is_empty());
}

// Suspended/tiny geometry retains canonical scroll and visibility. Usable
// resize reveals the same logical point rather than a neighboring visible row.
#[test]
fn tiny_projection() {
    for (width, height) in [(0, 0), (2, 3), (3, 3), (4, 4), (10, 4), (40, 8)] {
        let (buffers, id) = buffer("* A\nbody\n* 界e\u{301}\nlast");
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.point.position = rev::coordinates::Coordinates::new(buffer)
            .unwrap()
            .from_cell(Cell { row: 2, column: 2 })
            .unwrap();
        view.top = 2;
        view.set_folds(buffer, &[range(buffer, 1..2)]).unwrap();
        let size = Size { width, height };
        view.reveal(buffer, size).unwrap();
        if !size.usable() {
            assert_eq!(view.top, 2);
        }
        assert_eq!(view.visible_lines(buffer), 3);
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        for row in frame.rows() {
            assert!(row.row() < height);
        }
        if let Some(cursor) = frame.cursor() {
            assert!(cursor.row < height && cursor.column < width);
        }
        view.reveal(
            buffer,
            Size {
                width: 40,
                height: 8,
            },
        )
        .unwrap();
        assert_eq!(view.cell(buffer).unwrap().row, 2);
        assert_eq!(
            Frame::project(
                buffer,
                &view,
                Size {
                    width: 40,
                    height: 8
                },
                ""
            )
            .unwrap()
            .cursor()
            .unwrap()
            .row,
            0
        );
    }
}

// Clipped wide/ZWJ edges have no mouse target, while complete combining/tab
// glyphs hit their original scalar boundary on the projected source row.
#[test]
fn unicode_hits() {
    let (buffers, id) = buffer("* A\r\nhidden\r\n* 界e\u{301}\t👩\u{200d}💻 tail\r\nlast");
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.set_folds(buffer, &[range(buffer, 1..2)]).unwrap();
    view.top = 2;
    view.left = 3;
    let coordinates = rev::coordinates::Coordinates::new(buffer).unwrap();
    for (name, column, logical) in [
        ("wide edge", 0, None),
        ("combining", 1, Some(4)),
        ("tab", 3, Some(5)),
        ("ZWJ edge", 5, None),
    ] {
        let point = view
            .hit(
                buffer,
                Area::new(0, 0, 6, 2).unwrap(),
                Position { row: 0, column },
            )
            .unwrap();
        let expected =
            logical.map(|column| coordinates.from_cell(Cell { row: 2, column }).unwrap());
        assert_eq!(point.map(|point| point.position), expected, "{name}");
    }
    assert_eq!(view.screen_row(buffer, 1), None);
}

// The maximum interval table remains a sparse map, not a document-row cache.
// Every boundary round-trips and every hidden row lacks a screen alias.
#[test]
fn maximum_map() {
    let (buffers, id) = buffer(&"row\n".repeat(8192));
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    let hidden: Vec<_> = (0..4096)
        .map(|n| range(buffer, n * 2 + 1..n * 2 + 2))
        .collect();
    view.set_folds(buffer, &hidden).unwrap();
    assert_eq!(view.visible_lines(buffer), 4097);
    for local in 0..4097 {
        assert_eq!(view.source_row(buffer, local), Some(local * 2));
        assert_eq!(view.screen_row(buffer, local * 2), Some(local));
        assert_eq!(view.screen_row(buffer, local * 2 + 1), None);
    }
}
