use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::buffer::Buffers;
use rev::command::{Command, History, LinearCommand, Motion, RectangleCommand};
use rev::coordinates::{Cell, Coordinates, Scalar};
use rev::display::Position;
use rev::gutter::Change;
use rev::session::{Action, Session};
use rev::view::{Size, View};
use rev::window::Area;

fn editor(source: &str, width: u16, height: u16) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.bookmark_markers = false;
    view.gutter.apply(Change::Visible(false));
    Session::new(buffers, view, Size { width, height }).unwrap()
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn point(session: &Session) -> usize {
    session.view().point.position.0
}

fn text(session: &Session) -> String {
    let b = session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Continuation cells map to complete source graphemes, including expanded tabs.
// Word-break fill, oversized-glyph fallback and viewport padding have no hit.
#[test]
fn fragment_hits() {
    for (name, source, width, cells) in [
        (
            "words",
            "one two three",
            6,
            vec![
                (0, 0, Some(0)),
                (3, 0, Some(3)),
                (4, 0, None),
                (0, 1, Some(4)),
                (2, 2, Some(10)),
                (5, 2, Some(13)),
                (0, 3, None),
            ],
        ),
        (
            "unicode",
            "ab\t界e\u{301}z",
            4,
            vec![
                (2, 0, Some(2)),
                (3, 0, Some(2)),
                (0, 1, Some(3)),
                (1, 1, Some(3)),
                (2, 1, Some(4)),
                (3, 1, Some(6)),
                (0, 2, Some(7)),
            ],
        ),
        (
            "oversized",
            "界x",
            1,
            vec![(0, 0, None), (0, 1, Some(1)), (0, 2, Some(2))],
        ),
    ] {
        let session = editor(source, width, 9);
        let buffer = session.buffer().unwrap();
        for (column, row, expected) in cells {
            let hit = session
                .view()
                .hit(
                    buffer,
                    Area::new(0, 0, width, 7).unwrap(),
                    Position { column, row },
                )
                .unwrap();
            assert_eq!(hit.map(|p| p.position.0), expected, "{name}/{column}/{row}");
        }
        assert_eq!(text(&session), source);
        assert!(buffer.history().is_empty());
    }
}

// Native drag across word-break padding selects exactly one canonical range.
// Copy/kill/undo preserve the original source, without introducing display LF.
#[test]
fn continuation_drag() {
    let source = "one two three four\r\nlast";
    let mut session = editor(source, 6, 10);
    let stamp = session.buffer().unwrap().stamp();
    session.handle(mouse(MouseEventKind::Down(MouseButton::Left), 1, 0));
    session.handle(mouse(MouseEventKind::Drag(MouseButton::Left), 4, 0));
    assert_eq!(point(&session), 1); // Blank word-break padding cannot move point.
    session.handle(mouse(MouseEventKind::Drag(MouseButton::Left), 3, 2));
    session.handle(mouse(MouseEventKind::Up(MouseButton::Left), 3, 2));
    assert_eq!(point(&session), 11);
    let buffer = session.buffer().unwrap();
    assert_eq!(
        buffer.region(session.view().point).unwrap().unwrap().range,
        1..11
    );
    assert_eq!(buffer.stamp(), stamp);
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].selection(), Some(1..4));
    assert_eq!(frame.rows()[1].selection(), Some(0..4));
    assert_eq!(frame.rows()[2].selection(), Some(0..3));
    session.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(session.killed_text().unwrap().to_string(), "ne two thr");
    session.execute(Command::Linear(LinearCommand::Kill));
    assert_eq!(text(&session), "oee four\r\nlast");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert!(!session.buffer().unwrap().is_modified());
}

// Wheel movement advances visual rows within one logical line. A visible point
// stays fixed; the next edge crossing clamps only that point, and resize hits
// resolve the new width before any redraw can supply an obsolete cell map.
#[test]
fn wheel_and_resize() {
    let source = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut session = editor(source, 6, 7);
    session.handle(mouse(MouseEventKind::ScrollDown, 0, 0));
    assert_eq!(point(&session), 18);
    assert_eq!(session.view().top, 0);
    assert_eq!(
        session.frame().unwrap().cursor(),
        Some(Position { column: 0, row: 0 })
    );
    session.handle(mouse(MouseEventKind::Down(MouseButton::Left), 2, 1));
    session.handle(mouse(MouseEventKind::Up(MouseButton::Left), 2, 1));
    assert_eq!(point(&session), 26);
    session.handle(mouse(MouseEventKind::ScrollUp, 0, 0));
    assert_eq!(point(&session), 26);
    session.handle(Event::Resize(8, 7));
    session.handle(mouse(MouseEventKind::Down(MouseButton::Left), 1, 1));
    assert_eq!(point(&session), 9);
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(text(&session), source);
}

