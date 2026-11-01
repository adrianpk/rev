use super::{
    Fixture, Status,
    candidates::{self, complete, settle, text, wait},
};
use rev::{
    buffer::Buffers,
    command::{BufferCommand, Command, History, LspCommand, Split},
    session::{CompletionSource, Session},
    view::{Size, View},
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn session(f: &Fixture, mode: &str, last: bool) -> Session {
    let script = f.write("lifetime.sh", include_str!("lifetime.sh"));
    let args = [
        script.to_str().unwrap().to_owned(),
        mode.into(),
        f.0.join("peer").to_str().unwrap().to_owned(),
    ];
    let config = format!(
        "[completion]\nautomatic = false\n[lsp]\nenabled = true\ninitialize_timeout_ms = 500\nrequest_timeout_ms = 250\nshutdown_timeout_ms = 200\nshutdown_on_last_buffer = {last}\n[lsp.servers.rust]\ncommand = '/bin/sh'\nargs = {args:?}\n[lsp.servers.rust.settings]\nfixture.value = 42\n"
    );
    let settings = rev::config::prepare(&f.write("config.toml", &config));
    assert!(settings.diagnostic.is_none(), "{:?}", settings.diagnostic);
    let buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}
fn synchronized(s: &mut Session) -> rev::lsp::DocumentVersion {
    wait(|| {
        s.expire_input(Instant::now());
        s.lsp_document_version().is_some()
    });
    s.lsp_document_version().unwrap()
}
fn pids(f: &Fixture) -> Vec<u32> {
    std::fs::read_to_string(f.0.join("peer.pids"))
        .unwrap_or_default()
        .lines()
        .map(|pid| pid.parse().unwrap())
        .collect()
}
fn reaped(f: &Fixture) {
    for pid in pids(f) {
        assert!(
            !std::path::Path::new(&format!("/proc/{pid}")).exists(),
            "owned child {pid} survived"
        );
    }
}
fn gate(f: &Fixture) {
    assert!(
        std::process::Command::new("mkfifo")
            .arg(f.0.join("peer.gate"))
            .status()
            .unwrap()
            .success()
    );
}

// Restart replaces all document generations for one root, retaining other roots,
// exclusions and the latest unsaved canonical text across aliases and split views.
#[test]
fn scoped_restart() {
    let f = Fixture::new();
    let one = f.write("one/main.rs", "fn one() {}\n");
    let two = f.write("one/other.rs", "fn other() {}\n");
    let other = f.write("two/main.rs", "fn independent() {}\n");
    let alias = f.0.join("one/alias.rs");
    std::os::unix::fs::symlink(&one, &alias).unwrap();
    let mut s = session(&f, "ready", false);
    s.load(&one).unwrap();
    let old = synchronized(&mut s);
    s.execute(Command::Split(Split::Right));
    s.load(&two).unwrap();
    assert_eq!(synchronized(&mut s).connection, old.connection);
    s.execute(Command::Lsp(LspCommand::Disconnect));
    s.load(&other).unwrap();
    let independent = synchronized(&mut s);
    s.load(&alias).unwrap();
    s.replace(3..6, "unsaved").unwrap();
    synchronized(&mut s);
    s.execute(Command::Lsp(LspCommand::Restart));
    let new = synchronized(&mut s);
    assert_ne!(new.connection, old.connection);
    assert_ne!(new.generation, old.generation);
    assert_eq!(text(&s), "fn unsaved() {}\n");
    s.load(&two).unwrap();
    assert_eq!(s.lsp_status(), Status::Excluded);
    s.execute(Command::Lsp(LspCommand::Restart));
    assert_eq!(s.lsp_status(), Status::Excluded);
    s.load(&other).unwrap();
    assert_eq!(synchronized(&mut s), independent);
    assert_eq!(pids(&f).len(), 3);
    s.execute(Command::Lsp(LspCommand::Mode));
    s.execute(Command::Lsp(LspCommand::Restart));
    assert_eq!(s.lsp_status(), Status::Disabled);
    drop(s);
    reaped(&f);
}

// Even with last-buffer shutdown enabled, failure stays suppressed across close,
// reopen and completion requests until explicit restart. Native editing survives.
#[test]
fn failed_restart() {
    let f = Fixture::new();
    let file = f.write("main.rs", "fn cfg_native() {}\nfn main() { cfg }\n");
    let mut s = session(&f, "fail-once", true);
    s.load(&file).unwrap();
    wait(|| matches!(s.lsp_status(), Status::Failed(_)));
    for _ in 0..3 {
        s.execute(Command::Buffer(BufferCommand::Kill));
        candidates::key(&mut s, crossterm::event::KeyCode::Enter);
        s.load(&file).unwrap();
        complete(&mut s);
        assert!(matches!(
            s.completion_source(),
            Some(CompletionSource::Native(_))
        ));
        assert!(matches!(s.lsp_status(), Status::Failed(_)));
    }
    assert_eq!(pids(&f).len(), 1);
    s.execute(Command::Lsp(LspCommand::Restart));
    synchronized(&mut s);
    assert_eq!(pids(&f).len(), 2);
    drop(s);
    reaped(&f);
}

// Closing the last buffer by default retains a healthy idle child with no timer;
// reopen uses that child but a fresh document generation, and exit reaps it.
#[test]
fn idle_retention() {
    let f = Fixture::new();
    let file = f.write("main.rs", "fn main() {}\n");
    let mut s = session(&f, "ready", false);
    s.load(&file).unwrap();
    let old = synchronized(&mut s);
    s.execute(Command::Buffer(BufferCommand::Kill));
    candidates::key(&mut s, crossterm::event::KeyCode::Enter);
    settle(&mut s);
    assert!(s.remaining_input(Instant::now()).is_none());
    assert_eq!(pids(&f).len(), 1);
    assert!(std::path::Path::new(&format!("/proc/{}", pids(&f)[0])).exists());
    s.load(&file).unwrap();
    let new = synchronized(&mut s);
    assert_eq!(old.connection, new.connection);
    assert_ne!(old.generation, new.generation);
    drop(s);
    reaped(&f);
}

// Nonresponsive initialization, blocked writes and ignored shutdown retain finite
// cleanup deadlines. Queue pressure cannot reject or lose canonical native edits.
#[test]
fn bounded_cleanup() {
    for mode in ["initialize-stall", "no-read", "shutdown-stall"] {
        let f = Fixture::new();
        gate(&f);
        let file = f.write("main.rs", "fn main() {}\n");
        let mut s = session(&f, mode, false);
        s.load(&file).unwrap();
        if mode == "initialize-stall" {
            wait(|| f.0.join("peer.waiting").exists());
        } else {
            synchronized(&mut s);
        }
        if mode == "no-read" {
            wait(|| f.0.join("peer.waiting").exists());
            let large = "x".repeat(65536);
            for _ in 0..80 {
                let len = s.buffer().unwrap().len_chars();
                s.replace(0..len, &large).unwrap();
            }
            assert_eq!(text(&s), large);
            wait(|| matches!(s.lsp_status(), Status::Failed(_)));
            assert!(s.lsp_document_version().is_none());
            s.replace(0..1, "y").unwrap();
            assert!(text(&s).starts_with('y'));
        }
        let start = Instant::now();
        s.execute(Command::Lsp(LspCommand::Mode));
        assert!(start.elapsed() < Duration::from_millis(100));
        assert_eq!(s.lsp_status(), Status::Disabled);
        drop(s);
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "{mode}: cleanup exceeded bound"
        );
        reaped(&f);
    }
}

