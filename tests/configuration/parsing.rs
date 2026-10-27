use std::path::Path;

use rev::config::parse;

// Version omission admits schema 1 without filling absent leaves. TOML 1.1's
// multiline inline tables and trailing commas exercise the selected parser.
#[test]
fn admitted_sources() {
    let cases = [
        ("empty", ""),
        ("version", "version = 1"),
        ("empty includes", "include = []"),
        ("false and zero", "[syntax]\nenabled = false\nmax_bytes = 0"),
        ("TOML 1.1", "editor = {\nindent_width = 2,\n}"),
        ("partial language", "[language.go]\nindent_width = 2"),
    ];
    for (name, source) in cases {
        let document = parse(Path::new("settings.toml"), source.as_bytes())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(document.editor.tab_width, None, "{name}");
    }
}

// One invalid field rejects the whole document with its source, key and byte
// location. Unknown fields and wrong types cannot silently select defaults.
#[test]
fn rejected_sources() {
    let cases = [
        ("version", "version = 2", vec!["version"]),
        ("unknown root", "unsupported = true", vec!["unsupported"]),
        (
            "unknown leaf",
            "[editor]\nmargin = 2",
            vec!["editor", "margin"],
        ),
        (
            "integer type",
            "[editor]\nindent_width = '2'",
            vec!["editor", "indent_width"],
        ),
        (
            "enum",
            "[editor]\nindent_style = 'automatic'",
            vec!["editor", "indent_style"],
        ),
        (
            "bound",
            "[editor]\ntab_width = 0",
            vec!["editor", "tab_width"],
        ),
        (
            "nonfinite",
            "[windows]\nsplit_ratio = nan",
            vec!["windows", "split_ratio"],
        ),
        (
            "color",
            "[palette]\nbackground = '#123'",
            vec!["palette", "background"],
        ),
        (
            "syntax role",
            "[palette.syntax]\noperator = '#ffffff'",
            vec!["palette", "syntax", "operator"],
        ),
    ];
    for (name, source, key) in cases {
        let error = parse(Path::new("settings.toml"), source.as_bytes()).unwrap_err();
        assert_eq!(error.source, Path::new("settings.toml"), "{name}");
        assert_eq!(error.key, key, "{name}: {error}");
        assert!(error.span.is_some(), "{name}");
        assert!(error.position.is_some(), "{name}");
    }
}

// Byte spans remain byte coordinates, while diagnostic columns count scalars
// even when a multibyte association basename appears earlier on the same line.
#[test]
fn unicode_location() {
    let source = "files = { associations = { '界.rs' = 'rust' }, bom = 1 }";
    let error = parse(Path::new("unicode.toml"), source.as_bytes()).unwrap_err();
    let start = source.rfind('1').unwrap();
    assert_eq!(error.key, ["files", "bom"]);
    assert_eq!(error.span, Some(start..start + 1));
    let position = error.position.unwrap();
    assert_eq!(position.line, 1);
    assert_eq!(position.column, source[..start].chars().count() + 1);
    assert!(error.to_string().contains("unicode.toml:1:"));
    assert!(!error.to_string().contains("界.rs"));
}
