use super::fixture::Fixture;
use rev::config::{Settings, Shell, prepare};

// Includes preserve assigning sources and replace args as a whole, including
// an explicitly empty program that selects the startup environment fallback.
#[test]
fn shell_layers() {
    let f = Fixture::new();
    let base = f.write("base.toml", "[shell]\nprogram='/bin/sh'\nargs=['-c','--']\ntimeout_seconds=9\nmax_input_bytes=7\nmax_output_bytes=8");
    let root = f.write(
        "config.toml",
        "include=['base.toml']\n[shell]\nprogram=''\nargs=['-c']",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(
        startup.snapshot.settings().shell,
        Shell {
            program: String::new(),
            args: vec!["-c".into()],
            timeout_seconds: 9,
            max_input_bytes: 7,
            max_output_bytes: 8,
        }
    );
    for (key, source) in [
        ("program", &root),
        ("args", &root),
        ("timeout_seconds", &base),
        ("max_input_bytes", &base),
        ("max_output_bytes", &base),
    ] {
        assert_eq!(
            startup.snapshot.origin(&["shell", key]),
            Some(source.as_path()),
            "{key}"
        );
    }
    assert_eq!(
        Shell::default(),
        Shell {
            program: String::new(),
            args: vec!["-c".into()],
            timeout_seconds: 600,
            max_input_bytes: 8_388_608,
            max_output_bytes: 8_388_608
        }
    );
}

// A bad earlier layer rejects the complete candidate even when a later layer
// would replace it. Byte, aggregate, count, numeric and unknown-field bounds apply.
#[test]
fn shell_admission() {
    let f = Fixture::new();
    let quoted = |text: &str| serde_json::to_string(text).unwrap();
    let cases = [
        ("empty program", "program=''".into(), true),
        (
            "max program",
            format!("program={}", quoted(&"s".repeat(4096))),
            true,
        ),
        (
            "excess program",
            format!("program={}", quoted(&"s".repeat(4097))),
            false,
        ),
        ("nul program", "program=\"\\u0000\"".into(), false),
        ("empty args", "args=[]".into(), false),
        (
            "max args",
            format!("args={}", serde_json::to_string(&vec![""; 32]).unwrap()),
            true,
        ),
        (
            "excess args",
            format!("args={}", serde_json::to_string(&vec![""; 33]).unwrap()),
            false,
        ),
        (
            "max aggregate",
            format!(
                "args={}",
                serde_json::to_string(&vec!["a".repeat(4096); 8]).unwrap()
            ),
            true,
        ),
        (
            "excess aggregate",
            format!(
                "args={}",
                serde_json::to_string(&vec!["a".repeat(4096); 9]).unwrap()
            ),
            false,
        ),
        ("nul arg", "args=[\"\\u0000\"]".into(), false),
        ("low deadline", "timeout_seconds=0".into(), false),
        ("high deadline", "timeout_seconds=601".into(), false),
        ("negative input", "max_input_bytes=-1".into(), false),
        ("excess output", "max_output_bytes=16777217".into(), false),
        ("unknown", "interactive=true".into(), false),
    ];
    for (name, leaf, valid) in cases {
        f.write("base.toml", &format!("editor.tab_width=3\n[shell]\n{leaf}"));
        let root = f.write("config.toml", "include=['base.toml']\n[shell]\nprogram='/bin/sh'\nargs=['-c']\ntimeout_seconds=2\nmax_input_bytes=8\nmax_output_bytes=8");
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
