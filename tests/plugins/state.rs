use super::{
    buffers::Rpc,
    fixture::Fixture,
    lifecycle::{Editor, peer},
};
use crossterm::event::Event;
use rev::{command::PluginCommand, plugins::lifecycle::State};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Child, Command},
};

fn rpc(root: &Path) -> Rpc {
    let mut e = Editor::state(root, 1);
    e.wait(|e| e.status().state == State::Active);
    Rpc::from_editor(e)
}
fn path(root: &Path, package: &str) -> PathBuf {
    root.join("state/plugins/packages")
        .join(package)
        .join("state.toml")
}
fn get(rpc: &mut Rpc, concern: &str, record: &str) -> Value {
    rpc.ok("state.get", json!({"concern":concern,"record":record}))
}
fn put(rpc: &mut Rpc, concern: &str, record: &str, expected: Value, value: Value) -> Value {
    rpc.ok(
        "state.put",
        json!({"concern":concern,"record":record,"expected_version":expected,"value":value}),
    )
}
fn list(rpc: &mut Rpc, concern: &str, cursor: Value, limit: usize) -> Value {
    rpc.ok(
        "state.list",
        json!({"concern":concern,"page":{"cursor":cursor,"limit":limit}}),
    )
}
fn seed(rpc: &mut Rpc) -> Value {
    put(rpc, "notes", "alpha", Value::Null, json!("first"))
}
fn reply(rpc: &mut Rpc, sequence: usize) -> Value {
    rpc.editor
        .wait(|e| e.path(&format!("response-{sequence}")).exists());
    serde_json::from_slice(&fs::read(rpc.editor.path(&format!("response-{sequence}"))).unwrap())
        .unwrap()
}

// State reads create no directories; typed JSON, including present null, stays
// inert in schema-1 TOML. Restart changes process authority, not durable values.
#[test]
fn persistence() {
    let root = Fixture::new();
    let mut rpc = rpc(&root.0);
    assert_eq!(get(&mut rpc, "notes", "absent")["status"], "absent");
    assert_eq!(
        list(&mut rpc, "notes", Value::Null, 256)["records"],
        json!([])
    );
    assert!(!root.0.join("state").exists());
    let values = [
        json!(null),
        json!(true),
        json!(42),
        json!(-7.25),
        json!("Unicode λ 🦀\n\"quoted\""),
        json!([1,null,{"nested":"literal"}]),
        json!({"command":"inert","data":{"number":9}}),
    ];
    let mut expected = Value::Null;
    for value in values {
        expected = put(&mut rpc, "notes", "alpha", expected, value.clone())["version"].clone();
        let result = get(&mut rpc, "notes", "alpha");
        assert_eq!(result["status"], "present");
        assert_eq!(result["value"], value);
        assert_eq!(result["version"], expected);
    }
    let bytes = fs::read(path(&root.0, "field.notes")).unwrap();
    let envelope: toml::Value = toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap();
    assert_eq!(envelope["schema_version"].as_integer(), Some(1));
    assert_eq!(envelope["records"].as_array().unwrap().len(), 1);
    assert!(!root.0.join("config").exists());
    let previous = rpc.editor.status().generation;
    for entry in fs::read_dir(rpc.editor.fixture.0.join("field.notes")).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("request-") || name.starts_with("response-") {
            fs::remove_file(entry.path()).unwrap();
        }
    }
    rpc.editor.manage(PluginCommand::Restart);
    rpc.editor
        .wait(|e| e.status().state == State::Active && e.status().generation != previous);
    let mut rpc = Rpc::from_editor(rpc.editor);
    assert_eq!(get(&mut rpc, "notes", "alpha")["version"], expected);
    assert_eq!(fs::read(path(&root.0, "field.notes")).unwrap(), bytes);
}

