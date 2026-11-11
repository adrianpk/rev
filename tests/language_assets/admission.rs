use crate::{configured, fixture::Fixture, rust_config, session, support};

// A real replacement changes query output and mode without changing the source.
// Loaded code survives dropping its startup snapshot and unlinking owned assets.
#[test]
fn replacement_retention() {
    let libraries = support::libraries();
    let fixture = Fixture::new();
    let parser = fixture.0.join("rust-v1.so");
    std::fs::copy(libraries.parser("rust"), &parser).unwrap();
    let config = rust_config(&fixture, &parser, "tree_sitter_rust");
    let path = fixture.write("sample.rs", "fn retained() {}\n");
    let mut editor = session(configured(&fixture, &config), &path);
    support::settle(&mut editor);
    assert_eq!(
        editor.buffer().unwrap().external_grammar_status(),
        Some(Ok(()))
    );
    assert_eq!(editor.buffer().unwrap().mode(), "External");
    let buffer = editor.buffer().unwrap();
    let captures = buffer.syntax().captures(buffer, 0..buffer.len_bytes());
    assert!(
        captures.iter().any(
            |capture| capture.bytes == (3..11) && capture.style == rev::syntax::Style::Function
        )
    );
    assert!(
        !captures
            .iter()
            .any(|capture| capture.style == rev::syntax::Style::Keyword)
    );
    std::fs::remove_file(parser).unwrap();
    std::fs::remove_file(fixture.0.join("highlights.scm")).unwrap();
    std::fs::remove_file(fixture.0.join("completions.scm")).unwrap();
    editor.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' ')),
    ));
    editor.poll_frame().unwrap();
    assert_eq!(editor.buffer().unwrap().mode(), "External");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
}

// Every rejected asset preserves the bundled parser and canonical text. Fixing
// the files later cannot retry a cached failure in this session.
#[test]
fn rejection_fallback() {
    let libraries = support::libraries();
    for case in [
        "missing",
        "symbol",
        "abi",
        "highlights",
        "completions",
        "capture",
        "predicate",
        "utf8",
        "large",
        "directory",
        "empty",
        "scope-only",
        "property",
        "highlight-capture",
    ] {
        let fixture = Fixture::new();
        let parser = match case {
            "missing" => fixture.0.join("absent.so"),
            "abi" => libraries.parser("incompatible").clone(),
            _ => libraries.parser("rust").clone(),
        };
        let symbol = if case == "symbol" {
            "tree_sitter_absent"
        } else {
            "tree_sitter_rust"
        };
        let config = rust_config(&fixture, &parser, symbol);
        reject_asset(&fixture, case);
        let path = fixture.write("sample.rs", "fn intact() {}\n");
        let mut editor = session(configured(&fixture, &config), &path);
        support::settle(&mut editor);
        let error = editor
            .buffer()
            .unwrap()
            .external_grammar_status()
            .unwrap()
            .unwrap_err()
            .to_owned();
        assert_eq!(editor.buffer().unwrap().mode(), "Rust", "{case}: {error}");
        assert!(editor.buffer().unwrap().history().is_empty(), "{case}");
        if case == "missing" {
            std::fs::copy(libraries.parser("rust"), &parser).unwrap();
        }
        fixture.write(
            "completions.scm",
            "(function_item name: (identifier) @function)",
        );
        editor.poll_frame().unwrap();
        assert_eq!(
            editor.buffer().unwrap().external_grammar_status(),
            Some(Err(error.as_str())),
            "{case}"
        );
    }
}

fn reject_asset(fixture: &Fixture, case: &str) {
    let queries = [
        ("empty", "completions.scm", ""),
        ("scope-only", "completions.scm", "(block) @scope"),
        (
            "property",
            "completions.scm",
            "((identifier) @function (#set! custom))",
        ),
        ("highlight-capture", "highlights.scm", "(identifier) @typo"),
        ("highlights", "highlights.scm", "(absent) @function"),
        ("completions", "completions.scm", "(absent) @function"),
        ("capture", "completions.scm", "(identifier) @typo"),
        (
            "predicate",
            "completions.scm",
            "((identifier) @function (#unknown? @function))",
        ),
    ];
    if let Some((_, file, source)) = queries.into_iter().find(|(name, _, _)| *name == case) {
        fixture.write(file, source);
        return;
    }
    match case {
        "utf8" => std::fs::write(fixture.0.join("completions.scm"), [255]).unwrap(),
        "large" => {
            fixture.write("completions.scm", &" ".repeat(1024 * 1024 + 1));
        }
        "directory" => {
            std::fs::remove_file(fixture.0.join("highlights.scm")).unwrap();
            std::fs::create_dir(fixture.0.join("highlights.scm")).unwrap();
        }
        _ => (),
    }
}

// An unused startup registration does not touch assets. An unbundled identity
// remains editable plain text when its explicitly selected asset cannot load.
#[test]
fn lazy_plain_fallback() {
    let fixture = Fixture::new();
    let config = rust_config(&fixture, &fixture.0.join("absent.so"), "tree_sitter_rust");
    let config = config.replace("language.rust", "language.python");
    let snapshot = configured(&fixture, &config);
    let mut buffers = rev::buffer::Buffers::with_settings(snapshot.clone()).unwrap();
    let rust = fixture.write("sample.rs", "fn bundled() {}\n");
    let id = buffers.visit(&rust).unwrap();
    assert_eq!(buffers.get(id).unwrap().mode(), "Rust");
    assert!(buffers.get(id).unwrap().external_grammar_status().is_none());
    let path = fixture.write("sample.py", "def intact():\n    pass\n");
    let mut editor = session(snapshot, &path);
    support::settle(&mut editor);
    assert_eq!(editor.buffer().unwrap().mode(), "Text");
    assert!(
        editor
            .buffer()
            .unwrap()
            .external_grammar_status()
            .unwrap()
            .is_err()
    );
    editor.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('#')),
    ));
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
}
