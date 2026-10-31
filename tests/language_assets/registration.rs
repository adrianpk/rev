use crate::{configured, fixture::Fixture, rust_config, session, support};

// Startup admits the four explicit leaves, caps registrations and rejects
// partial/unknown/invalid-symbol tables without accepting a partial snapshot.
#[test]
fn startup_contract() {
    let fixture = Fixture::new();
    let valid = rust_config(&fixture, &fixture.0.join("unread.so"), "tree_sitter_rust");
    assert!(rev::config::parse(std::path::Path::new("config.toml"), valid.as_bytes()).is_ok());
    for (name, source) in [
        (
            "partial",
            "[language.rust.external]\nparser = 'parser.so'".to_owned(),
        ),
        ("unknown", format!("{valid}install = true\n")),
        ("symbol", valid.replace("tree_sitter_rust", "bad-symbol")),
        ("empty", valid.replace("tree_sitter_rust", "tree_sitter_")),
        ("path", valid.replace("unread.so", "https://parser.so")),
    ] {
        assert!(
            rev::config::parse(std::path::Path::new("config.toml"), source.as_bytes()).is_err(),
            "{name}"
        );
    }
    let registrations = (0..33)
        .map(|index| valid.replace("language.rust", &format!("language.custom{index}")))
        .collect::<String>();
    assert!(
        rev::config::parse(
            std::path::Path::new("config.toml"),
            registrations.as_bytes()
        )
        .is_err()
    );
    let bounded = (0..32)
        .map(|index| valid.replace("language.rust", &format!("language.custom{index}")))
        .collect::<String>();
    assert!(rev::config::parse(std::path::Path::new("config.toml"), bounded.as_bytes()).is_ok());
}

// Relative versioned assets resolve under the resolved XDG data namespace;
// parent escapes and symlinks outside it fail instead of reading another root.
#[test]
fn relative_root() {
    let libraries = support::libraries();
    for case in ["inside", "parent", "symlink"] {
        let fixture = Fixture::new();
        let data = fixture.0.join("data/rev");
        let version = data.join("grammars/rust-v1");
        std::fs::create_dir_all(&version).unwrap();
        std::fs::copy(libraries.parser("rust"), version.join("parser.so")).unwrap();
        std::fs::write(version.join("highlights.scm"), "(identifier) @function").unwrap();
        std::fs::write(
            version.join("completions.scm"),
            "(function_item name: (identifier) @function)",
        )
        .unwrap();
        let parser = match case {
            "parent" => "../parser.so",
            "symlink" => {
                std::os::unix::fs::symlink(libraries.parser("rust"), version.join("escape.so"))
                    .unwrap();
                "rust-v1/escape.so"
            }
            _ => "rust-v1/parser.so",
        };
        let config = format!(
            "[language.rust.external]\nparser = {parser:?}\nsymbol = 'tree_sitter_rust'\nhighlights = 'rust-v1/highlights.scm'\ncompletions = 'rust-v1/completions.scm'\n"
        );
        let snapshot = configured(&fixture, &config).with_data_root(&data);
        let path = fixture.write("sample.rs", "fn original() {}\n");
        let mut editor = session(snapshot, &path);
        support::settle(&mut editor);
        let result = editor.buffer().unwrap().external_grammar_status().unwrap();
        assert_eq!(result.is_ok(), case == "inside", "{case}: {result:?}");
        assert_eq!(
            editor.buffer().unwrap().mode(),
            if case == "inside" { "External" } else { "Rust" }
        );
    }
}

// Files can appear after startup but before first use. Once admitted, editing
// registration/query files cannot replace that session's immutable generation.
#[test]
fn first_use_snapshot() {
    let libraries = support::libraries();
    let fixture = Fixture::new();
    let parser = fixture.0.join("later-v1.so");
    let config = rust_config(&fixture, &parser, "tree_sitter_rust");
    let snapshot = configured(&fixture, &config);
    let mut buffers = rev::buffer::Buffers::with_settings(snapshot).unwrap();
    std::fs::copy(libraries.parser("rust"), &parser).unwrap();
    let path = fixture.write("sample.rs", "fn original() {}\n");
    let id = buffers.visit(&path).unwrap();
    let view = rev::view::View::new(buffers.get(id).unwrap());
    let mut editor = rev::session::Session::new(
        buffers,
        view,
        rev::display::Size {
            width: 80,
            height: 16,
        },
    )
    .unwrap();
    support::settle(&mut editor);
    fixture.write("config.toml", "[language.rust]\ngrammar = 'plain'\n");
    fixture.write("highlights.scm", "(absent) @function");
    editor.poll_frame().unwrap();
    assert_eq!(
        editor.buffer().unwrap().external_grammar_status(),
        Some(Ok(()))
    );
    assert_eq!(editor.buffer().unwrap().mode(), "External");
    assert!(editor.remaining_input(std::time::Instant::now()).is_none());
}
