#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rev::buffer::{Buffer, Buffers};
use rev::display::{Frame, Size};
use rev::syntax::{Style, Syntax};
use rev::view::View;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rev-syntax-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn file(&self, name: &str, source: &str) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, source).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn syntax(buffer: &Buffer, query: Option<&str>) -> Syntax {
    let mut syntax = Syntax::rust(query);
    syntax.update(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap(),
        buffer.stamp(),
    );
    syntax
}
fn frame(buffer: &Buffer) -> Frame {
    Frame::project(
        buffer,
        &View::new(buffer),
        Size {
            width: 80,
            height: 8,
        },
        "ready",
    )
    .unwrap()
}

// The bundled grammar/query, including text predicates, supplies each palette
// class. These are exact source-byte spans, not cell or scalar approximations.
#[test]
fn bundled_palette() {
    let source = "// note\nfn build(value: &i32) { let text = \"hello\"; let count = 42; }";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let syntax = syntax(buffer, Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
    assert_eq!(syntax.mode(buffer.stamp()), "Rust");
    let captures = syntax.captures(buffer, 0..buffer.len_bytes());
    for (token, style) in [
        ("// note", Style::Comment),
        ("fn", Style::Keyword),
        ("build", Style::Function),
        ("i32", Style::Type),
        ("\"hello\"", Style::String),
        ("42", Style::Number),
        ("&", Style::Punctuation),
    ] {
        assert!(
            captures
                .iter()
                .any(|capture| capture.style == style && &source[capture.bytes.clone()] == token),
            "missing {token}: {captures:?}"
        );
    }
    assert_eq!(text(buffer), source);
    assert!(!buffer.is_modified());
}

// Canonical file extensions select language; unique display suffixes and a
// missing .rs destination do not change recognition. Unsupported text stays live.
#[test]
fn file_modes() {
    let fixture = Fixture::new();
    let mut buffers = Buffers::new().unwrap();
    for (name, expected) in [
        ("main.rs", "Rust"),
        ("main.txt", "Text"),
        ("main.RS", "Text"),
        ("README", "Text"),
    ] {
        let path = fixture.file(name, "fn main() {}\n");
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(buffer.mode(), expected);
        assert_eq!(text(buffer), "fn main() {}\n");
        assert!(!buffer.is_modified());
        assert_eq!(
            frame(buffer).rows()[6].text().split_whitespace().last(),
            Some(expected)
        );
    }
    let id = buffers.visit(&fixture.path("absent.rs")).unwrap();
    assert_eq!(buffers.get(id).unwrap().mode(), "Rust");
    assert!(!fixture.path("absent.rs").exists());
    assert_eq!(buffers.get(buffers.initial()).unwrap().mode(), "Text");
}

// Styles are separate row data and ANSI foreground changes. Raw document,
// save bytes, logical state and cursor remain unchanged by projection/writing.
#[test]
fn visual_only() {
    crossterm::style::force_color_output(true);
    let fixture = Fixture::new();
    let source = "fn main() { let s = \"界\"; }\n// comment";
    let path = fixture.file("main.rs", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get(id).unwrap();
    let stamp = buffer.stamp();
    let state = buffer.state();
    let frame = frame(buffer);
    assert!(!frame.rows()[0].syntax().is_empty());
    assert!(frame.rows()[6..].iter().all(|row| row.syntax().is_empty()));
    let mut terminal = Vec::new();
    frame.write(&mut terminal).unwrap();
    assert!(
        terminal
            .windows(b"\x1b[38;5;176m".len())
            .any(|bytes| bytes == b"\x1b[38;5;176m")
    );
    assert!(terminal.windows(5).any(|bytes| bytes == b"\x1b[39m"));
    assert_eq!(frame.cursor().unwrap().column, 0);
    let mut output = Vec::new();
    rev::file::write(buffer, &mut output).unwrap();
    assert_eq!(output, source.as_bytes());
    assert_eq!(text(buffer), source);
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.state(), state);
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}

// Accepted edits invalidate old coordinates immediately. Session presentation
// refreshes the current revision once; repeated redraw cannot alter its history.
#[test]
fn current_revision() {
    use rev::session::Session;
    let fixture = Fixture::new();
    let path = fixture.file("main.rs", "fn main() {}\n");
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.load(&path).unwrap();
    let before = session.buffer().unwrap().stamp();
    session.insert('/');
    assert_ne!(session.buffer().unwrap().stamp(), before);
    assert_eq!(session.buffer().unwrap().mode(), "Text");
    assert!(
        frame(session.buffer().unwrap())
            .rows()
            .iter()
            .all(|row| row.syntax().is_empty())
    );
    let stamp = session.buffer().unwrap().stamp();
    let state = session.buffer().unwrap().state();
    for _ in 0..2 {
        let frame = session.frame().unwrap();
        assert!(frame.rows()[6].text().contains("Rust"));
        assert!(!frame.rows()[0].syntax().is_empty());
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().state(), state);
    assert_eq!(text(session.buffer().unwrap()), "/fn main() {}\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

#[path = "syntax/edges.rs"]
mod edges;

#[path = "syntax/incremental.rs"]
mod incremental;

#[path = "syntax/fidelity.rs"]
mod fidelity;
