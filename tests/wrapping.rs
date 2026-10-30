#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;

use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use rev::display::{Frame, Position, Size};
use rev::gutter::Change;
use rev::view::View;

// Word breaks preserve every source scalar; tokens and oversized graphemes
// advance finitely without emitting half a cluster or altering canonical text.
#[test]
fn source_fragments() {
    let cases = [
        ("words", "one two three", 8, vec!["one two ", "three   "]),
        ("token", "abcdefghij", 4, vec!["abcd", "efgh", "ij  "]),
        (
            "combining",
            "e\u{301}e\u{301}e\u{301}",
            2,
            vec!["e\u{301}e\u{301}", "e\u{301} "],
        ),
        ("wide", "界界x", 3, vec!["界 ", "界x", "   "]),
        ("tiny", "界x", 1, vec![" ", "x", " "]),
        ("tabs", "a\tb", 4, vec!["a   ", "b   "]),
        ("empty", "", 4, vec!["    "]),
    ];
    for (name, source, width, expected) in cases {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let view = View::new(buffer);
        let fragments = view.wrapped_rows(buffer, width, 20).unwrap();
        assert_eq!(
            fragments
                .iter()
                .map(|f| f.text.as_str())
                .collect::<Vec<_>>(),
            expected,
            "{name}"
        );
        let mut end = Scalar(0);
        for fragment in &fragments {
            assert_eq!(fragment.stamp, buffer.stamp(), "{name}");
            assert_eq!(fragment.line, 0, "{name}");
            assert_eq!(fragment.source.start, end, "{name}");
            assert!(
                rev::coordinates::Coordinates::new(buffer)
                    .unwrap()
                    .is_boundary(fragment.source.end)
                    .unwrap(),
                "{name}"
            );
            end = fragment.source.end;
        }
        assert_eq!(end, Scalar(source.chars().count()), "{name}");
        assert!(!buffer.is_modified(), "{name}");
        assert!(buffer.history().is_empty(), "{name}");
        assert!(
            view.wrapped_rows(buffer, 0, 20).unwrap().is_empty(),
            "{name}"
        );
    }
}

// Actual rendering and reveal use pane-local fragments over one buffer. A
// continuation cursor remains visible after resize without changing its point.
#[test]
fn pane_widths() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("one two three four five six").unwrap();
    let buffer = buffers.get(id).unwrap();
    let stamp = buffer.stamp();
    let mut view = View::new(buffer);
    view.gutter.apply(Change::Visible(false));
    view.point.position = Scalar(buffer.len_chars());
    for width in [12, 8, 3, 15] {
        let size = Size { width, height: 5 };
        view.reveal(buffer, size).unwrap();
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        let cursor = frame.cursor().unwrap();
        assert!(cursor.column < width && cursor.row < size.text_rows());
        assert_eq!(view.point.position, Scalar(buffer.len_chars()));
        assert_eq!(frame.stamp(), stamp);
        assert_eq!(view.left, 0);
    }
    let mut other = View::new(buffer);
    other.gutter.apply(Change::Visible(false));
    let frame = Frame::project(
        buffer,
        &other,
        Size {
            width: 8,
            height: 6,
        },
        "",
    )
    .unwrap();
    assert_eq!(frame.rows()[0].text(), "one two ");
    assert_eq!(frame.rows()[1].text(), "three   ");
    assert_eq!(frame.cursor(), Some(Position { column: 0, row: 0 }));
    assert_eq!(buffer.stamp(), stamp);
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}

