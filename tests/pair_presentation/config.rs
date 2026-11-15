use rev::config::{PairHighlight, Snapshot, prepare};

use crate::fixture::Fixture;

#[path = "config/controls.rs"]
mod controls;
#[path = "config/palette.rs"]
mod palette;

fn settings(source: &str) -> Snapshot {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", source));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    startup.snapshot
}

// Empty groups retain absence and do not shadow global settings or provenance.
// Boundary durations use the same admission at global and language scope.
#[test]
fn defaults_and_inheritance() {
    for source in [
        "",
        "[pairs]\n[language.rust.pairs]",
        rev::config::default_template(),
    ] {
        let snapshot = settings(source);
        for language in ["plain", "rust", "custom-code"] {
            let policy = snapshot.pairs(language);
            assert_eq!(policy.highlight, PairHighlight::Depth);
            assert_eq!(policy.pulse_ms, 250);
            assert!(policy.enabled && policy.quotes);
        }
    }
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        "pairs.highlight='match'\npairs.pulse_ms=50\nlanguage.rust.pairs.highlight='pulse'",
    );
    let root = f.write("config.toml", "include=['base.toml']\npairs.highlight='off'\npairs.pulse_ms=1000\nsyntax.enabled=false\nlanguage.rust.grammar='plain'\nlanguage.custom-code.pairs.pulse_ms=250");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    for (language, style, duration) in [
        ("plain", PairHighlight::Off, 1000),
        ("rust", PairHighlight::Pulse, 1000),
        ("custom-code", PairHighlight::Off, 250),
    ] {
        let policy = startup.snapshot.pairs(language);
        assert_eq!(
            (policy.highlight, policy.pulse_ms),
            (style, duration),
            "{language}"
        );
        assert!(
            policy.enabled
                && policy.wrap_selection
                && policy.delete_empty
                && policy.skip_closing
                && policy.quotes
        );
    }
    for key in ["highlight", "pulse_ms"] {
        assert_eq!(
            startup.snapshot.origin(&["pairs", key]),
            Some(root.as_path())
        );
    }
    assert_eq!(
        startup
            .snapshot
            .origin(&["language", "rust", "pairs", "highlight"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup
            .snapshot
            .origin(&["language", "rust", "pairs", "pulse_ms"]),
        None
    );
    assert_eq!(
        startup
            .snapshot
            .origin(&["language", "custom-code", "pairs", "pulse_ms"]),
        Some(root.as_path())
    );
}

// Invalid included leaves reject their entire candidate before an overriding
// root can mask them. Fallback preserves compiled settings and source files.
#[test]
fn candidate_rejection() {
    let fixture = Fixture::new();
    let cases = [
        ("unknown style", "highlight='rainbow'"),
        ("wrong case", "highlight='Depth'"),
        ("style boolean", "highlight=true"),
        ("style sequence", "highlight=[]"),
        ("style table", "highlight={depth={}}"),
        ("short duration", "pulse_ms=49"),
        ("long duration", "pulse_ms=1001"),
        ("negative duration", "pulse_ms=-1"),
        ("fractional duration", "pulse_ms=250.0"),
        ("string duration", "pulse_ms='250'"),
        ("boolean duration", "pulse_ms=true"),
        ("duration sequence", "pulse_ms=[]"),
    ];
    for prefix in ["pairs", "language.rust.pairs"] {
        for (name, leaf) in cases {
            let source = format!("editor.tab_width=8\n{prefix}.{leaf}");
            let bad = fixture.write("bad.toml", &source);
            let root = fixture.write("config.toml", "include=['bad.toml']\npairs.highlight='depth'\npairs.pulse_ms=250\nlanguage.rust.pairs.highlight='off'\nlanguage.rust.pairs.pulse_ms=50");
            let startup = prepare(&root);
            let error = startup.diagnostic.expect(name);
            assert_eq!(error.source, bad, "{prefix}: {name}");
            assert!(
                error.span.is_some() && error.position.is_some(),
                "{prefix}: {name}"
            );
            assert_eq!(startup.snapshot.settings(), Snapshot::default().settings());
            assert_eq!(std::fs::read_to_string(bad).unwrap(), source);
        }
    }
}

// Later language leaves merge independently; missing siblings inherit the final
// globals and retain their own origin even for syntax-disabled custom modes.
#[test]
fn language_leaf_overrides() {
    let f = Fixture::new();
    let base = f.write("base.toml", "language.custom-code.pairs.highlight='match'\nlanguage.custom-code.pairs.pulse_ms=50\npairs.enabled=false");
    let root = f.write(
        "config.toml",
        "include=['base.toml']\npairs.pulse_ms=1000\nlanguage.custom-code.pairs.pulse_ms=250",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let policy = startup.snapshot.pairs("custom-code");
    assert_eq!(
        (policy.highlight, policy.pulse_ms, policy.enabled),
        (PairHighlight::Match, 250, false)
    );
    assert_eq!(
        startup
            .snapshot
            .origin(&["language", "custom-code", "pairs", "highlight"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup
            .snapshot
            .origin(&["language", "custom-code", "pairs", "pulse_ms"]),
        Some(root.as_path())
    );
}
