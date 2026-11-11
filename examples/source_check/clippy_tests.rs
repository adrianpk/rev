use super::*;
use crate::fixture::Fixture;
use serde_json::json;
use std::process::Command;

fn workspace() -> Workspace {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap();
    Workspace {
        sources: BTreeMap::from([
            (root.join("examples/source-check.rs"), true),
            (root.join("src/lib.rs"), true),
        ]),
        packages: ["member".to_owned()].into_iter().collect(),
        root,
    }
}

fn message(code: &str, path: &str, level: &str, package: &str) -> serde_json::Value {
    json!({"reason": "compiler-message", "package_id": package, "message": {
        "level": level, "message": "fixture diagnostic", "code": {"code": code},
        "spans": [{"file_name": path, "line_start": 1, "is_primary": true}]
    }})
}

// Scoped adoption may defer only the selected warning profile. Baseline,
// compiler, unmapped-location and selected-owner warnings still block.
#[test]
fn diagnostic_admission() {
    let workspace = workspace();
    let sources = workspace
        .select(&["examples/source-check.rs".into()])
        .unwrap();
    for (name, diagnostic, blocking, deferred) in [
        (
            "selected owner",
            message(
                "clippy::redundant_clone",
                "examples/source-check.rs",
                "warning",
                "member",
            ),
            1,
            0,
        ),
        (
            "other owner",
            message("clippy::redundant_clone", "src/lib.rs", "warning", "member"),
            0,
            1,
        ),
        (
            "baseline",
            message("clippy::eq_op", "src/lib.rs", "warning", "member"),
            1,
            0,
        ),
        (
            "compiler warning",
            message("unused_imports", "src/lib.rs", "warning", "member"),
            1,
            0,
        ),
        (
            "selected error",
            message("clippy::redundant_clone", "src/lib.rs", "error", "member"),
            1,
            0,
        ),
        (
            "dependency",
            message("unused_parens", "src/lib.rs", "warning", "foreign"),
            0,
            0,
        ),
        (
            "unknown location",
            message("clippy::redundant_clone", "missing.rs", "warning", "member"),
            1,
            0,
        ),
        (
            "note",
            message("unused_imports", "src/lib.rs", "note", "member"),
            0,
            0,
        ),
    ] {
        let input = format!("{diagnostic}\n{{\"reason\":\"build-finished\",\"success\":true}}\n");
        let admission = read(&mut input.as_bytes(), &workspace, &sources).unwrap();
        assert_eq!(admission.blocking, blocking, "{name}");
        assert_eq!(
            admission.deferred.values().sum::<usize>(),
            deferred,
            "{name}"
        );
    }
}

// Exit success alone is insufficient; missing/malformed/contradictory Cargo
// output cannot be promoted into a clean compiler result.
#[test]
fn compiler_results() {
    let workspace = workspace();
    let sources = workspace.select(&[]).unwrap();
    for (name, input, status, valid) in [
        (
            "success",
            "{\"reason\":\"build-finished\",\"success\":true}\n",
            0,
            true,
        ),
        (
            "failed exit",
            "{\"reason\":\"build-finished\",\"success\":true}\n",
            101,
            false,
        ),
        (
            "failed build",
            "{\"reason\":\"build-finished\",\"success\":false}\n",
            0,
            false,
        ),
        ("no result", "", 0, false),
        ("malformed", "not JSON\n", 0, false),
        (
            "duplicate",
            "{\"reason\":\"build-finished\",\"success\":true}\n{\"reason\":\"build-finished\",\"success\":true}\n",
            0,
            false,
        ),
    ] {
        let passed = read(&mut input.as_bytes(), &workspace, &sources)
            .is_ok_and(|admission| admission.passed(status));
        assert_eq!(passed, valid, "{name}");
    }
}

fn compile(fixture: &Fixture, scopes: &[PathBuf]) -> Admission {
    let workspace = Workspace::at(&fixture.0).unwrap();
    let sources = workspace.select(scopes).unwrap();
    let output = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .current_dir(&fixture.0)
        .args([
            "clippy",
            "--locked",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--message-format=json",
            "--",
        ])
        .args(["-Wclippy::all", "-Dunfulfilled_lint_expectations"])
        .args(
            profile::SELECTED
                .iter()
                .map(|name| format!("-Wclippy::{name}")),
        )
        .output()
        .unwrap();
    let admission = read(&mut output.stdout.as_slice(), &workspace, &sources).unwrap();
    assert_eq!(admission.finished, Some(output.status.success()));
    admission
}

// A real Cargo fixture proves selected warnings defer outside scope, a narrow
// expectation fulfills, and unrelated baseline/unfulfilled warnings never defer.
#[test]
fn real_clippy() {
    let fixture = Fixture::new();
    fixture.workspace();
    fixture.write(
        "sdk/src/lib.rs",
        "#![forbid(unsafe_code)]\npub mod child;\n",
    );
    fixture.write("sdk/src/child.rs", "pub fn count(value: String) -> usize { value.len() }\n#[expect(clippy::needless_pass_by_value, reason = \"Caller transfers ownership.\")]\npub fn accepted(value: String) -> usize { value.len() }\n");
    let scopes = ["src".into()];
    let admission = compile(&fixture, &scopes);
    assert!(admission.passed(0));
    assert!(
        admission
            .deferred
            .get("clippy::needless_pass_by_value")
            .is_some_and(|count| *count > 0)
    );
    assert!(!compile(&fixture, &["sdk".into()]).passed(0));
    fixture.write("sdk/src/child.rs", "pub fn baseline() -> bool { 1 == 1 }\n");
    assert!(!compile(&fixture, &scopes).passed(0));
    fixture.write("sdk/src/child.rs", "#[expect(clippy::redundant_clone, reason = \"Unfulfilled on purpose.\")]\npub fn clean() {}\n");
    assert!(!compile(&fixture, &scopes).passed(0));
}
