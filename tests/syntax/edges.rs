use std::fs;
use std::os::unix::fs::symlink;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{BufferCommand, Command, Motion, RegionCommand, Split, TreeCommand};
use rev::file::Rename;
use rev::session::Session;

use super::*;

// Missing, malformed and unusable query inputs all take the same real setup
// fallback. Parsing attempts cannot change the canonical document or save state.
#[test]
fn query_fallbacks() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("fn main() { let value = 42; }").unwrap();
    let buffer = buffers.get(id).unwrap();
    for query in [
        None,
        Some(""),
        Some("("),
        Some("(unknown_node) @keyword"),
        Some("(identifier) @unknown"),
    ] {
        let syntax = syntax(buffer, query);
        assert_eq!(syntax.mode(buffer.stamp()), "Text");
        assert!(syntax.captures(buffer, 0..buffer.len_bytes()).is_empty());
    }
    assert_eq!(text(buffer), "fn main() { let value = 42; }");
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}

// Error nodes from unfinished code do not make valid captures unusable. All
// emitted ranges still address exact current UTF-8 bytes, including Unicode.
#[test]
fn incomplete_source() {
    for source in [
        "fn unfinished() { let value =",
        "fn main() { let text = \"界",
        "fn main() { /* unfinished",
    ] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let syntax = syntax(buffer, Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
        assert_eq!(syntax.mode(buffer.stamp()), "Rust");
        let captures = syntax.captures(buffer, 0..buffer.len_bytes());
        assert!(
            captures
                .iter()
                .any(|capture| capture.style == Style::Keyword)
        );
        for capture in captures {
            assert!(source.get(capture.bytes).is_some());
        }
        assert_eq!(text(buffer), source);
        assert!(!buffer.is_modified());
    }
}

// Byte captures cross multibyte source and node text spans several actual rope
// chunks. Predicate evaluation still identifies the constructor without a copy
// of the entire document or captures from the offscreen prelude.
#[test]
fn chunk_queries() {
    let name = format!("Type{}", "A".repeat(1800));
    let prefix = "// padding\n".repeat(300);
    let source = format!("{prefix}fn main() {{ let value = {name}; }}");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get(id).unwrap();
    assert!(
        buffer
            .chunks(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .count()
            > 2
    );
    let syntax = syntax(buffer, Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
    let captures = syntax.captures(buffer, prefix.len()..buffer.len_bytes());
    assert!(
        captures
            .iter()
            .all(|capture| capture.bytes.start >= prefix.len())
    );
    assert!(
        captures
            .iter()
            .any(|capture| capture.style == Style::Type && source[capture.bytes.clone()] == name)
    );
    assert_eq!(text(buffer), source);
}

// Complete wide graphemes receive one style. A horizontal edge through a wide
// glyph remains an unstyled blank, while following quotes keep cell alignment.
#[test]
fn cell_projection() {
    let fixture = Fixture::new();
    let source = "fn main() {\n\tlet 名 = \"界\";\n}";
    let path = fixture.file("main.rs", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get(id).unwrap();
    for (left, expected) in [(14, 0..3), (15, 1..2), (16, 0..1)] {
        let mut view = View::new(buffer);
        view.top = 1;
        view.left = left;
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 8,
                height: 4,
            },
            "",
        )
        .unwrap();
        assert!(
            frame.rows()[0]
                .syntax()
                .iter()
                .any(|paint| paint.style == Style::String && paint.columns == expected),
            "left {left}: {:?}",
            frame.rows()[0].syntax()
        );
        if left == 15 {
            assert!(frame.rows()[0].text().starts_with(' '));
            assert!(
                frame.rows()[0]
                    .syntax()
                    .iter()
                    .all(|paint| paint.columns.start > 0)
            );
        }
    }
    assert_eq!(text(buffer), source);
}

// Recognition follows canonical referents and successful destination changes,
// never a unique display label. Rename/save keep all document/history owners.
#[test]
fn association_modes() {
    let fixture = Fixture::new();
    let source = "fn main() {}\n";
    let path = fixture.file("main.rs", source);
    symlink("main.rs", fixture.path("alias.txt")).unwrap();
    fs::create_dir(fixture.path("other")).unwrap();
    let other = fixture.file("other/main.rs", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("alias.txt")).unwrap();
    let second = buffers.visit(&other).unwrap();
    assert_eq!(buffers.get(id).unwrap().mode(), "Rust");
    assert_eq!(buffers.get(second).unwrap().name(), "main.rs<2>");
    assert_eq!(buffers.get(second).unwrap().mode(), "Rust");
    let stamp = buffers.get(id).unwrap().stamp();
    buffers
        .rename(Rename::capture(&path).unwrap(), "plain.txt")
        .unwrap();
    assert_eq!(buffers.get(id).unwrap().mode(), "Text");
    assert!(
        frame(buffers.get(id).unwrap())
            .rows()
            .iter()
            .all(|row| row.syntax().is_empty())
    );
    let save = buffers
        .prepare_save(id, Some(&fixture.path("new.rs")))
        .unwrap();
    assert!(buffers.publish_save(save).is_complete());
    assert_eq!(buffers.get(id).unwrap().mode(), "Rust");
    assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
    assert_eq!(text(buffers.get(id).unwrap()), source);
    assert_eq!(fs::read(fixture.path("new.rs")).unwrap(), source.as_bytes());
}

// Foregrounds remain independent of region/search attributes. Composition
// shifts cell ranges into split/tree geometry; inventory clears document styles.
#[test]
fn surface_layers() {
    let fixture = Fixture::new();
    let path = fixture.file("main.rs", "fn alpha() {}\n");
    let mut session = Session::empty(Size {
        width: 80,
        height: 12,
    })
    .unwrap();
    session.load(&path).unwrap();
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::ForwardWord));
    let first = session.frame().unwrap();
    assert_eq!(first.rows()[0].selection(), Some(0..2));
    assert!(!first.rows()[0].syntax().is_empty());
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("alpha".into()));
    let searched = session.frame().unwrap();
    assert_eq!(searched.rows()[0].search(), Some(3..8));
    assert!(!searched.rows()[0].syntax().is_empty());
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    session.execute(Command::Split(Split::Below));
    session.execute(Command::Tree(TreeCommand::Toggle));
    let tree = session.frame().unwrap();
    assert!(
        tree.rows()
            .iter()
            .flat_map(|row| row.syntax())
            .all(|paint| paint.columns.start >= 25 && paint.columns.end <= 80)
    );
    assert!(
        tree.rows()
            .iter()
            .filter(|row| !row.syntax().is_empty())
            .count()
            >= 2
    );
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Buffer(BufferCommand::List));
    let inventory = session.frame().unwrap();
    assert!(inventory.rows().iter().all(|row| row.syntax().is_empty()));
    assert_eq!(text(session.buffer().unwrap()), "fn alpha() {}\n");
    assert!(!session.buffer().unwrap().is_modified());
    assert!(session.buffer().unwrap().history().is_empty());
}
