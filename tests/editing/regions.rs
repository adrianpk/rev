use rev::command::{Command, Motion, RegionCommand};
use rev::coordinates::Scalar;
use rev::session::{Action, Session};

fn region(session: &mut Session, command: RegionCommand) {
    assert_eq!(session.execute(Command::Region(command)), Action::Redraw);
}

fn range(session: &Session) -> Option<std::ops::Range<usize>> {
    session
        .buffer()
        .unwrap()
        .region(session.view().point)
        .unwrap()
        .map(|span| span.range)
}

// Movement grows/shrinks a region in either direction. Exchange preserves the
// selected source text while swapping the stored mark and selected view point.
#[test]
fn endpoints() {
    let mut session = super::editor("ab界e\u{301}z", 2, false);
    let stamp = session.buffer().unwrap().stamp();
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(session.echo(), "Mark is not set");
    region(&mut session, RegionCommand::Set);
    assert_eq!(range(&session), Some(2..2));
    session.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(range(&session), Some(2..6));
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(6)));
    assert_eq!(range(&session), Some(2..6));
    session.execute(Command::Motion(Motion::BufferStart));
    assert_eq!(range(&session), Some(0..6));
    session.execute(Command::KeyboardQuit);
    assert_eq!(range(&session), None);
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(6)));
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(range(&session), Some(0..6));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Multi-row presentation highlights complete Unicode display cells and selected
// line breaks, leaving document text, status and echo free of escape sequences.
#[test]
fn multiline_surface() {
    let mut session = super::editor("a\t界e\u{301}\r\nlast", 1, false);
    region(&mut session, RegionCommand::Set);
    session.execute(Command::Motion(Motion::BufferEnd));
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].selection(), Some(1..48));
    assert_eq!(frame.rows()[1].selection(), Some(0..4));
    assert!(
        frame
            .rows()
            .iter()
            .skip(2)
            .all(|row| row.selection().is_none())
    );
    let cells = super::surface::draw(&frame);
    let palette = rev::config::Palette::default();
    assert_eq!(cells[&(0, 1)].foreground, Some(palette.selection_text));
    assert_eq!(cells[&(0, 1)].background, Some(palette.selection));
    assert_eq!(cells[&(1, 0)].background, Some(palette.selection));
    assert_eq!(cells[&(1, 4)].background, Some(palette.background));
    assert!(frame.rows().iter().all(|row| !row.text().contains('\x1b')));
    assert!(!session.buffer().unwrap().is_modified());
}

// Successful text publication deactivates the transient region but still maps
// the left-affine stored mark; a read-only refusal preserves active selection.
#[test]
fn edit_deactivation() {
    for read_only in [false, true] {
        let mut session = super::editor("abcdef", 2, read_only);
        region(&mut session, RegionCommand::Set);
        session.execute(Command::Motion(Motion::BufferEnd));
        session.insert('X');
        assert_eq!(session.buffer().unwrap().mark_active(), read_only);
        assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(2)));
        assert_eq!(
            session.buffer().unwrap().history().len(),
            usize::from(!read_only)
        );
    }
}

// Activation is shared by the buffer, while only the selected pane's current
// point determines the visible region. Other points remain independent.
#[test]
fn shared_focus() {
    use rev::command::{History, Split};
    let mut session = super::editor("abcdef\nsecond", 2, false);
    region(&mut session, RegionCommand::Set);
    session.execute(Command::Motion(Motion::LineEnd));
    let first = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let second = session.windows().order()[1];
    assert_eq!(session.frame().unwrap().rows()[0].selection(), Some(2..6));
    session.select_view(second).unwrap();
    session.execute(Command::Motion(Motion::LineStart));
    assert_eq!(range(&session), Some(0..2));
    assert_eq!(session.frame().unwrap().rows()[0].selection(), Some(24..26));
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(
        session.windows().get(first).unwrap().point.position,
        Scalar(6)
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(0)));
    session.insert('X');
    assert_eq!(
        session.windows().get(first).unwrap().point.position,
        Scalar(7)
    );
    assert!(!session.buffer().unwrap().mark_active());
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.selection().is_none())
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(
        session.windows().get(first).unwrap().point.position,
        Scalar(6)
    );
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(range(&session), Some(0..2));
    assert_eq!(session.frame().unwrap().rows()[0].selection(), Some(24..26));
}

