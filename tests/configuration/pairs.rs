use std::path::Path;

use rev::buffer::Buffers;
use rev::config::{Document, Grammar, Pairs, Settings, Snapshot, default_template, parse, prepare};

use super::fixture::Fixture;

const LEAVES: [&str; 5] = [
    "enabled",
    "wrap_selection",
    "delete_empty",
    "skip_closing",
    "quotes",
];

fn values(policy: Pairs) -> [bool; 5] {
    [
        policy.enabled,
        policy.wrap_selection,
        policy.delete_empty,
        policy.skip_closing,
        policy.quotes,
    ]
}

fn text(buffer: &rev::buffer::Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Empty groups cannot freeze defaults into a language partial. The shipped
// template must explicitly declare exactly the five approved true leaves.
#[test]
fn defaults() {
    let f = Fixture::new();
    for (name, source) in [
        ("absent", ""),
        ("empty groups", "[pairs]\n[language.rust.pairs]"),
        ("packaged", default_template()),
    ] {
        let root = f.write("config.toml", source);
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none(), "{name}");
        assert_eq!(
            values(startup.snapshot.settings().pairs),
            [true; 5],
            "{name}"
        );
        assert_eq!(values(startup.snapshot.pairs("rust")), [true; 5], "{name}");
    }
    let input = parse(Path::new("empty.toml"), b"[language.rust.pairs]").unwrap();
    assert_eq!(input.pairs, Document::default().pairs);
    assert_eq!(input.language["rust"].pairs, input.pairs);
    assert_eq!(values(Snapshot::default().pairs("plain")), [true; 5]);
    let raw: toml::Table = toml::from_str(default_template()).unwrap();
    let table = raw["pairs"].as_table().unwrap();
    assert_eq!(table.len(), 5);
    for leaf in LEAVES {
        assert_eq!(table[leaf].as_bool(), Some(true), "{leaf}");
    }
}

// Each explicit false overrides a true include, changes only its own leaf and
// keeps source provenance. Resolving again must not mutate an older snapshot.
#[test]
fn false_overrides() {
    let f = Fixture::new();
    for (index, leaf) in LEAVES.into_iter().enumerate() {
        let base = f.write("base.toml", &format!("pairs.{leaf}=true"));
        let source = format!("include=['base.toml']\npairs.{leaf}=false\neditor.tab_width=8");
        let root = f.write("config.toml", &source);
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none(), "{leaf}");
        let mut expected = [true; 5];
        expected[index] = false;
        assert_eq!(
            values(startup.snapshot.pairs("unknown")),
            expected,
            "{leaf}"
        );
        assert_eq!(startup.snapshot.settings().editor.tab_width, 8);
        let buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
        assert_eq!(
            values(buffers.get(buffers.initial()).unwrap().pair_policy()),
            expected,
            "buffer {leaf}"
        );
        assert_eq!(
            startup.snapshot.origin(&["pairs", leaf]),
            Some(root.as_path())
        );
        assert_eq!(
            std::fs::read_to_string(&base).unwrap(),
            format!("pairs.{leaf}=true")
        );
        assert_eq!(std::fs::read_to_string(&root).unwrap(), source);
        f.write("config.toml", &format!("pairs.{leaf}=true"));
        assert_eq!(values(prepare(&root).snapshot.pairs("unknown")), [true; 5]);
        assert_eq!(
            values(startup.snapshot.pairs("unknown")),
            expected,
            "frozen {leaf}"
        );
    }
}

// A later true must replace only its global false leaf, retaining false siblings
// from includes; an empty language table still inherits that complete result.
#[test]
fn global_layers() {
    let f = Fixture::new();
    let all_false = LEAVES.map(|leaf| format!("{leaf}=false\n")).concat();
    let base = f.write("base.toml", &format!("[pairs]\n{all_false}"));
    for (index, leaf) in LEAVES.into_iter().enumerate() {
        let root = f.write(
            "config.toml",
            &format!("include=['base.toml']\npairs.{leaf}=true\n[language.rust.pairs]"),
        );
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none(), "{leaf}");
        let mut expected = [false; 5];
        expected[index] = true;
        assert_eq!(
            values(startup.snapshot.settings().pairs),
            expected,
            "{leaf}"
        );
        assert_eq!(values(startup.snapshot.pairs("rust")), expected, "{leaf}");
        for sibling in LEAVES {
            let origin = if sibling == leaf { &root } else { &base };
            assert_eq!(
                startup.snapshot.origin(&["pairs", sibling]),
                Some(origin.as_path()),
                "{leaf}/{sibling}"
            );
        }
    }
}

