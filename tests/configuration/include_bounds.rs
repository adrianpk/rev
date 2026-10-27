use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

use rev::config::{Diagnostic, Snapshot, Sources, load, prepare};

use crate::fixture::Fixture;

fn padded(text: &str, length: usize) -> String {
    assert!(text.len() <= length);
    let mut padded = text.to_owned();
    padded.extend(std::iter::repeat_n(' ', length - text.len()));
    padded
}

// Active canonical and device/inode identities detect ordinary, symlink and
// hard-link cycles. Diagnostics retain both the declaration and reached source.
#[test]
fn alias_cycles() {
    for name in ["ordinary", "symlink", "hard link"] {
        let fixture = Fixture::new();
        let root = fixture.write(
            "config.toml",
            "editor.tab_width = 8\ninclude = ['alias.toml']",
        );
        let alias = fixture.0.join("alias.toml");
        cycle_alias(&fixture, name, &root, &alias);
        let error = checked_load(&root).unwrap_err();
        let diagnostic = error.to_string();
        assert!(diagnostic.contains("cycle"), "{name}: {error}");
        assert!(diagnostic.contains("config.toml"), "{name}");
        assert!(diagnostic.contains("alias.toml"), "{name}");
        assert_eq!(error.key, ["include"], "{name}");
    }
}

fn cycle_alias(fixture: &Fixture, name: &str, root: &std::path::Path, alias: &std::path::Path) {
    match name {
        "ordinary" => {
            fixture.write("alias.toml", "include = ['config.toml']");
        }
        "symlink" => symlink("config.toml", alias).unwrap(),
        "hard link" => fs::hard_link(root, alias).unwrap(),
        _ => unreachable!("known fixture kind"),
    }
}

// Depth counts edges, so root depth zero permits exactly 17 sources in a chain.
// The next edge is refused before trying to read another occurrence.
#[test]
fn depth_limit() {
    for (name, edges, accepted) in [("maximum", 16, true), ("one past", 17, false)] {
        let fixture = Fixture::new();
        chain(&fixture, edges);
        let result = checked_load(&fixture.0.join("node-0.toml"));
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        match result {
            Ok(graph) => {
                assert_eq!(graph.sources.len(), 17, "{name}");
                assert_eq!(graph.order, (0..17).rev().collect::<Vec<_>>(), "{name}");
            }
            Err(error) => assert!(error.message.contains("16 edges"), "{name}: {error}"),
        }
    }
}

fn chain(fixture: &Fixture, edges: usize) {
    for depth in 0..=edges {
        let text = if depth < edges {
            format!(
                "editor.tab_width = 8\ninclude = ['node-{}.toml']",
                depth + 1
            )
        } else {
            String::new()
        };
        fixture.write(&format!("node-{depth}.toml"), &text);
    }
}

// Repeated files are occurrences, not deduplicated cache hits. Empty repeated
// leaves still count, even though they contribute zero bytes to the read budget.
#[test]
fn occurrence_limit() {
    let fixture = Fixture::new();
    fixture.write("leaf.toml", "");
    for (name, count, accepted) in [("maximum", 63, true), ("one past", 64, false)] {
        let includes = std::iter::repeat_n("'leaf.toml'", count)
            .collect::<Vec<_>>()
            .join(",");
        let root = fixture.write(
            "config.toml",
            &format!("editor.tab_width = 8\ninclude = [{includes}]"),
        );
        let result = checked_load(&root);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        occurrence_result(result, accepted);
    }
}

fn occurrence_result(
    result: Result<rev::config::Sources, rev::config::Diagnostic>,
    accepted: bool,
) {
    if accepted {
        assert_eq!(result.unwrap().sources.len(), 64);
    } else {
        assert!(
            result
                .unwrap_err()
                .message
                .contains("64 source occurrences")
        );
    }
}

// Both root and explicit sources admit 256 KiB exactly, then refuse one extra
// byte using metadata and a bounded reader rather than an unbounded read.
#[test]
fn source_byte_limit() {
    for name in ["root", "include"] {
        source_sizes(name);
    }
}

fn source_sizes(name: &str) {
    let fixture = Fixture::new();
    for (extra, accepted) in [(0, true), (1, false)] {
        let path = fixture.write(
            "large.toml",
            &padded("editor.tab_width = 8\n", 256 * 1024 + extra),
        );
        let root = if name == "include" {
            fixture.write("config.toml", "include = ['large.toml']")
        } else {
            path
        };
        let result = checked_load(&root);
        assert_eq!(result.is_ok(), accepted, "{name}, extra {extra}");
        if let Err(error) = result {
            assert!(error.message.contains("256 KiB"), "{name}: {error}");
        }
    }
}

// Duplicate full-size leaves consume the total budget on each read. The root's
// own bytes count too, allowing exactly 2 MiB but refusing one additional byte.
#[test]
fn total_byte_limit() {
    let fixture = Fixture::new();
    let repeated = std::iter::repeat_n("'full.toml'", 7)
        .collect::<Vec<_>>()
        .join(",");
    let text = format!("editor.tab_width = 8\ninclude = [{repeated}, 'tail.toml', 'empty.toml']");
    let root = fixture.write("config.toml", &text);
    fixture.write("full.toml", &padded("", 256 * 1024));
    fixture.write("empty.toml", "");
    for (extra, accepted) in [(0, true), (1, false)] {
        fixture.write("tail.toml", &padded("", 256 * 1024 - text.len() + extra));
        let result = checked_load(&root);
        assert_eq!(result.is_ok(), accepted, "extra {extra}: {result:?}");
        total_result(result, accepted);
    }
}