// Selection follows display cells across clipped tabs, complete combining
// clusters and partial wide/emoji edge spaces; status/echo never inherit style.
#[test]
fn clipped_regions() {
    use rev::buffer::Buffers;
    use rev::buffer::edit::Point;
    use rev::display::{Frame, Size};
    use rev::view::View;
    use unicode_width::UnicodeWidthStr;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\t界e\u{301}👩\u{200d}💻\nlast").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer
        .activate_mark(Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(buffer.len_chars());
    for (left, expected) in [(0, "a  "), (3, " 界"), (5, " e\u{301} ")] {
        view.left = left;
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 3,
                height: 4,
            },
            "notice",
        )
        .unwrap();
        assert_eq!(frame.rows()[0].text(), expected);
        assert_eq!(frame.rows()[0].selection(), Some(0..3));
        assert!(frame.rows().iter().all(|row| row.text().width() == 3));
        assert!(
            frame
                .rows()
                .iter()
                .skip(2)
                .all(|row| row.selection().is_none())
        );
    }
    for size in [
        Size {
            width: 0,
            height: 4,
        },
        Size {
            width: 3,
            height: 0,
        },
        Size {
            width: 1,
            height: 1,
        },
    ] {
        let frame = Frame::project(buffer, &view, size, "notice").unwrap();
        assert!(frame.rows().iter().all(|row| row.selection().is_none()));
        let mut bytes = Vec::new();
        frame.write(&mut bytes).unwrap();
        assert!(!bytes.windows(4).any(|part| part == b"\x1b[7m"));
    }
}

// Changing activation after preparation cannot overwrite the newer selection.
// Both directions reject without moving text, points, mark or history.
#[test]
fn prepared_selection() {
    use rev::buffer::Buffers;
    use rev::buffer::edit::Error;
    for active in [false, true] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("abcdef").unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let point = super::point(buffer, 2);
        buffer.set_mark(Some(Scalar(2))).unwrap();
        if active {
            buffer.activate_mark(point).unwrap();
        }
        let prepared = buffer
            .prepare(&[super::replacement(buffer, 0..0, "X")], &[point], None)
            .unwrap();
        if active {
            buffer.deactivate_region();
        } else {
            buffer.activate_mark(point).unwrap();
        }
        let mut points = [point];
        assert_eq!(
            buffer.publish(prepared, &mut points),
            Err(Error::ChangedPositions)
        );
        assert_eq!(points, [point]);
        assert_eq!(buffer.mark_active(), !active);
        assert_eq!(buffer.mark(), Some(Scalar(2)));
        assert_eq!(super::text(buffer), "abcdef");
        assert!(buffer.history().is_empty());
    }
}

// A transient owner rejects native mark commands, and inventory removes all
// document styling. C-g cancels that owner and deactivates, retaining its mark.
#[test]
fn transient_selection() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::BufferCommand;
    for mode in ["argument", "inventory", "decision"] {
        let mut session = super::editor("abcdef", 2, false);
        session.insert('X');
        region(&mut session, RegionCommand::Set);
        session.execute(Command::Motion(Motion::BufferEnd));
        let point = session.view().point;
        let mark = session.buffer().unwrap().mark();
        match mode {
            "argument" => {
                session.execute(Command::FindFile);
            }
            "inventory" => {
                session.execute(Command::Buffer(BufferCommand::List));
            }
            _ => {
                session.execute(Command::Buffer(BufferCommand::Kill));
                session.handle(Event::Key(KeyEvent::new(
                    KeyCode::Enter,
                    KeyModifiers::NONE,
                )));
            }
        }
        for command in [RegionCommand::Set, RegionCommand::Exchange] {
            region(&mut session, command);
            assert!(session.echo().contains("owns input focus"));
            assert_eq!(session.view().point, point);
            assert_eq!(session.buffer().unwrap().mark(), mark);
            assert!(session.buffer().unwrap().mark_active());
        }
        if mode == "inventory" {
            assert!(
                session
                    .frame()
                    .unwrap()
                    .rows()
                    .iter()
                    .all(|row| row.selection().is_none())
            );
        }
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(range(&session), None);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        region(&mut session, RegionCommand::Exchange);
        assert_eq!(range(&session), Some(3..7));
    }
}