// Language resolution happens after global composition. Empty language groups
// inherit the final false while an included language false survives a later
// global true; unavailable grammars and disabled colors do not erase identity.
#[test]
fn language_layers() {
    let f = Fixture::new();
    for (index, leaf) in LEAVES.into_iter().enumerate() {
        let base = f.write("base.toml", &format!("language.rust.pairs.{leaf}=false"));
        let source = format!(
            "include=['base.toml']\nsyntax.enabled=false\npairs.{leaf}=true\nlanguage.rust.grammar='plain'\n[language.custom.pairs]"
        );
        let root = f.write("config.toml", &source);
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none(), "{leaf}");
        let mut expected = [true; 5];
        expected[index] = false;
        assert_eq!(values(startup.snapshot.pairs("rust")), expected, "{leaf}");
        assert_eq!(values(startup.snapshot.pairs("custom")), [true; 5]);
        assert_eq!(startup.snapshot.grammar("rust"), Grammar::Plain);
        assert_eq!(startup.snapshot.grammar("custom"), Grammar::Plain);
        assert_eq!(
            startup
                .snapshot
                .origin(&["language", "rust", "pairs", leaf]),
            Some(base.as_path())
        );
        assert_eq!(
            startup.snapshot.origin(&["pairs", leaf]),
            Some(root.as_path())
        );
        assert_eq!(
            startup
                .snapshot
                .origin(&["language", "custom", "pairs", leaf]),
            None
        );
        f.write(
            "config.toml",
            &format!("pairs.{leaf}=false\n[language.custom.pairs]"),
        );
        assert_eq!(
            values(prepare(&root).snapshot.pairs("custom")),
            expected,
            "inherit {leaf}"
        );
    }
}

// Overriding one language leaf must retain included false siblings and keep
// each dynamic origin separate, including a hyphenated custom language ID.
#[test]
fn language_precedence() {
    let f = Fixture::new();
    let all_false = LEAVES.map(|leaf| format!("{leaf}=false\n")).concat();
    let base = f.write(
        "base.toml",
        &format!("[language.custom-code.pairs]\n{all_false}"),
    );
    for (index, leaf) in LEAVES.into_iter().enumerate() {
        let root = f.write(
            "config.toml",
            &format!("include=['base.toml']\nlanguage.custom-code.pairs.{leaf}=true"),
        );
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none(), "{leaf}");
        let mut expected = [false; 5];
        expected[index] = true;
        assert_eq!(
            values(startup.snapshot.pairs("custom-code")),
            expected,
            "{leaf}"
        );
        for sibling in LEAVES {
            let origin = if sibling == leaf { &root } else { &base };
            assert_eq!(
                startup
                    .snapshot
                    .origin(&["language", "custom-code", "pairs", sibling]),
                Some(origin.as_path()),
                "{leaf}/{sibling}"
            );
        }
        assert_eq!(startup.snapshot.origin(&["pairs", leaf]), None);
        assert_eq!(values(startup.snapshot.pairs("rust")), [true; 5]);
    }
}

// Every Boolean rejects strings, integers, arrays and tables at both scopes,
// with a full decoded diagnostic key and a byte/line location.
#[test]
fn malformed_leaves() {
    for prefix in ["pairs", "language.rust.pairs"] {
        for leaf in LEAVES {
            reject_types(&format!("{prefix}.{leaf}"));
        }
    }
}