// Two independent native hosts re-read under one package lock. Exactly one
// simultaneous same-version writer wins; independent records rebase safely.
#[test]
fn concurrent_instances() {
    let root = Fixture::new();
    let mut a = rpc(&root.0);
    let mut b = rpc(&root.0);
    let first = seed(&mut a);
    assert_eq!(get(&mut b, "notes", "alpha")["version"], first["version"]);
    let params = |label| json!({"concern":"notes","record":"alpha","expected_version":first["version"],"value":label});
    let qa = a.enqueue("state.put", params("writer-a"));
    let qb = b.enqueue("state.put", params("writer-b"));
    let results = [reply(&mut a, qa), reply(&mut b, qb)];
    assert_eq!(
        results.iter().filter(|v| v.get("result").is_some()).count(),
        1,
        "{results:?}"
    );
    let loser = results.iter().find(|v| v.get("error").is_some()).unwrap();
    assert!([json!("busy"), json!("state_conflict")].contains(&loser["error"]["data"]["kind"]));
    let current = get(&mut b, "notes", "alpha");
    b.error("state.put", params("stale"), "state_conflict");
    let beta = put(&mut b, "notes", "beta", Value::Null, json!("independent"));
    let gamma = put(&mut a, "other", "gamma", Value::Null, json!(true));
    let updated = put(
        &mut a,
        "notes",
        "alpha",
        current["version"].clone(),
        json!("updated-a"),
    );
    put(
        &mut b,
        "notes",
        "beta",
        beta["version"].clone(),
        json!("updated-b"),
    );
    assert_eq!(get(&mut b, "notes", "alpha")["version"], updated["version"]);
    assert_eq!(get(&mut a, "notes", "beta")["value"], "updated-b");
    assert_eq!(get(&mut b, "other", "gamma")["version"], gamma["version"]);
}

// Delete/recreate retains the aggregate high-water mark; stale versions cannot
// match a recreated record. Cursor continuations bind concern and revision.
#[test]
fn deletion_and_pages() {
    let root = Fixture::new();
    let mut rpc = rpc(&root.0);
    let first = seed(&mut rpc);
    rpc.error(
        "state.delete",
        json!({"concern":"notes","record":"alpha","expected_version":"99"}),
        "state_conflict",
    );
    rpc.ok(
        "state.delete",
        json!({"concern":"notes","record":"alpha","expected_version":first["version"]}),
    );
    assert_eq!(get(&mut rpc, "notes", "alpha")["status"], "absent");
    let second = put(&mut rpc, "notes", "alpha", Value::Null, json!(null));
    assert!(
        second["version"].as_str().unwrap().parse::<u64>().unwrap()
            > first["version"].as_str().unwrap().parse::<u64>().unwrap()
    );
    rpc.error("state.put",json!({"concern":"notes","record":"alpha","expected_version":first["version"],"value":"ABA"}),"state_conflict");
    put(&mut rpc, "notes", "charlie", Value::Null, json!(3));
    put(&mut rpc, "notes", "bravo", Value::Null, json!(2));
    let page = list(&mut rpc, "notes", Value::Null, 1);
    assert_eq!(page["records"][0]["record"], "alpha");
    let next = list(&mut rpc, "notes", page["next"].clone(), 1);
    assert_eq!(next["records"][0]["record"], "bravo");
    assert_eq!(
        list(&mut rpc, "notes", next["next"].clone(), 1)["records"][0]["record"],
        "charlie"
    );
    rpc.error(
        "state.list",
        json!({"concern":"other","page":{"cursor":page["next"],"limit":1}}),
        "invalid_arguments",
    );
    put(&mut rpc, "other", "delta", Value::Null, json!(4));
    rpc.error(
        "state.list",
        json!({"concern":"notes","page":{"cursor":page["next"],"limit":1}}),
        "state_conflict",
    );
}