fn total_result(result: Result<rev::config::Sources, rev::config::Diagnostic>, accepted: bool) {
    if accepted {
        let graph = result.unwrap();
        assert_eq!(graph.total_bytes, 2 * 1024 * 1024);
        assert_eq!(graph.sources.len(), 10);
    } else {
        assert!(result.unwrap_err().message.contains("2 MiB total bytes"));
    }
}

// An absent root differs from a present dangling alias or non-regular source.
// Neither broken roots nor explicitly requested directories select defaults.
#[test]
fn broken_sources() {
    let fixture = Fixture::new();
    let root = fixture.0.join("config.toml");
    symlink("missing.toml", &root).unwrap();
    let error = checked_load(&root).unwrap_err();
    assert_eq!(error.source, root);
    assert!(error.message.contains("read configuration"));
    fs::remove_file(&root).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(
        checked_load(&root)
            .unwrap_err()
            .message
            .contains("regular configuration file")
    );
    fs::remove_dir(&root).unwrap();
    let root = fixture.write("config.toml", "include = ['directory']");
    fs::create_dir(fixture.0.join("directory")).unwrap();
    let error = checked_load(&root).unwrap_err();
    assert_eq!(error.source, root);
    assert_eq!(error.key, ["include"]);
}

// Exercise real OS permission failures for the optional root, an explicit
// include and discovery through an unreadable parent. No permission case skips.
#[test]
fn denied_sources() {
    for name in ["root", "include", "parent"] {
        denied_source(name);
    }
}

fn denied_source(name: &str) {
    let fixture = Fixture::new();
    let child = fixture.write("private/child.toml", "editor.tab_width = 8");
    let root = fixture.write(
        "config.toml",
        "editor.indent_width = 2\ninclude = ['private/child.toml']",
    );
    let (selected, denied) = match name {
        "root" => (root.clone(), root.clone()),
        "include" => (root, child),
        "parent" => (child, fixture.0.join("private")),
        _ => unreachable!("known permission fixture"),
    };
    let original = fs::metadata(&denied).unwrap().permissions();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o0)).unwrap();
    let result = load(&selected);
    let startup = prepare(&selected);
    fs::set_permissions(&denied, original).unwrap();
    let error = result.unwrap_err();
    assert_eq!(startup.snapshot, Snapshot::default(), "{name}");
    assert_eq!(startup.diagnostic.unwrap().to_string(), error.to_string());
    assert!(
        error.message.contains("Permission denied"),
        "{name}: {error}"
    );
}

// Absolute paths are supported; tilde, variable and glob-looking literals are
// ordinary local names. No unrelated directory entry is discovered or repaired.
#[test]
fn literal_paths() {
    let fixture = Fixture::new();
    let absolute = fixture.write("outside.toml", "editor.tab_width = 8");
    let text = format!(
        "include = ['$HOME.toml', '~/literal.toml', '*.toml', {:?}]",
        absolute.to_str().unwrap()
    );
    let root = fixture.write("settings/config.toml", &text);
    fixture.write("settings/$HOME.toml", "");
    fixture.write("settings/~/literal.toml", "");
    fixture.write("settings/*.toml", "");
    fixture.write("settings/unused.toml", "not valid TOML");
    let graph = checked_load(&root).unwrap();
    assert_eq!(graph.sources.len(), 5);
    assert_eq!(graph.sources[4].path, absolute);
    assert_eq!(fs::read_to_string(root).unwrap(), text);
}

// Empty, NUL and remote path literals reject the graph. A malformed later
// source also discards earlier valid layers rather than returning a partial set.
#[test]
fn rejected_graphs() {
    let cases = [
        ("empty", "include = ['']", "nonempty local"),
        ("NUL", r#"include = ["\u0000"]"#, "nonempty local"),
        (
            "remote",
            "include = ['https://example.invalid/settings.toml']",
            "nonempty local",
        ),
    ];
    let fixture = Fixture::new();
    for (name, text, message) in cases {
        let root = fixture.write("config.toml", text);
        let error = checked_load(&root).unwrap_err();
        assert!(error.message.contains(message), "{name}: {error}");
        assert_eq!(error.key, ["include"], "{name}");
    }
    let root = fixture.write("config.toml", "include = ['valid.toml', 'invalid.toml']");
    fixture.write("valid.toml", "editor.tab_width = 8");
    let invalid = fixture.write("invalid.toml", "editor.tab_width = 0");
    let error = checked_load(&root).unwrap_err();
    assert_eq!(error.source, invalid);
    assert_eq!(error.key, ["editor", "tab_width"]);
    assert!(error.position.is_some());
}

// CFG07-CFG08/CFG14: loader rejection must reach startup as one rejected
// candidate, never accepted settings/provenance from the earlier graph prefix.
fn checked_load(root: &std::path::Path) -> Result<Sources, Diagnostic> {
    let result = load(root);
    if let Err(error) = &result {
        let startup = prepare(root);
        assert_eq!(startup.snapshot, Snapshot::default());
        assert_eq!(startup.diagnostic.unwrap().to_string(), error.to_string());
        assert_eq!(startup.snapshot.origin(&["editor", "tab_width"]), None);
    }
    result
}