// Startup settings retain explicit false across includes and reject wrong
// types atomically. Language tables cannot silently acquire a wrap override.
#[test]
fn startup_policy() {
    let fixture = fixture::Fixture::new();
    for (name, source, expected, invalid) in [
        ("factory", "", true, false),
        ("enabled", "editor.soft_wrap = true", true, false),
        ("disabled", "editor.soft_wrap = false", false, false),
        ("wrong type", "editor.soft_wrap = 0", true, true),
        ("language", "language.rust.soft_wrap = false", true, true),
    ] {
        let startup = rev::config::prepare(&fixture.write("config.toml", source));
        assert_eq!(startup.diagnostic.is_some(), invalid, "{name}");
        let buffers = Buffers::with_settings(startup.snapshot).unwrap();
        assert_eq!(
            buffers.get(buffers.initial()).unwrap().soft_wrap(),
            expected,
            "{name}"
        );
    }
    fixture.write("included.toml", "editor.soft_wrap = false");
    let path = fixture.write("config.toml", "include = ['included.toml']");
    let startup = rev::config::prepare(&path);
    assert!(startup.diagnostic.is_none());
    assert!(!startup.snapshot.settings().editor.soft_wrap);
    assert_eq!(
        startup.snapshot.origin(&["editor", "soft_wrap"]),
        Some(fixture.0.join("included.toml").as_path())
    );
    let startup = rev::config::prepare(&fixture.write(
        "config.toml",
        "include = ['included.toml']\neditor.soft_wrap = true",
    ));
    assert!(startup.snapshot.settings().editor.soft_wrap);
    let sample: rev::config::Document = toml::from_str(rev::config::default_template()).unwrap();
    assert_eq!(sample.editor.soft_wrap, Some(true));
}

// A real M-x invocation toggles buffer-local presentation shared by split
// views. Save and undo still consume the original CRLF/Unicode source bytes.
#[test]
fn native_toggle_and_save() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::{Command, History, Split, WindowCommand};
    let fixture = fixture::Fixture::new();
    let source = "one two three four\t界e\u{301} five six seven eight\r\n";
    let path = fixture.write("source.txt", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let stamp = buffers.get(id).unwrap().stamp();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = rev::session::Session::new(
        buffers,
        view,
        Size {
            width: 40,
            height: 10,
        },
    )
    .unwrap();
    assert!(session.buffer().unwrap().soft_wrap());
    session.execute(Command::Split(Split::Right));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    session.handle(Event::Paste("visual-line-mode".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert!(!session.buffer().unwrap().soft_wrap());
    session.execute(Command::Window(WindowCommand::Other));
    assert!(!session.buffer().unwrap().soft_wrap());
    session.execute(Command::VisualLineMode);
    assert!(session.buffer().unwrap().soft_wrap());
    session.frame().unwrap();
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    session.execute(Command::SaveBuffer);
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    session.handle(Event::Paste("X".into()));
    session.execute(Command::History(History::Undo));
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .slice(
                &session
                    .buffer()
                    .unwrap()
                    .span(0..source.chars().count())
                    .unwrap()
            )
            .unwrap()
            .to_string(),
        source
    );
}

// Explicit no-wrap restores the admitted horizontal viewport, while toggling
// one buffer does not alter another buffer created from the same snapshot.
#[test]
fn unwrapped_alternative() {
    let fixture = fixture::Fixture::new();
    let startup = rev::config::prepare(&fixture.write("config.toml", "editor.soft_wrap = false"));
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create("one two three four five").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.gutter.apply(Change::Visible(false));
    view.point.position = Scalar(buffer.len_chars());
    let size = Size {
        width: 8,
        height: 5,
    };
    view.reveal(buffer, size).unwrap();
    assert!(view.left > 0);
    assert_eq!(
        Frame::project(buffer, &view, size, "")
            .unwrap()
            .cursor()
            .unwrap()
            .row,
        0
    );
    buffers.get_mut(id).unwrap().toggle_soft_wrap();
    let other = buffers.create("new text").unwrap();
    assert!(!buffers.get(other).unwrap().soft_wrap());
    assert!(buffers.get(id).unwrap().soft_wrap());
}

// At a word break, point belongs to the following fragment, including when
// the previous row has unused cells. Exact-width EOL retains an insertion row.
#[test]
fn cursor_affinity() {
    for (source, point, expected) in [
        ("one two three", 4, Position { column: 0, row: 1 }),
        ("abcdef", 6, Position { column: 0, row: 1 }),
    ] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.gutter.apply(Change::Visible(false));
        view.point.position = Scalar(point);
        let size = Size {
            width: 6,
            height: 5,
        };
        view.reveal(buffer, size).unwrap();
        assert_eq!(
            Frame::project(buffer, &view, size, "").unwrap().cursor(),
            Some(expected)
        );
    }
}