struct Held {
    child: Child,
    directory: PathBuf,
}
impl Drop for Held {
    fn drop(&mut self) {
        fs::write(self.directory.join("release"), "release").unwrap();
        assert!(self.child.wait().unwrap().success());
    }
}
// A separately owned Rust process holds the actual lock. RPC returns busy and
// ordinary native input continues; releasing the fixture restores publication.
#[test]
fn contention() {
    let root = Fixture::new();
    let mut rpc = rpc(&root.0);
    let first = seed(&mut rpc);
    let directory = root.0.join("holder");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("mode"), "state-lock").unwrap();
    fs::write(
        directory.join("lock-path"),
        path(&root.0, "field.notes")
            .with_file_name("state.toml.lock")
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let held = Held {
        child: Command::new(peer())
            .arg(&directory)
            .arg("field.notes")
            .spawn()
            .unwrap(),
        directory,
    };
    rpc.editor.wait(|_| held.directory.join("locked").exists());
    let params = json!({"concern":"notes","record":"alpha","expected_version":first["version"],"value":"blocked"});
    rpc.error("state.put", params.clone(), "busy");
    rpc.error(
        "state.get",
        json!({"concern":"notes","record":"alpha"}),
        "busy",
    );
    rpc.editor.session.handle(Event::Paste("native".into()));
    assert!(rpc.editor.session.buffer().unwrap().len_chars() > "source 🦀\n".chars().count());
    drop(held);
    rpc.ok("state.put", params);
    assert_eq!(get(&mut rpc, "notes", "alpha")["value"], "blocked");
}

// Lexical IDs and strict objects cannot select a foreign namespace or path.
// Packages sharing one host still see separate stores; native neighbors survive.
#[test]
fn namespaces() {
    let root = Fixture::new();
    let mut e = Editor::state(&root.0, 2);
    e.wait(|e| {
        ["field.notes", "other.notes"]
            .iter()
            .all(|id| e.session.plugin_status(id).unwrap().state == State::Active)
    });
    let mut rpc = Rpc::from_editor(e);
    seed(&mut rpc);
    let directory = rpc.editor.fixture.0.join("other.notes");
    fs::write(directory.join("request-0"),format!("{}\n",json!({"jsonrpc":"2.0","id":"buffer-request-0","method":"state.get","params":{"concern":"notes","record":"alpha"}}))).unwrap();
    rpc.editor.wait(|_| directory.join("response-0").exists());
    let response: Value =
        serde_json::from_slice(&fs::read(directory.join("response-0")).unwrap()).unwrap();
    assert_eq!(response["result"]["status"], "absent");
    for (field, value) in [
        ("concern", json!("../other.notes")),
        ("record", json!("../../config")),
        ("package", json!("other.notes")),
    ] {
        let mut params = json!({"concern":"notes","record":"alpha"});
        params[field] = value;
        rpc.error("state.get", params, "invalid_arguments");
    }
    rpc.error(
        "state.put",
        json!({"concern":"notes","record":"omitted","value":null}),
        "invalid_arguments",
    );
    assert!(!root.0.join("config").exists());
    assert!(!path(&root.0, "other.notes").exists());
    fs::create_dir_all(root.0.join("state/macros")).unwrap();
    let neighbor = root.0.join("state/macros/library.toml");
    fs::write(&neighbor, "native neighbor").unwrap();
    put(&mut rpc, "notes", "beta", Value::Null, json!(true));
    assert_eq!(fs::read_to_string(neighbor).unwrap(), "native neighbor");
}

