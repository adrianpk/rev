use rev::buffer::edit::{History, Replacement};
use rev::config::prepare;
use rev::coordinates::Scalar;
use rev::file::Rename;

use super::*;

// A configured grammar selects only the bundled Rust parser or plain fallback;
// associations still identify language independently of grammar availability.
#[test]
fn grammar_policy() {
    for (name, config, mode) in [
        ("main.rs", "", "Rust"),
        ("main.rs", "syntax.enabled = false", "Text"),
        ("main.rs", "language.rust.grammar = 'plain'", "Text"),
        ("main.go", "", "Text"),
        ("main.go", "language.go.grammar = 'rust'", "Rust"),
        (
            "main.txt",
            "files.associations.'*.txt' = 'rust'\nlanguage.rust.grammar = 'plain'",
            "Text",
        ),
    ] {
        let fixture = Fixture::new();
        let path = fixture.file(name, "fn main() {}\n");
        let startup = prepare(&fixture.file("config.toml", config));
        assert!(startup.diagnostic.is_none(), "{config}");
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let stamp = buffer.stamp();
        let state = buffer.state();
        buffer.refresh_syntax();
        assert_eq!(buffer.mode(), mode, "{name}: {config}");
        assert_eq!(!frame(buffer).rows()[0].syntax().is_empty(), mode == "Rust");
        assert_eq!((buffer.stamp(), buffer.state()), (stamp, state));
        assert_eq!(text(buffer), "fn main() {}\n");
        assert!(buffer.history().is_empty() && !buffer.is_modified());
    }
}

// The cutoff counts UTF-8 bytes and admits equality. Crossing it invalidates
// colors without rejecting edits; undo/shrink restores parsing from current text.
#[test]
fn byte_cutoff() {
    let fixture = Fixture::new();
    let source = "fn main() {} // 界\n";
    let path = fixture.file("main.rs", source);
    let startup = prepare(&fixture.file(
        "config.toml",
        &format!("syntax.max_bytes = {}", source.len()),
    ));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    assert_eq!(buffer.mode(), "Rust");
    let saved = buffer.state();
    let insertion = Replacement {
        source: buffer.span(buffer.len_chars()..buffer.len_chars()).unwrap(),
        text: "界".into(),
    };
    let prepared = buffer.prepare(&[insertion], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let changed = buffer.state();
    assert_eq!(buffer.mode(), "Text");
    buffer.refresh_syntax();
    assert!(
        frame(buffer)
            .rows()
            .iter()
            .all(|row| row.syntax().is_empty())
    );
    assert_eq!(text(buffer), format!("{source}界"));
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    buffer.refresh_syntax();
    assert_eq!(buffer.mode(), "Rust");
    assert!(!buffer.syntax().is_incremental());
    assert_eq!(buffer.state(), saved);
    assert!(!buffer.is_modified());
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    buffer.refresh_syntax();
    assert_eq!(buffer.mode(), "Text");
    assert_eq!(buffer.state(), changed);
    assert_eq!(buffer.history().len(), 1);
    assert!(buffer.is_modified());
}

// Supplying another admitted policy can discard only derived syntax. Rust/plain
// path changes re-resolve that same snapshot and never alter a dirty history.
#[test]
fn policy_transitions() {
    let fixture = Fixture::new();
    let path = fixture.file("main.rs", "fn main() {}\n");
    let plain = prepare(&fixture.file("plain.toml", "language.rust.grammar = 'plain'"));
    let rust = prepare(&fixture.file("rust.toml", ""));
    assert!(plain.diagnostic.is_none() && rust.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(plain.snapshot.clone()).unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "// note\n".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.set_mark(Some(Scalar(3))).unwrap();
    let stamp = buffer.stamp();
    let state = buffer.state();
    for (snapshot, mode) in [(&rust.snapshot, "Rust"), (&plain.snapshot, "Text")] {
        buffer.apply_settings(snapshot);
        assert_eq!(buffer.mode(), mode);
        assert_eq!((buffer.stamp(), buffer.state()), (stamp, state));
        assert_eq!(buffer.mark(), Some(Scalar(3)));
        assert_eq!(buffer.history().len(), 1);
    }
    buffers
        .rename(Rename::capture(&path).unwrap(), "other.rs")
        .unwrap();
    let buffer = buffers.get(id).unwrap();
    assert_eq!(buffer.mode(), "Text");
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(text(buffer), "// note\nfn main() {}\n");
    assert!(buffer.is_modified());
}
