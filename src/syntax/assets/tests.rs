use super::*;

// Admission changes derived identity even with unchanged text/path/language.
// An observation captured from the still-valid fallback must become stale.
#[test]
fn generation_retires_fallback() {
    let fixture = crate::fixture::Fixture::new();
    let root = fixture.write("config.toml", "[language.rust.external]\nparser = 'fixture.so'\nsymbol = 'tree_sitter_rust'\nhighlights = 'highlights.scm'\ncompletions = 'completions.scm'\n");
    let snapshot = crate::config::prepare(&root).snapshot;
    let handle = snapshot.external("rust").unwrap();
    // Hold admission pending without scheduling native IO; real shared-library
    // paths and rejection/retention are exercised in tests/language_assets.rs.
    handle
        .entry
        .started
        .store(true, std::sync::atomic::Ordering::Release);
    let mut buffers = crate::buffer::Buffers::with_settings(snapshot).unwrap();
    let path = fixture.write("sample.rs", "fn intact() {}\n");
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get(id).unwrap();
    let stamp = buffer.stamp();
    let observed = crate::completion::source::Observation::capture(buffer);
    assert!(observed.current(&buffers));
    assert_eq!(buffer.mode(), "Rust");
    let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
    let highlights =
        Arc::new(tree_sitter::Query::new(&language, "(identifier) @function").unwrap());
    let completions = Arc::new(
        tree_sitter::Query::new(&language, "(function_item name: (identifier) @function)").unwrap(),
    );
    let assets = Arc::new(Assets {
        language,
        highlights,
        completions,
    });
    assert!(handle.entry.outcome.set(Ok(assets)).is_ok());
    assert!(buffers.poll_assets());
    assert!(!observed.current(&buffers));
    let buffer = buffers.get(id).unwrap();
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.mode(), "External");
    assert!(buffer.history().is_empty());
    assert!(!buffers.assets_pending());
}