// Corrupt, oversized and substituted stores are refused without changing any
// existing bytes or following symlinks into another package/configuration path.
#[test]
fn invalid_stores() {
    for variant in [
        "schema",
        "json",
        "count",
        "record",
        "canonical-record",
        "store",
        "file-link",
        "lock-link",
        "parent-link",
    ] {
        let root = Fixture::new();
        let mut rpc = rpc(&root.0);
        seed(&mut rpc);
        let file = path(&root.0, "field.notes");
        let valid = fs::read(&file).unwrap();
        let victim = root.0.join("protected");
        fs::write(&victim, &valid).unwrap();
        fs::set_permissions(&victim, fs::Permissions::from_mode(0o600)).unwrap();
        let kind = match variant {
            "schema" => {
                fs::write(&file, "schema_version=99\nrevision=0\nrecords=[]\n").unwrap();
                "invalid_arguments"
            }
            "json" => {
                fs::write(&file,"schema_version=1\nrevision=1\n[[records]]\nconcern='notes'\nrecord='alpha'\nversion=1\nvalue_json='not JSON'\n").unwrap();
                "invalid_arguments"
            }
            "count" => {
                let mut text = "schema_version=1\nrevision=257\n".to_owned();
                for i in 0..257 {
                    text.push_str(&format!("[[records]]\nconcern='notes'\nrecord='r{i:03}'\nversion={}\nvalue_json='null'\n",i+1));
                }
                fs::write(&file, text).unwrap();
                "invalid_arguments"
            }
            "record" => {
                fs::write(&file,format!("schema_version=1\nrevision=1\n[[records]]\nconcern='notes'\nrecord='alpha'\nversion=1\nvalue_json='\"{}\"'\n","x".repeat(256*1024))).unwrap();
                "limit_exceeded"
            }
            "canonical-record" => {
                let json = format!("[{}]", vec!["1e1"; 60_000].join(","));
                fs::write(&file,format!("schema_version=1\nrevision=1\n[[records]]\nconcern='notes'\nrecord='alpha'\nversion=1\nvalue_json='{json}'\n")).unwrap();
                "limit_exceeded"
            }
            "store" => {
                File::options()
                    .write(true)
                    .open(&file)
                    .unwrap()
                    .set_len(4 * 1024 * 1024 + 1)
                    .unwrap();
                "limit_exceeded"
            }
            "file-link" => {
                fs::remove_file(&file).unwrap();
                std::os::unix::fs::symlink(&victim, &file).unwrap();
                "unsupported_resource"
            }
            "lock-link" => {
                let lock = file.with_file_name("state.toml.lock");
                fs::remove_file(&lock).unwrap();
                std::os::unix::fs::symlink(&victim, lock).unwrap();
                "unsupported_resource"
            }
            _ => {
                let parent = file.parent().unwrap();
                fs::rename(parent, root.0.join("retained")).unwrap();
                let foreign = root.0.join("foreign");
                fs::DirBuilder::new().mode(0o700).create(&foreign).unwrap();
                fs::write(foreign.join("state.toml"), &valid).unwrap();
                std::os::unix::fs::symlink(foreign, parent).unwrap();
                "unsupported_resource"
            }
        };
        let before = fs::read(&file).unwrap();
        rpc.error(
            "state.get",
            json!({"concern":"notes","record":"alpha"}),
            kind,
        );
        rpc.error(
            "state.put",
            json!({"concern":"notes","record":"beta","expected_version":null,"value":true}),
            kind,
        );
        assert_eq!(fs::read(&file).unwrap(), before, "{variant}");
        assert_eq!(fs::read(&victim).unwrap(), valid, "{variant}");
        assert!(!root.0.join("config").exists());
    }
}

// Caller-side payload limits and the aggregate encoded ceiling reject a whole
// publication while preserving the last valid store and unrelated records.
#[test]
fn bounds() {
    let root = Fixture::new();
    let mut rpc = rpc(&root.0);
    seed(&mut rpc);
    let before = fs::read(path(&root.0, "field.notes")).unwrap();
    rpc.error("state.put",json!({"concern":"notes","record":"oversized","expected_version":null,"value":"x".repeat(256*1024)}),"limit_exceeded");
    assert_eq!(fs::read(path(&root.0, "field.notes")).unwrap(), before);
    let data = json!("x".repeat(240 * 1024));
    for i in 0..17 {
        put(
            &mut rpc,
            "large",
            &format!("r{i:03}"),
            Value::Null,
            data.clone(),
        );
    }
    let before = fs::read(path(&root.0, "field.notes")).unwrap();
    rpc.error(
        "state.put",
        json!({"concern":"large","record":"overflow","expected_version":null,"value":data}),
        "limit_exceeded",
    );
    assert_eq!(fs::read(path(&root.0, "field.notes")).unwrap(), before);
    assert_eq!(get(&mut rpc, "notes", "alpha")["value"], "first");
}
