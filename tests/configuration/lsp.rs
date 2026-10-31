use super::fixture::Fixture;
use rev::config::{Lsp, parse, prepare};
use std::path::Path;

// Every new limit rejects zero/negative/excess input at the typed boundary;
// launch declarations reject traversal, empty requirements and non-JSON data.
#[test]
fn lsp_bounds() {
    for field in [
        "request_timeout_ms",
        "initialize_timeout_ms",
        "shutdown_timeout_ms",
        "max_message_bytes",
        "max_servers",
        "max_pending_requests",
    ] {
        for value in ["0", "-1", "9223372036854775807"] {
            let source = format!("lsp.{field} = {value}");
            assert!(
                parse(Path::new("config.toml"), source.as_bytes()).is_err(),
                "{source}"
            );
        }
    }
    for leaf in [
        "command = ''",
        "languages = []",
        "root_markers = ['../Cargo.toml']",
        "settings = { bad = nan }",
        "args = [\"\\u0000\"]",
    ] {
        let source = format!("[lsp.servers.rust]\n{leaf}");
        assert!(
            parse(Path::new("config.toml"), source.as_bytes()).is_err(),
            "{source}"
        );
    }
    assert!(!Lsp::default().enabled);
}

// Partial entries inherit across includes by leaf, while explicit empty arrays
// replace defaults. Unbundled language identities remain admissible.
#[test]
fn lsp_inheritance() {
    let fixture = Fixture::new();
    fixture.write("base.toml", "[lsp.servers.extra]\ncommand = 'ordinary-server'\nlanguages = ['custom']\nargs = ['--stdio']");
    let root = fixture.write("config.toml", "include = ['base.toml']\n[lsp.servers.extra]\nargs = []\n[lsp.servers.rust]\nroot_markers = []");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let policy = &startup.snapshot.settings().lsp;
    assert_eq!(policy.servers["extra"].command, "ordinary-server");
    assert_eq!(policy.servers["extra"].languages, ["custom"]);
    assert!(policy.servers["extra"].args.is_empty());
    assert!(policy.servers["rust"].root_markers.is_empty());
    assert!(!policy.enabled);
    assert_eq!(
        startup
            .snapshot
            .origin(&["lsp", "servers", "extra", "args"]),
        Some(root.as_path())
    );
}

// Incomplete new entries are allowed in a source but reject the final merged
// candidate; invalid prerequisites cannot silently become working defaults.
#[test]
fn lsp_candidate() {
    let fixture = Fixture::new();
    for source in [
        "[lsp.servers.extra]\nlanguages = ['custom']",
        "[lsp.servers.extra]\ncommand = 'server'",
    ] {
        let root = fixture.write("config.toml", source);
        assert!(prepare(&root).diagnostic.is_some(), "{source}");
    }
}
