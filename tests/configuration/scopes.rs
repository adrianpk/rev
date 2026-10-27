use std::path::Path;

use rev::config::parse;

// Language identity does not require a grammar or an override table. Exact and
// compound-suffix association patterns retain their source spelling and order-free map.
#[test]
fn admitted_scopes() {
    let cases = [
        ("global inheritance", "[language.go]\nindent_width = 2"),
        ("known grammar", "[language.rust]\ngrammar = 'rust'"),
        ("plain grammar", "[language.other-2]\ngrammar = 'plain'"),
        (
            "compound suffix",
            "[files.associations]\n'*.test.ts' = 'test-typescript'",
        ),
        (
            "exact Unicode basename",
            "[files.associations]\n'界.rs' = 'rust'",
        ),
        (
            "user language",
            "[files.associations]\n'Gemfile' = 'custom-ruby'",
        ),
        ("editor command", "[keys.editor]\n'M-j' = 'next-line'"),
        (
            "transient command",
            "[keys.minibuffer]\n'C-g' = 'keyboard-quit'",
        ),
        ("tree command", "[keys.tree]\n'RET' = 'tree-open'"),
    ];
    for (name, source) in cases {
        assert!(
            parse(Path::new("scopes.toml"), source.as_bytes()).is_ok(),
            "{name}"
        );
    }
}

// Global-only leaves, unsupported grammar installations, malformed patterns
// and non-command binding values reject with a decoded dynamic key path.
#[test]
fn rejected_scopes() {
    let cases = [
        (
            "language ID",
            "[language.Go]\nindent_width = 2",
            vec!["language", "Go"],
        ),
        (
            "language width",
            "[language.go]\nindent_width = 0",
            vec!["language", "go", "indent_width"],
        ),
        (
            "global leaf",
            "[language.go]\npage_overlap = 2",
            vec!["language", "go", "page_overlap"],
        ),
        (
            "nested global table",
            "[language.go.files]\nbom = true",
            vec!["language", "go", "files"],
        ),
        (
            "grammar",
            "[language.go]\ngrammar = 'go'",
            vec!["language", "go", "grammar"],
        ),
        (
            "association ID",
            "[files.associations]\n'*.rs' = 'Rust'",
            vec!["files", "associations", "*.rs"],
        ),
        (
            "pattern",
            "[files.associations]\n'src/*.rs' = 'rust'",
            vec!["files", "associations", "src/*.rs"],
        ),
        (
            "empty command",
            "[keys.editor]\n'M-j' = ''",
            vec!["keys", "editor", "M-j"],
        ),
        (
            "unknown command",
            "[keys.tree]\n'RET' = 'shell-command'",
            vec!["keys", "tree", "RET"],
        ),
        (
            "unbinding",
            "[keys.minibuffer]\n'C-g' = false",
            vec!["keys", "minibuffer", "C-g"],
        ),
    ];
    for (name, source, key) in cases {
        let error = parse(Path::new("scopes.toml"), source.as_bytes()).unwrap_err();
        assert_eq!(error.key, key, "{name}: {error}");
        assert!(error.span.is_some(), "{name}");
    }
}

// Invalid pattern diagnostics point at the key, while invalid command names
// point at the value. Neither requires guessing source locations from text.
#[test]
fn dynamic_locations() {
    let cases = [
        (
            "pattern",
            "[files.associations]\n'src/*.rs' = 'rust'",
            "'src/*.rs'",
        ),
        (
            "language ID",
            "[language.'Rust']\nindent_width = 2",
            "'Rust'",
        ),
        (
            "binding value",
            "[keys.editor]\n'M-j' = 'unbound'",
            "'unbound'",
        ),
    ];
    for (name, source, expected) in cases {
        let error = parse(Path::new("locations.toml"), source.as_bytes()).unwrap_err();
        assert_eq!(&source[error.span.unwrap()], expected, "{name}");
    }
}