// Advertised configuration/folder/progress requests receive bounded responses;
// unsolicited edits, commands and notifications never gain publication authority.
#[test]
fn server_requests() {
    let f = Fixture::new();
    let file = f.write("main.rs", "fn unchanged() {}\n");
    let requests = [
        json!({"id":101,"method":"workspace/configuration","params":{"items":[{"section":"fixture.value"},{"section":"absent"}]}}),
        json!({"id":102,"method":"workspace/workspaceFolders","params":{}}),
        json!({"id":103,"method":"window/workDoneProgress/create","params":{"token":"work"}}),
        json!({"id":104,"method":"workspace/applyEdit","params":{"edit":{"changes":{file.to_str().unwrap():[{"newText":"unwanted"}]}}}}),
        json!({"id":105,"method":"workspace/executeCommand","params":{"command":"unwanted"}}),
        json!({"id":106,"method":"client/registerCapability","params":{"registrations":[]}}),
        json!({"id":107,"method":"window/showMessageRequest","params":{"message":"notice"}}),
        json!({"id":108,"method":"workspace/configuration","params":{"items":vec![json!({});65]}}),
        json!({"method":"$/progress","params":{"token":"work","value":{"kind":"end"}}}),
        json!({"method":"textDocument/publishDiagnostics","params":{"uri":file,"diagnostics":[]}}),
    ];
    f.write(
        "peer.requests",
        &(requests
            .iter()
            .map(|r| {
                let mut r = r.clone();
                r["jsonrpc"] = json!("2.0");
                r.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"),
    );
    let mut s = session(&f, "requests", false);
    s.load(&file).unwrap();
    let before = synchronized(&mut s);
    wait(|| candidates::frames(&f).iter().any(|v| v["id"] == 108));
    let frames = candidates::frames(&f);
    let response = |id| frames.iter().find(|v| v["id"] == id).unwrap();
    assert_eq!(response(101)["result"], json!([42, null]));
    assert_eq!(
        response(102)["result"][0]["uri"],
        format!("file://{}", f.0.display())
    );
    assert_eq!(response(103)["result"], Value::Null);
    assert_eq!(response(104)["result"]["applied"], false);
    for id in [105, 106] {
        assert_eq!(response(id)["error"]["code"], -32601);
    }
    assert_eq!(response(107)["result"], Value::Null);
    assert_eq!(response(108)["error"]["code"], -32602);
    assert_eq!(s.lsp_document_version(), Some(before));
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "fn unchanged() {}\n");
    assert_eq!(std::fs::read_to_string(file).unwrap(), text(&s));
    drop(s);
    reaped(&f);
}
