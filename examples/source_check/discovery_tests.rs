use super::*;
use crate::{fixture::Fixture, inspect};

// Real Cargo metadata must include both members, every actual target root and
// SDK child sources while foreign packages and build output stay excluded.
#[test]
fn workspace_owners() {
    let fixture = Fixture::new();
    fixture.workspace();
    let workspace = Workspace::at(&fixture.0).unwrap();
    assert_eq!(workspace.packages.len(), 2);
    let actual: BTreeMap<_, _> = workspace
        .sources
        .iter()
        .map(|(path, root)| (workspace.display(path).to_str().unwrap(), *root))
        .collect();
    let expected = BTreeMap::from([
        ("src/lib.rs", true),
        ("src/child.rs", false),
        ("tools/check.rs", true),
        ("tests/editor.rs", true),
        ("benches/work.rs", true),
        ("build.rs", true),
        ("sdk/src/lib.rs", true),
        ("sdk/src/child.rs", false),
        ("sdk/tests/client.rs", true),
    ]);
    assert_eq!(actual, expected);
    let sdk = workspace.select(&["sdk".into()]).unwrap();
    let mut issues = Vec::new();
    for source in sdk {
        issues.extend(
            inspect(
                &fs::read_to_string(source.path).unwrap(),
                source.root,
                false,
            )
            .unwrap(),
        );
    }
    assert!(issues.iter().any(|issue| issue.contains("crate root")));
    assert!(issues.iter().any(|issue| issue.contains("raw pointers")));
}

// Every requested selector must resolve to owned Rust, even when another
// selector is valid; an empty intersection or escaped path cannot pass.
#[test]
fn scope_admission() {
    let fixture = Fixture::new();
    fixture.workspace();
    let workspace = Workspace::at(&fixture.0).unwrap();
    for (name, scopes, count) in [
        ("full", vec![], Some(9)),
        ("member", vec!["sdk"], Some(3)),
        ("one file", vec!["src/lib.rs"], Some(1)),
        ("overlap", vec!["sdk", "sdk/src"], Some(3)),
        ("missing", vec!["missing"], None),
        ("empty", vec![""], None),
        ("manifest", vec!["Cargo.toml"], None),
        ("foreign", vec!["vendor/foreign"], None),
        ("generated", vec!["target"], None),
        ("escaped", vec![".."], None),
        ("partly valid", vec!["sdk", "vendor/foreign"], None),
    ] {
        let paths: Vec<_> = scopes.into_iter().map(PathBuf::from).collect();
        assert_eq!(
            workspace.select(&paths).ok().map(|sources| sources.len()),
            count,
            "{name}"
        );
    }
}

// A private lint target must not admit stale default-target generated Rust or
// a rendered copy of documented source as another first-party implementation.
#[test]
fn alternate_target() {
    let fixture = Fixture::new();
    fixture.workspace();
    fixture.write("target/userguide/example.rs", "unsafe fn generated() {}\n");
    let metadata = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
        .current_dir(&fixture.0)
        .env("CARGO_TARGET_DIR", fixture.0.join("lint-output"))
        .output()
        .unwrap();
    assert!(metadata.status.success());
    let workspace =
        Workspace::from_metadata(serde_json::from_slice(&metadata.stdout).unwrap()).unwrap();
    assert_eq!(workspace.sources.len(), 9);
    assert!(
        workspace
            .sources
            .keys()
            .all(|path| !path.starts_with(fixture.0.join("target")))
    );
}
