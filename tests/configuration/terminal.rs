use super::fixture::Fixture;
use rev::config::{Settings, Terminal, prepare};

// Terminal preferences inherit by leaf, including explicit empty argv, and
// retain the source of every accepted setting independently of shell commands.
#[test]
fn terminal_layers() {
    let f = Fixture::new();
    let base = f.write("base.toml", "[terminal]\nprogram='/bin/sh'\nargs=['-i']\npanel_height=7\nmax_sessions=3\nscrollback_lines=0\nmax_pending_input_bytes=4096\nmax_pending_output_bytes=8192");
    let root = f.write(
        "config.toml",
        "include=['base.toml']\n[terminal]\nprogram=''\nargs=[]",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(
        startup.snapshot.settings().terminal,
        Terminal {
            program: String::new(),
            args: vec![],
            panel_height: 7,
            max_sessions: 3,
            scrollback_lines: 0,
            max_pending_input_bytes: 4096,
            max_pending_output_bytes: 8192
        }
    );
    for (key, source) in [
        ("program", &root),
        ("args", &root),
        ("panel_height", &base),
        ("max_sessions", &base),
        ("scrollback_lines", &base),
        ("max_pending_input_bytes", &base),
        ("max_pending_output_bytes", &base),
    ] {
        assert_eq!(
            startup.snapshot.origin(&["terminal", key]),
            Some(source.as_path()),
            "{key}"
        );
    }
    assert_eq!(startup.snapshot.settings().shell, Settings::default().shell);
}

// Every source validates before overlay. A replaced bad value must still reject
// the complete candidate and cannot leak unrelated settings into defaults.
#[test]
fn terminal_admission() {
    let f = Fixture::new();
    let quoted = |text: &str| serde_json::to_string(text).unwrap();
    let mut cases = vec![
        ("empty args".to_owned(), "args=[]".to_owned(), true),
        (
            "max program".into(),
            format!("program={}", quoted(&"x".repeat(4096))),
            true,
        ),
        (
            "excess program".into(),
            format!("program={}", quoted(&"x".repeat(4097))),
            false,
        ),
        ("nul executable".into(), "program=\"\\u0000\"".into(), false),
        (
            "max args".into(),
            format!("args={}", serde_json::to_string(&vec![""; 32]).unwrap()),
            true,
        ),
        (
            "excess args".into(),
            format!("args={}", serde_json::to_string(&vec![""; 33]).unwrap()),
            false,
        ),
        (
            "max bytes".into(),
            format!("args=[{}]", quoted(&"x".repeat(16384))),
            true,
        ),
        (
            "excess bytes".into(),
            format!("args=[{}]", quoted(&"x".repeat(16385))),
            false,
        ),
        ("nul argument".into(), "args=[\"\\u0000\"]".into(), false),
        ("unknown setting".into(), "timeout_seconds=1".into(), false),
    ];
    for (key, min, max) in [
        ("panel_height", 3, 200),
        ("max_sessions", 1, 32),
        ("scrollback_lines", 0, 20000),
        ("max_pending_input_bytes", 4096, 1048576),
        ("max_pending_output_bytes", 4096, 1048576),
    ] {
        for (value, valid) in [(min - 1, false), (min, true), (max, true), (max + 1, false)] {
            cases.push((format!("{key}={value}"), format!("{key}={value}"), valid));
        }
    }
    for (name, leaf, valid) in cases {
        f.write(
            "base.toml",
            &format!("editor.tab_width=3\n[terminal]\n{leaf}"),
        );
        let root = f.write(
            "config.toml",
            "include=['base.toml']\n[terminal]\nprogram='/bin/sh'\nargs=[]",
        );
        let startup = prepare(&root);
        assert_eq!(
            startup.diagnostic.is_none(),
            valid,
            "{name}: {:?}",
            startup.diagnostic
        );
        if !valid {
            assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
        }
    }
}