fn reject_types(key: &str) {
    for (name, value) in [
        ("string", "'false'"),
        ("integer", "0"),
        ("array", "[]"),
        ("table", "{}"),
    ] {
        let source = format!("{key}={value}");
        let error = parse(Path::new("invalid.toml"), source.as_bytes()).unwrap_err();
        assert_eq!(
            error.key,
            key.split('.').collect::<Vec<_>>(),
            "{name}: {source}"
        );
        assert!(error.span.is_some(), "{source}");
        assert!(error.position.is_some(), "{source}");
    }
}

// Future visual controls, malformed groups and invalid related language values
// must reject the candidate even if the root would mask an included bad leaf.
#[test]
fn candidate_rejection() {
    let f = Fixture::new();
    for (name, source) in [
        ("global future style", "pairs.highlight='depth'"),
        ("global future pulse", "pairs.pulse_ms=250"),
        (
            "language future style",
            "language.rust.pairs.highlight='off'",
        ),
        ("language future pulse", "language.rust.pairs.pulse_ms=50"),
        ("unknown leaf", "pairs.unknown=true"),
        ("future palette", "palette.pairs.match_text='#123456'"),
        ("global group", "pairs=false"),
        ("global sequence", "pairs=[]"),
        ("language group", "language.rust.pairs=[]"),
        ("invalid language", "language.Rust.pairs.enabled=false"),
        (
            "invalid grammar",
            "language.rust.grammar='future'\nlanguage.rust.pairs.enabled=false",
        ),
        (
            "invalid sibling",
            "language.rust.indent_width=0\nlanguage.rust.pairs.enabled=false",
        ),
        ("syntax", "[pairs\nenabled=false"),
    ] {
        let base = f.write("base.toml", &format!("editor.tab_width=8\n{source}"));
        let root = f.write(
            "config.toml",
            "include=['base.toml']\npairs.enabled=false\nlanguage.rust.pairs.enabled=true",
        );
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
        assert_eq!(
            std::fs::read_to_string(base).unwrap(),
            format!("editor.tab_width=8\n{source}")
        );
    }
}

// A later valid assignment cannot hide a bad pair Boolean in an include. The
// fallback also withholds unrelated valid settings and retains the source error.
#[test]
fn masked_invalid() {
    let f = Fixture::new();
    for prefix in ["pairs", "language.rust.pairs"] {
        for leaf in LEAVES {
            let base = f.write("base.toml", &format!("{prefix}.{leaf}='false'"));
            let source =
                format!("include=['base.toml']\neditor.tab_width=8\n{prefix}.{leaf}=false");
            let root = f.write("config.toml", &source);
            let startup = prepare(&root);
            let diagnostic = startup.diagnostic.unwrap();
            assert_eq!(diagnostic.source, base, "{prefix}.{leaf}");
            assert_eq!(startup.snapshot.settings(), &Settings::default());
            assert_eq!(std::fs::read_to_string(root).unwrap(), source);
        }
    }
}

// Save-as changes the native language base without touching policy overrides,
// source bytes, source revision, undo history or unrelated buffer settings.
#[test]
fn language_lifecycle() {
    let f = Fixture::new();
    for (name, toggle_count, expected_enabled) in [
        ("base", 0, true),
        ("explicit on", 1, true),
        ("explicit off", 2, false),
    ] {
        let root = f.write("config.toml", "syntax.enabled=false\neditor.tab_width=8\nlanguage.rust.grammar='plain'\nlanguage.rust.pairs.enabled=false\nlanguage.rust.pairs.quotes=false\nlanguage.go.pairs.wrap_selection=false");
        let snapshot = prepare(&root).snapshot;
        let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
        let id = buffers.visit(&f.write("source.rs", "a界\r\n()")).unwrap();
        let b = buffers.get_mut(id).unwrap();
        assert_eq!(values(b.pair_policy()), [false, true, true, true, false]);
        let before = b.stamp();
        for _ in 0..toggle_count {
            b.toggle_auto_pairs();
        }
        b.toggle_soft_wrap();
        b.apply_settings(&snapshot);
        let target = f.0.join(format!("{toggle_count}.go"));
        let save = buffers.prepare_save(id, Some(&target)).unwrap();
        assert!(buffers.publish_save(save).is_complete(), "{name}");
        let b = buffers.get(id).unwrap();
        assert_eq!(b.language(&snapshot), "go", "{name}");
        assert_eq!(
            values(b.pair_policy()),
            [expected_enabled, false, true, true, true],
            "{name}"
        );
        assert_eq!(b.stamp(), before, "{name}");
        assert_eq!(text(b), "a界\r\n()", "{name}");
        assert!(b.history().is_empty(), "{name}");
        assert!(!b.is_modified(), "{name}");
        assert!(!b.soft_wrap(), "{name}");
        assert_eq!(b.policy().tab_width, 8, "{name}");
        assert_eq!(std::fs::read(target).unwrap(), "a界\r\n()".as_bytes());
        let fresh = buffers.create("").unwrap();
        assert_eq!(values(buffers.get(fresh).unwrap().pair_policy()), [true; 5]);
    }
}

