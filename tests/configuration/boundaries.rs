use std::path::Path;

use rev::config::{Rgb, parse};

fn admit(key: &str, value: &str, accepted: bool) {
    let source = format!("{key} = {value}");
    let result = parse(Path::new("bounds.toml"), source.as_bytes());
    assert_eq!(result.is_ok(), accepted, "{source}: {result:?}");
    if let Err(error) = result {
        assert_eq!(error.key, key.split('.').collect::<Vec<_>>(), "{source}");
        assert!(error.span.is_some(), "{source}");
    }
}

// Every admitted integer boundary is inclusive. Zero is legal only for overlap
// and syntax cutoff; a one-byte soft history target remains a valid preference.
#[test]
fn integer_bounds() {
    let cases = [
        ("version", 1, 1),
        ("editor.indent_width", 1, 32),
        ("editor.tab_width", 1, 32),
        ("scroll.page_overlap", 0, 1000),
        ("minibuffer.candidate_limit", 1, 32),
        ("minibuffer.history_limit", 1, 4096),
        ("minibuffer.input_limit", 1, 65536),
        ("history.group_limit", 1, 65536),
        ("history.payload_target_bytes", 1, 1_073_741_824),
        ("kill_ring.capacity", 1, 4096),
        ("tree.width", 1, 512),
        ("syntax.max_bytes", 0, 1_073_741_824),
        ("language.go.indent_width", 1, 32),
        ("language.go.tab_width", 1, 32),
    ];
    for (key, minimum, maximum) in cases {
        admit(key, &minimum.to_string(), true);
        admit(key, &maximum.to_string(), true);
        admit(key, &(minimum - 1).to_string(), false);
        admit(key, &(maximum + 1).to_string(), false);
    }
}

// NaNs, infinities and overflow cannot become geometry preferences. Values
// immediately outside each admitted interval fail rather than being clamped here.
#[test]
fn float_bounds() {
    let cases = [
        ("windows.split_ratio", "0.1", "0.9", "0.099999", "0.900001"),
        ("tree.max_fraction", "0.1", "0.5", "0.099999", "0.500001"),
    ];
    for (key, minimum, maximum, below, above) in cases {
        admit(key, minimum, true);
        admit(key, maximum, true);
        admit(key, below, false);
        admit(key, above, false);
        nonfinite(key);
    }
}

fn nonfinite(key: &str) {
    for value in ["nan", "+nan", "-nan", "inf", "+inf", "-inf", "1e400"] {
        admit(key, value, false);
    }
}

// Validate every semantic role, so forgetting one field cannot admit malformed
// RGB. Channel parsing must not accept numeric plus signs as hexadecimal digits.
#[test]
fn palette_roles() {
    let roles = [
        "background",
        "foreground",
        "cursor",
        "selection",
        "selection_text",
        "search",
        "search_text",
        "status",
        "status_text",
        "status_inactive",
        "border",
        "border_active",
        "tree",
        "tree_selected",
        "tree_selected_text",
        "message",
        "warning",
        "error",
        "syntax.keyword",
        "syntax.string",
        "syntax.comment",
        "syntax.type",
        "syntax.function",
        "syntax.number",
        "syntax.punctuation",
    ];
    for role in roles {
        let key = format!("palette.{role}");
        admit(&key, "'#A0b1C2'", true);
        admit(&key, "'#+1+2+3'", false);
    }
    for value in [
        "''",
        "'#123'",
        "'#1234567'",
        "'123456'",
        "'#00gg00'",
        "'#界界'",
        "'default'",
        "'transparent'",
    ] {
        admit("palette.background", value, false);
    }
    assert_eq!(Rgb::parse("#A0b1C2"), Some(Rgb(0xa0, 0xb1, 0xc2)));
}

// Distinct enum choices are admitted through their documented TOML spellings;
// unsupported future controls cannot appear through nested language tables.
#[test]
fn enum_values() {
    let cases: [(&str, &[&str]); 5] = [
        ("editor.indent_style", &["'spaces'", "'tabs'"]),
        (
            "minibuffer.match",
            &["'prefix-then-substring'", "'prefix'", "'substring'"],
        ),
        ("tree.root", &["'auto'", "'cwd'", "'file-parent'"]),
        ("files.newline", &["'lf'", "'crlf'"]),
        ("language.rust.grammar", &["'rust'", "'plain'"]),
    ];
    for (key, values) in cases {
        for value in values {
            admit(key, value, true);
        }
        admit(key, "'unsupported'", false);
    }
}

