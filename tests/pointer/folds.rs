use std::time::{Duration, Instant};

use rev::buffer::{Buffer, Buffers, Mode};
use rev::coordinates::{Byte, Coordinates, LineByte};
use rev::org::{Source, SourceRange};
use rev::session::{Action, Session};
use rev::view::{Size, View};

use super::selection::{drag, gesture, release};
use super::*;

fn rows(buffer: &Buffer, start: usize, end: usize) -> SourceRange {
    let at = |row| {
        if row == buffer.len_lines() {
            return buffer.len_bytes();
        }
        let coordinates = Coordinates::new(buffer).unwrap();
        coordinates
            .to_byte(coordinates.from_line(LineByte { row, column: 0 }).unwrap())
            .unwrap()
            .0
    };
    Source::new(buffer)
        .range(Byte(at(start))..Byte(at(end)))
        .unwrap()
}

fn editor(edges: bool) -> Session {
    let source = (0..40)
        .map(|n| format!("* Row {n} 界e\u{301}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    let ranges = if edges {
        view.point.position = Coordinates::new(buffer)
            .unwrap()
            .from_line(LineByte { row: 20, column: 0 })
            .unwrap();
        view.top = 20;
        vec![rows(buffer, 0, 20), rows(buffer, 30, 40)]
    } else {
        vec![rows(buffer, 1, 20), rows(buffer, 25, 35)]
    };
    view.set_folds(buffer, &ranges).unwrap();
    Session::new(
        buffers,
        view,
        Size {
            width: 60,
            height: 8,
        },
    )
    .unwrap()
}

fn row(session: &Session) -> usize {
    session.view().cell(session.buffer().unwrap()).unwrap().row
}

// Wheel steps and native click/drag boundaries resolve canonical visible rows.
// A region crossing a folded gap includes original hidden bytes exactly once.
#[test]
fn projected_gesture() {
    let mut session = editor(false);
    let stamp = session.buffer().unwrap().stamp();
    let wheel = gesture(MouseEventKind::ScrollDown, 6, 0);
    assert_eq!(session.handle(wheel), Action::Redraw);
    assert_eq!((session.view().top, row(&session)), (22, 22));
    session.handle(gesture(MouseEventKind::Down(MouseButton::Left), 6, 2));
    assert_eq!(row(&session), 24);
    session.handle(drag(6, 3));
    assert_eq!(row(&session), 35);
    let buffer = session.buffer().unwrap();
    let region = buffer.region(session.view().point).unwrap().unwrap();
    let selected = buffer.slice(&region).unwrap().to_string();
    assert!(selected.starts_with("* Row 24"));
    assert!(selected.contains("* Row 30"));
    assert!(!selected.contains("* Row 35"));
    session.handle(release(6, 3));
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// A hidden source tail cannot keep pointer deadlines armed forever. Each tick
// moves displayed rows, stops at the last visible source and releases its timer;
// reverse wheel scrolling also clamps at a hidden-prefix visible boundary.
#[test]
fn projected_autoscroll() {
    let mut session = editor(true);
    let now = Instant::now();
    session.handle_at(gesture(MouseEventKind::Down(MouseButton::Left), 6, 0), now);
    session.handle_at(drag(6, 6), now);
    assert!(session.remaining_input(now).is_some());
    for (tick, top, point) in [(1, 23, 28), (2, 26, 29), (3, 29, 29)] {
        assert_eq!(
            session.expire_input(now + Duration::from_millis(tick * 100)),
            Action::Redraw
        );
        assert_eq!((session.view().top, row(&session)), (top, point));
    }
    assert_eq!(
        session.remaining_input(now + Duration::from_millis(400)),
        None
    );
    session.handle_at(release(6, 6), now + Duration::from_millis(400));
    for _ in 0..4 {
        session.handle(gesture(MouseEventKind::ScrollUp, 6, 0));
    }
    assert_eq!(session.view().top, 20);
    assert_eq!(
        session.handle(gesture(MouseEventKind::ScrollUp, 6, 0)),
        Action::Wait
    );
    assert!(session.buffer().unwrap().history().is_empty());
}
