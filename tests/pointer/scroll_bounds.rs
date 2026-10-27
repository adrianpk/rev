use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, History, Motion, RegionCommand, Scroll, Split};
use rev::coordinates::Scalar;
use rev::display::Position;
use rev::session::Action;
use rev::view::View;

use super::scroll::{editor, key, lines, size, wheel};

// Repeated input clamps at actual logical rows, including the final empty row.
// Empty/single-line sources have no scroll work, and reversal has no momentum.
#[test]
fn document_bounds() {
    for (name, source, last, end) in [
        ("empty", "".into(), 0, 0),
        ("single", "one".into(), 0, 0),
        ("final empty", "one\n".into(), 1, 4),
        ("crlf", "\r\n".into(), 1, 2),
        ("unterminated", "a\nb\nc".into(), 2, 4),
        ("large", lines(10_000), 10_000, 98_900),
    ] {
        let mut session = editor(&source);
        let stamp = session.buffer().unwrap().stamp();
        let steps = last / 3 + 2;
        for _ in 0..steps {
            session.handle(wheel(false, 6, 0));
        }
        assert_eq!(
            (session.view().top, session.view().point.position),
            (last, Scalar(end)),
            "{name}"
        );
        let before = session.view().clone();
        assert_eq!(session.handle(wheel(false, 6, 0)), Action::Wait, "{name}");
        assert_eq!(session.view(), &before);
        for _ in 0..steps {
            session.handle(wheel(true, 6, 0));
        }
        assert_eq!(session.view().top, 0);
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap().row,
            last.min(9)
        );
        assert_eq!(session.handle(wheel(true, 6, 0)), Action::Wait);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        let mut bytes = Vec::new();
        rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
        assert_eq!(bytes, source.as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// A still-visible point keeps its scalar and navigation goal. Only crossing
// the viewport edge adjusts its row, retaining the current display column.
#[test]
fn necessary_adjustment() {
    let mut session = editor(&lines(30));
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("8".into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Scroll(Scroll::NextLine));
    assert_eq!(session.view().point.position, Scalar(66));
    assert_eq!(session.view().column, Some(2));
    let mut expected = session.view().clone();
    for top in [3, 6] {
        expected.top = top;
        session.handle(wheel(false, 6, 0));
        assert_eq!(session.view(), &expected);
    }
    session.handle(wheel(false, 6, 0));
    assert_eq!(
        (
            session.view().top,
            session.view().point.position,
            session.view().column
        ),
        (9, Scalar(74), Some(2))
    );
    assert_eq!(
        session.frame().unwrap().cursor(),
        Some(Position { column: 8, row: 0 })
    );
}

// Forced edge adjustment uses the buffer's actual tab policy and canonical
// grapheme/CRLF boundaries, including read-only text and horizontal scrolling.
#[test]
fn checked_columns() {
    let fixture = super::config_fixture::Fixture::new();
    let source = "x\t界e\u{301}\r\n".repeat(20);
    for width in [1, 4, 8, 32] {
        let config = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let mut buffers = Buffers::with_settings(rev::config::prepare(&config).snapshot).unwrap();
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(true);
        let mut view = View::new(buffer);
        view.point.position = Scalar(2);
        view.left = width.max(2);
        let mut session = rev::session::Session::new(buffers, view, size()).unwrap();
        session.handle(wheel(false, 6, 0));
        assert_eq!(
            (session.view().top, session.view().point.position),
            (3, Scalar(23))
        );
        assert_eq!(
            session
                .view()
                .cell(session.buffer().unwrap())
                .unwrap()
                .column,
            width.max(2)
        );
        assert_eq!(session.view().left, width.max(2));
        assert_eq!(
            session.frame().unwrap().cursor(),
            Some(Position { column: 6, row: 0 })
        );
        assert_eq!(session.buffer().unwrap().newline(), "\r\n");
        assert!(session.buffer().unwrap().history().is_empty());
        let mut bytes = Vec::new();
        rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
        assert_eq!(bytes, source.as_bytes());
    }
}

// Existing typing groups include every shared point. A visible peer preserves
// the group; necessary peer movement starts a new group on the next edit.
#[test]
fn typing_continuity() {
    for (line, groups) in [(1, 2), (9, 1)] {
        let original = lines(30);
        let mut session = editor(&original);
        let active = session.windows().selected_id();
        session.execute(Command::Split(Split::Right));
        let peer = session.windows().order()[1];
        session.select_view(peer).unwrap();
        session.execute(Command::GotoLine);
        session.handle(Event::Paste(line.to_string()));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
        session.select_view(active).unwrap();
        session.handle(key(KeyCode::Char('A'), KeyModifiers::NONE));
        let before = session.view().clone();
        session.handle(wheel(false, 46, 0));
        assert_eq!(session.view(), &before);
        session.handle(key(KeyCode::Char('B'), KeyModifiers::NONE));
        assert_eq!(session.buffer().unwrap().history().len(), groups);
        for _ in 0..groups {
            session.execute(Command::History(History::Undo));
        }
        assert!(!session.buffer().unwrap().is_modified());
        let buffer = session.buffer().unwrap();
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap(),
            original
        );
        for _ in 0..groups {
            session.execute(Command::History(History::Redo));
        }
        let buffer = session.buffer().unwrap();
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap(),
            format!("AB{original}")
        );
    }
}

// The active region depends on the selected point and shared mark. Scrolling
// an inactive view changes neither, and records no buffer edit.
#[test]
fn shared_region() {
    let mut session = editor(&lines(30));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Split(Split::Right));
    let active = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    session.handle(wheel(false, 46, 0));
    assert_eq!(session.view(), &active);
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer.mark(), Some(Scalar(0)));
    assert!(buffer.mark_active());
    assert_eq!(
        buffer.region(session.view().point).unwrap().unwrap().range,
        0..1
    );
    assert_eq!(buffer.stamp(), stamp);
    assert!(buffer.history().is_empty());
}

// Borders, prompt and outside coordinates own no scrolling target. Pane
// decorations scroll their pane; the same old cell resolves anew after resize.
#[test]
fn current_bounds() {
    let mut session = editor(&lines(30));
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    let active = session.view().clone();
    let other = session.windows().get(peer).unwrap().clone();
    for (column, row) in [(39, 0), (0, 11), (u16::MAX, u16::MAX)] {
        assert_eq!(session.handle(wheel(false, column, row)), Action::Wait);
        assert_eq!(session.view(), &active);
        assert_eq!(session.windows().get(peer).unwrap(), &other);
    }
    session.handle(wheel(false, 46, 10));
    session.handle(wheel(false, 40, 0));
    assert_eq!(session.windows().get(peer).unwrap().top, 6);
    session.handle(Event::Resize(120, 12));
    let other = session.windows().get(peer).unwrap().clone();
    session.handle(wheel(false, 46, 0));
    assert_eq!(session.view().top, 3);
    assert_eq!(session.windows().get(peer).unwrap(), &other);
    session.handle(Event::Resize(2, 2));
    let before = session.view().clone();
    assert_eq!(session.handle(wheel(false, 0, 0)), Action::Wait);
    assert_eq!(session.view(), &before);
}