// All contexts reject attempted unbinding, wrong value types and unknown names.
// Exact duplicate decoded keys remain TOML errors before semantic validation.
#[test]
fn binding_values() {
    for context in ["editor", "minibuffer", "tree"] {
        binding_context(context);
    }
}

fn binding_context(context: &str) {
    let key = format!("keys.{context}.F");
    admit(&key, "'keyboard-quit'", true);
    for value in [
        "false",
        "0",
        "[]",
        "{}",
        "''",
        "'false'",
        "'none'",
        "'unbound'",
        "'run-shell'",
    ] {
        admit(&key, value, false);
    }
    let source = format!("[keys.{context}]\n'M-j' = 'next-line'\n\"M-j\" = 'next-line'");
    assert!(parse(Path::new("duplicate.toml"), source.as_bytes()).is_err());
}

// Parser rejection must retain a source and available span while withholding
// the candidate. A syntax failure need not invent a semantic key path.
#[test]
fn malformed_sources() {
    let cases = [
        ("syntax", "editor = { indent_width ="),
        ("duplicate root", "version = 1\nversion = 1"),
        ("duplicate table", "[editor]\n[editor]"),
        (
            "duplicate dotted key",
            "editor.tab_width = 4\n[editor]\ntab_width = 8",
        ),
        ("integer overflow", "editor.tab_width = 9223372036854775808"),
        ("invalid scalar", "editor.indent_width = 0x"),
        ("trailing input", "version = 1 garbage"),
    ];
    for (name, source) in cases {
        let error = parse(Path::new("malformed.toml"), source.as_bytes()).unwrap_err();
        assert_eq!(error.source, Path::new("malformed.toml"), "{name}");
        assert!(error.span.is_some(), "{name}");
        assert!(error.position.is_some(), "{name}");
        assert!(!error.message.is_empty(), "{name}");
    }
}

// Encoding failure spans cover either a bad byte or an incomplete final scalar;
// preceding valid Unicode contributes one column per scalar in the diagnostic.
#[test]
fn utf8_locations() {
    let prefix = "# 界\nsyntax.enabled = ";
    for (name, suffix) in [("invalid", &[0xff][..]), ("incomplete", &[0xe2, 0x82][..])] {
        let mut bytes = prefix.as_bytes().to_vec();
        bytes.extend_from_slice(suffix);
        let error = parse(Path::new("encoding.toml"), &bytes).unwrap_err();
        assert_eq!(error.span, Some(prefix.len()..bytes.len()), "{name}");
        let position = error.position.unwrap();
        assert_eq!(position.line, 2, "{name}");
        assert_eq!(
            position.column,
            "syntax.enabled = ".chars().count() + 1,
            "{name}"
        );
        assert_eq!(error.message, "configuration must be UTF-8", "{name}");
    }
}

// Pattern grammar permits exact Unicode basenames and compound suffixes, but
// rejects glob syntax and paths. Language IDs use the same grammar in both scopes.
#[test]
fn identifier_shapes() {
    for pattern in [
        "", "*", "*.", "foo*.rs", "a?b", "[ab]", "a/b", "a\\b", "a\nb",
    ] {
        let source = format!("[files.associations]\n{pattern:?} = 'rust'");
        let error = parse(Path::new("pattern.toml"), source.as_bytes()).unwrap_err();
        assert_eq!(error.key, ["files", "associations", pattern]);
    }
    for id in ["", "Rust", "2rust", "rust.lang", "rust_lang", "é"] {
        invalid_id(id);
    }
}

fn invalid_id(id: &str) {
    let source = format!("[language.{id:?}]\nindent_width = 2");
    let error = parse(Path::new("id.toml"), source.as_bytes()).unwrap_err();
    assert_eq!(error.key, ["language", id]);
    let source = format!("[files.associations]\n'*.rs' = {id:?}");
    let error = parse(Path::new("id.toml"), source.as_bytes()).unwrap_err();
    assert_eq!(error.key, ["files", "associations", "*.rs"]);
}