// Due ticks keep moving through continuations of the final logical line and
// stop at its real end. Release cancels the owned deadline and preserves mark.
#[test]
fn continuation_autoscroll() {
    let source = "0123456789".repeat(10);
    let mut session = editor(&source, 6, 7);
    let now = Instant::now();
    session.handle_at(mouse(MouseEventKind::Down(MouseButton::Left), 1, 0), now);
    session.handle_at(mouse(MouseEventKind::Drag(MouseButton::Left), 2, 5), now);
    for tick in 1..=3 {
        assert_eq!(
            session.expire_input(now + Duration::from_millis(tick * 100)),
            Action::Redraw
        );
        assert_eq!(point(&session), (tick as usize * 3 + 4) * 6 + 2);
        assert_eq!(session.view().top, 0);
        assert!(
            session
                .remaining_input(now + Duration::from_millis(tick * 100))
                .is_some()
        );
    }
    session.handle_at(
        mouse(MouseEventKind::Drag(MouseButton::Left), u16::MAX, 5),
        now + Duration::from_millis(300),
    );
    for tick in 4..=10 {
        session.expire_input(now + Duration::from_millis(tick * 100));
    }
    assert_eq!(point(&session), source.len());
    assert_eq!(session.remaining_input(now + Duration::from_secs(2)), None);
    session.handle(mouse(MouseEventKind::Up(MouseButton::Left), 2, 5));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(1)));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Search underlines source spans on every intersected visual row, leaving
// word-break fill untouched. A far continuation is revealed without editing.
#[test]
fn continuation_search() {
    let mut session = editor("one two three four five six seven eight nine ten", 6, 7);
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("two three".into()));
    assert_eq!(session.search_match().unwrap().range, 4..13);
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].search(), None);
    assert_eq!(frame.rows()[1].search(), Some(0..4));
    assert_eq!(frame.rows()[2].search(), Some(0..5));
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("nine".into()));
    assert_eq!(session.search_match().unwrap().range, 40..44);
    let frame = session.frame().unwrap();
    assert!(frame.cursor().is_some());
    assert!(
        frame
            .rows()
            .iter()
            .any(|r| r.text().starts_with("nine") && r.search() == Some(0..4))
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// Rectangles cover logical source columns on each source row, regardless of
// how many display continuations each occupies. Tab/wide-cell extraction and
// one atomic kill/undo retain the existing rectangular editing contract.
#[test]
fn logical_rectangle() {
    let source = "0123456789\na\t界zABC\n0123456789";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let start = coordinates.from_cell(Cell { row: 0, column: 2 }).unwrap();
    let end = coordinates.from_cell(Cell { row: 2, column: 5 }).unwrap();
    buffer
        .toggle_rectangle(rev::buffer::edit::Point {
            stamp: buffer.stamp(),
            position: start,
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    view.point.position = end;
    view.gutter.apply(Change::Visible(false));
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 4,
            height: 14,
        },
    )
    .unwrap();
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].selection(), Some(2..4));
    assert_eq!(frame.rows()[1].selection(), Some(0..1));
    assert_eq!(frame.rows()[2].selection(), None);
    assert_eq!(frame.rows()[3].selection(), Some(2..4));
    assert_eq!(frame.rows()[4].selection(), Some(0..2));
    session.execute(Command::Rectangle(RectangleCommand::Copy));
    assert_eq!(session.rectangle().unwrap().rows(), ["234", "  界", "234"]);
    session.execute(Command::Rectangle(RectangleCommand::Kill));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(text(&session), "0156789\na zABC\n0156789");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Source publication rejects the old view stamp, while native resize cancels
// a retained drag. The next press uses the edited text and the new pane width.
#[test]
fn source_and_capture() {
    let mut session = editor("one two three", 6, 7);
    let old = session.view().clone();
    session.handle(Event::Paste("X ".into()));
    assert!(
        old.hit(
            session.buffer().unwrap(),
            Area::new(0, 0, 6, 5).unwrap(),
            Position { column: 1, row: 1 }
        )
        .is_err()
    );
    let now = Instant::now();
    session.handle_at(mouse(MouseEventKind::Down(MouseButton::Left), 1, 1), now);
    assert_eq!(point(&session), 7);
    session.handle_at(mouse(MouseEventKind::Drag(MouseButton::Left), 1, 5), now);
    assert!(session.remaining_input(now).is_some());
    session.handle(Event::Resize(10, 7));
    assert_eq!(session.remaining_input(now), None);
    let retained = point(&session);
    assert_eq!(
        session.handle(mouse(MouseEventKind::Drag(MouseButton::Left), 1, 1)),
        Action::Wait
    );
    assert_eq!(point(&session), retained);
    session.handle(mouse(MouseEventKind::Down(MouseButton::Left), 1, 1));
    assert_eq!(point(&session), 11);
    assert_eq!(text(&session), "X one two three");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// An insertion row at an exact-width EOL may disappear on reflow. Its retained
// source anchor resolves inside the widened final fragment, even before redraw.
#[test]
fn resized_eol_anchor() {
    let mut session = editor("abcdefghij", 5, 3);
    session.execute(Command::Motion(Motion::BufferEnd));
    let retained = session.view().clone();
    let buffer = session.buffer().unwrap();
    let frame = rev::display::Frame::project(
        buffer,
        &retained,
        Size {
            width: 12,
            height: 3,
        },
        "",
    )
    .unwrap();
    assert_eq!(frame.cursor(), Some(Position { column: 10, row: 0 }));
    assert!(frame.rows()[0].text().starts_with("abcdefghij"));
    assert_eq!(
        retained
            .hit(
                buffer,
                Area::new(0, 0, 12, 1).unwrap(),
                Position { column: 2, row: 0 }
            )
            .unwrap()
            .unwrap()
            .position,
        Scalar(2)
    );
    assert!(buffer.history().is_empty());
}