// Splits reference one canonical buffer. A settings application and a toggle
// cannot create per-view policy copies or write the startup configuration.
#[test]
fn shared_buffer() {
    let f = Fixture::new();
    let source = "pairs.enabled=false\npairs.quotes=false";
    let root = f.write("config.toml", source);
    let snapshot = prepare(&root).snapshot;
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.create("literal").unwrap();
    let view = rev::view::View::with_settings(buffers.get(id).unwrap(), &snapshot);
    let mut windows = rev::window::Windows::new(view).unwrap();
    windows
        .split(
            rev::window::Split::Right,
            rev::view::Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap();
    assert!(buffers.get_mut(id).unwrap().toggle_auto_pairs());
    buffers.get_mut(id).unwrap().apply_settings(&snapshot);
    assert_eq!(windows.iter().len(), 2);
    for (_, view) in windows.iter() {
        assert_eq!(view.point.stamp.buffer, id);
        assert_eq!(
            values(buffers.get(view.point.stamp.buffer).unwrap().pair_policy()),
            [true, true, true, true, false]
        );
    }
    assert_eq!(std::fs::read_to_string(root).unwrap(), source);
    assert!(buffers.get(id).unwrap().history().is_empty());
}

// Policy changes preserve an existing edit, mark, points, read-only state and
// both history directions. A toggle cannot publish or discard source history.
#[test]
fn source_history() {
    use rev::buffer::edit::{History, Point, Replacement};
    use rev::coordinates::Scalar;

    let f = Fixture::new();
    let root = f.write(
        "config.toml",
        "pairs.enabled=false\npairs.delete_empty=false",
    );
    let snapshot = prepare(&root).snapshot;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\n").unwrap();
    let b = buffers.get_mut(id).unwrap();
    let mut points = [Point {
        stamp: b.stamp(),
        position: Scalar(1),
    }];
    let edits = [Replacement {
        source: b.span(1..1).unwrap(),
        text: "()".into(),
    }];
    let prepared = b.prepare(&edits, &points, None).unwrap();
    b.publish(prepared, &mut points).unwrap();
    b.set_mark(Some(Scalar(3))).unwrap();
    let before = (b.stamp(), b.state(), b.mark(), points);
    b.set_readonly(true);
    b.apply_settings(&snapshot);
    assert!(b.toggle_auto_pairs());
    assert_eq!(values(b.pair_policy()), [true, true, false, true, true]);
    assert_eq!((b.stamp(), b.state(), b.mark(), points), before);
    assert!(b.is_readonly());
    assert_eq!(text(b), "a()界\r\n");
    assert_eq!(b.history().len(), 1);
    assert!(b.is_modified());
    b.set_readonly(false);
    b.navigate(History::Undo, &mut points, None).unwrap();
    assert_eq!(text(b), "a界\r\n");
    b.apply_settings(&snapshot);
    assert_eq!(b.redo_history().len(), 1);
    assert!(b.pair_policy().enabled);
    b.navigate(History::Redo, &mut points, None).unwrap();
    assert_eq!(text(b), "a()界\r\n");
    assert_eq!(b.history().len(), 1);
}
