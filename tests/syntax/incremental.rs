use rev::buffer::edit::{History, Replacement};
use rev::command::{Command, History as Direction};
use rev::session::Session;
use tree_sitter::{InputEdit, Point};

use super::*;

fn apply(buffer: &mut Buffer, range: std::ops::Range<usize>, inserted: &str) {
    let replacement = Replacement {
        source: buffer.span(range).unwrap(),
        text: inserted.into(),
    };
    let prepared = buffer.prepare(&[replacement], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
}
fn compare(buffer: &mut Buffer) {
    buffer.refresh_syntax();
    let reference = syntax(buffer, Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
    assert_eq!(
        buffer.syntax().captures(buffer, 0..buffer.len_bytes()),
        reference.captures(buffer, 0..buffer.len_bytes())
    );
}
fn point(row: usize, column: usize) -> Point {
    Point { row, column }
}

// Input edits use bytes and LF-based byte columns, including inserted CRLF,
// multibyte scalars and removal across line boundaries. Preparation is inert.
#[test]
fn exact_edits() {
    let cases = [
        (
            "a界\r\nb\nc",
            1..5,
            "é\r\n🙂",
            InputEdit {
                start_byte: 1,
                old_end_byte: 7,
                new_end_byte: 9,
                start_position: point(0, 1),
                old_end_position: point(1, 1),
                new_end_position: point(1, 4),
            },
        ),
        (
            "a\rb",
            1..2,
            "\n",
            InputEdit {
                start_byte: 1,
                old_end_byte: 2,
                new_end_byte: 2,
                start_position: point(0, 1),
                old_end_position: point(0, 2),
                new_end_position: point(1, 0),
            },
        ),
        (
            "\r\n",
            0..2,
            "界",
            InputEdit {
                start_byte: 0,
                old_end_byte: 2,
                new_end_byte: 3,
                start_position: point(0, 0),
                old_end_position: point(1, 0),
                new_end_position: point(0, 3),
            },
        ),
        (
            "é",
            1..1,
            "🙂",
            InputEdit {
                start_byte: 2,
                old_end_byte: 2,
                new_end_byte: 6,
                start_position: point(0, 2),
                old_end_position: point(0, 2),
                new_end_position: point(0, 6),
            },
        ),
    ];
    for (source, range, inserted, expected) in cases {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let prepared = buffer
            .prepare(
                &[Replacement {
                    source: buffer.span(range).unwrap(),
                    text: inserted.into(),
                }],
                &[],
                None,
            )
            .unwrap();
        assert_eq!(prepared.syntax_edits(), &[expected]);
        assert_eq!(text(buffer), source);
        assert!(buffer.history().is_empty());
    }
}

// Disjoint replacements stay in descending base coordinates. Lower inserted
// newlines must not invalidate the higher row's already prepared input edit.
#[test]
fn multiple_edits() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\nbb\ncc").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let replacements = [
        Replacement {
            source: buffer.span(1..2).unwrap(),
            text: "é\nX".into(),
        },
        Replacement {
            source: buffer.span(6..8).unwrap(),
            text: "\r\n🙂".into(),
        },
    ];
    let prepared = buffer.prepare(&replacements, &[], None).unwrap();
    assert_eq!(
        prepared.syntax_edits(),
        &[
            InputEdit {
                start_byte: 8,
                old_end_byte: 10,
                new_end_byte: 14,
                start_position: point(2, 0),
                old_end_position: point(2, 2),
                new_end_position: point(3, 4)
            },
            InputEdit {
                start_byte: 1,
                old_end_byte: 4,
                new_end_byte: 5,
                start_position: point(0, 1),
                old_end_position: point(0, 4),
                new_end_position: point(1, 1)
            },
        ]
    );
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(text(buffer), "aé\nX\nbb\n\r\n🙂");
}

// Two accepted revisions can advance a hidden tree before any parse. Fresh
// captures must equal a full parse; undo and redo also reuse correctly edited trees.
#[test]
fn retained_parse() {
    let fixture = Fixture::new();
    let path = fixture.file("main.rs", "fn main() { let text = \"界\"; }\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    assert!(!buffer.syntax().is_incremental());
    apply(buffer, 0..0, "// α\n");
    apply(buffer, 0..0, "// β\n");
    assert_eq!(buffer.mode(), "Text");
    assert!(
        buffer
            .syntax()
            .captures(buffer, 0..buffer.len_bytes())
            .is_empty()
    );
    compare(buffer);
    assert!(buffer.syntax().is_incremental());
    let content = text(buffer);
    let offset = content[..content.find('界').unwrap()].chars().count();
    apply(buffer, offset..offset + 1, "🙂é");
    compare(buffer);
    assert!(buffer.syntax().is_incremental());
    let edited = text(buffer);
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    compare(buffer);
    assert!(buffer.syntax().is_incremental());
    assert_eq!(text(buffer), content);
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    compare(buffer);
    assert!(buffer.syntax().is_incremental());
    assert_eq!(text(buffer), edited);
}

// Typing contributes its own exact input edits even when history coalesces.
// A literal multiline paste and grouped undo/redo use the same publication path.
#[test]
fn grouped_paste() {
    use crossterm::event::Event;
    let fixture = Fixture::new();
    let path = fixture.file("main.rs", "fn main() {}\n");
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.load(&path).unwrap();
    for character in "// ".chars() {
        session.insert(character);
    }
    session.frame().unwrap();
    assert!(session.buffer().unwrap().syntax().is_incremental());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(Direction::Undo));
    session.frame().unwrap();
    assert!(session.buffer().unwrap().syntax().is_incremental());
    assert_eq!(text(session.buffer().unwrap()), "fn main() {}\n");
    session.execute(Command::History(Direction::Redo));
    session.frame().unwrap();
    assert!(session.buffer().unwrap().syntax().is_incremental());
    session.handle(Event::Paste("α\r\nβ\n".into()));
    session.frame().unwrap();
    assert!(session.buffer().unwrap().syntax().is_incremental());
    assert_eq!(text(session.buffer().unwrap()), "// α\r\nβ\nfn main() {}\n");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    let reference = syntax(
        session.buffer().unwrap(),
        Some(tree_sitter_rust::HIGHLIGHTS_QUERY),
    );
    let buffer = session.buffer().unwrap();
    assert_eq!(
        buffer.syntax().captures(buffer, 0..buffer.len_bytes()),
        reference.captures(buffer, 0..buffer.len_bytes())
    );
}
