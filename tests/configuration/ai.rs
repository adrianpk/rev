use std::path::Path;

use super::fixture::Fixture;
use rev::config::{Pi, parse, prepare};

// Partial AI settings merge by leaf, including explicit false and zero;
// the packaged template and immutable snapshot retain the complete defaults.
#[test]
fn pi_inheritance() {
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        "[ai.pi]\ncommand = '/opt/pi'\nmodel = 'provider/model'\nrequest_timeout_ms = 1000",
    );
    let root = f.write(
        "config.toml",
        "include = ['base.toml']\n[ai.pi]\npersist_sessions = false\nidle_timeout_ms = 0",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let pi = &startup.snapshot.settings().ai.pi;
    assert_eq!(pi.command, "/opt/pi");
    assert_eq!(pi.model.as_deref(), Some("provider/model"));
    assert!(!pi.persist_sessions);
    assert_eq!(pi.idle_timeout_ms, 0);
    assert_eq!(pi.request_timeout_ms, 1000);
    assert_eq!(pi.max_input_bytes, Pi::default().max_input_bytes);
    assert_eq!(
        startup.snapshot.origin(&["ai", "pi", "model"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["ai", "pi", "idle_timeout_ms"]),
        Some(root.as_path())
    );
}

// Reject invalid integer and string leaves at their declaring source, even
// when a later layer would otherwise hide them. No secret/argument map exists.
#[test]
fn pi_bounds() {
    for (field, min, max) in [
        ("idle_timeout_ms", 0, 3_600_000),
        ("request_timeout_ms", 1000, 1_800_000),
        ("max_input_bytes", 1, 4_194_304),
        ("max_result_bytes", 1, 1_048_576),
    ] {
        for value in [min - 1, max + 1] {
            let source = format!("ai.pi.{field} = {value}");
            assert!(
                parse(Path::new("config.toml"), source.as_bytes()).is_err(),
                "{source}"
            );
        }
        for value in [min, max] {
            let source = format!("ai.pi.{field} = {value}");
            assert!(
                parse(Path::new("config.toml"), source.as_bytes()).is_ok(),
                "{source}"
            );
        }
    }
    for leaf in [
        "command = ''",
        "command = '\u{0020}'",
        "model = ''",
        "args = []",
        "env = {}",
    ] {
        let source = format!("[ai.pi]\n{leaf}");
        assert!(
            parse(Path::new("config.toml"), source.as_bytes()).is_err(),
            "{leaf}"
        );
    }
    for (name, value) in [("command", "x".repeat(4097)), ("model", "x".repeat(257))] {
        let source = format!("ai.pi.{name} = '{value}'");
        assert!(parse(Path::new("config.toml"), source.as_bytes()).is_err());
    }
}
