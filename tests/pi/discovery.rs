use super::*;

fn directory(f: &Fixture, mode: &str) -> Operation {
    let command = support::peer(f, mode);
    Operation::directory(
        Pi {
            command: command.to_str().unwrap().into(),
            request_timeout_ms: 1000,
            ..Pi::default()
        },
        f.0.clone(),
        r#"{"instruction":"fixture only","dirty_sources":[]}"#.into(),
        41,
    )
    .unwrap()
}

// Independent handled/notification admission precedes fresh state and ordinary
// model work. Extraction is private, byte-exact and retired after owned exit.
#[test]
fn directory_profile() {
    let f = Fixture::new();
    let mut operation = directory(&f, "directory_tools");
    let result = finish(&mut operation).unwrap();
    assert!(result.directory);
    assert!(result.text.contains("selection explanation"));
    let args: Vec<String> =
        serde_json::from_str(&fs::read_to_string(f.0.join("args")).unwrap()).unwrap();
    assert!(!args.iter().any(|arg| arg == "--no-tools"));
    let tools = args.iter().position(|arg| arg == "--tools").unwrap();
    assert_eq!(args[tools + 1], "rev_read,rev_list,rev_search");
    let resource = args.iter().position(|arg| arg == "--extension").unwrap();
    assert!(!std::path::Path::new(&args[resource + 1]).exists());
    assert_eq!(
        fs::read(f.0.join("resource-copy")).unwrap(),
        include_bytes!("../../src/ai/pi/discovery.js")
    );
    let mode: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(f.0.join("resource-mode")).unwrap()).unwrap();
    assert_eq!(mode["file"], 0o600);
    assert_eq!(mode["parent"], 0o700);
    assert_eq!(mode["link"], false);
    let requests: Vec<serde_json::Value> = fs::read_to_string(f.0.join("requests"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        requests[0]["message"]
            .as_str()
            .unwrap()
            .starts_with("/rev-discovery ")
    );
    assert_eq!(requests[1]["type"], "get_state");
    assert_eq!(requests[2]["type"], "prompt");
}

// A missing admission half, broader profile, unsupported result, uncorrelated
// completion or exhausted invocation cannot become a successful native result.
#[test]
fn directory_rejections() {
    for mode in [
        "admit_extra",
        "admit_missing",
        "admit_rejected",
        "admit_partial",
        "directory_extra",
        "directory_unmatched",
        "directory_content",
        "directory_unfinished",
        "directory_calls",
        "directory_repeated",
        "directory_bytes",
        "directory_fatal",
        "directory_update",
    ] {
        let f = Fixture::new();
        let result = finish(&mut directory(&f, mode));
        assert!(result.is_err(), "{mode}");
        assert!(!result.unwrap_err().contains("SECRET-FIXTURE"), "{mode}");
        if mode.starts_with("admit_") {
            assert!(!f.0.join("prompt").exists());
        }
    }
}

// Replacing the root during version admission cannot retarget the delayed
// operation. The pinned inode remains live until this supervisor finishes.
#[test]
fn root_replaced_during_startup() {
    let f = Fixture::new();
    let command = support::peer(&f, "version_gate");
    let root = f.0.join("project");
    fs::create_dir(&root).unwrap();
    let mut operation = Operation::directory(
        Pi {
            command: command.to_str().unwrap().into(),
            request_timeout_ms: 1000,
            ..Pi::default()
        },
        root.clone(),
        r#"{"dirty_sources":[]}"#.into(),
        43,
    )
    .unwrap();
    support::until(|| f.0.join("version-pid").exists());
    fs::rename(&root, f.0.join("retired")).unwrap();
    fs::create_dir(&root).unwrap();
    f.write("release-version", "");
    assert!(finish(&mut operation).is_err());
    assert!(!f.0.join("prompt").exists());
}
