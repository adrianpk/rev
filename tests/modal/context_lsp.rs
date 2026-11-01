use super::completion::source;
use super::*;
use rev::command::CompletionCommand;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const RUST: &str = "fn cfg_native() {}\nfn main() { cfg| }\n";

fn semantic(mode: &str) -> (Fixture, Fixture, Session) {
    let peer = Fixture::new();
    peer.write("peer.result", &json!([item()]).to_string());
    let mut resolved = item();
    resolved["additionalTextEdits"] = json!([{
        "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},
        "newText":"use demo::Thing;\n"
    }]);
    peer.write("peer.resolved", &resolved.to_string());
    let script = peer.write("candidates.sh", include_str!("../lsp/candidates.sh"));
    let args = [
        script.to_str().unwrap().to_owned(),
        mode.to_owned(),
        peer.0.join("peer").to_str().unwrap().to_owned(),
    ];
    let settings = format!(
        "[completion]\nautomatic = false\ndelay_ms = 1\n[lsp]\nenabled = true\nrequest_timeout_ms = 1000\nshutdown_timeout_ms = 200\n[lsp.servers.rust]\ncommand = '/bin/sh'\nargs = {args:?}\nlanguages = ['rust']"
    );
    // Retain both owned directories until the native client has been dropped.
    let (document, editor) = source(RUST, &settings);
    (peer, document, editor)
}

fn item() -> Value {
    json!({"label":"cfg_server","kind":3,"data":{"index":7}})
}

fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "bounded modal LSP wait expired");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn requested(peer: &Fixture, method: &str) -> bool {
    std::fs::read_to_string(peer.0.join("peer.frames"))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .any(|frame| frame["method"] == method)
}

fn stopped(peer: &Fixture, editor: Session) {
    drop(editor);
    let pid = std::fs::read_to_string(peer.0.join("peer.pid")).unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{}", pid.trim())).exists());
}

// Actual owned LSP transport resolves selected imports in Insert and Replace.
// Both edits publish atomically through native history and preserve entry state.
#[test]
fn modal_resolved_import() {
    for entry in ['i', 'R'] {
        let (peer, _document, mut editor) = semantic("resolve");
        keys(&mut editor, &entry.to_string());
        let state = editor.editing_state();
        let original = text(&editor);
        editor.execute(Command::Completion(CompletionCommand::AtPoint));
        settle(&mut editor);
        assert_eq!(
            editor.completion().unwrap().candidates()[0].text(),
            "cfg_server"
        );
        editor.handle(key(KeyCode::Tab));
        settle(&mut editor);
        assert_eq!(editor.editing_state(), state);
        assert_eq!(
            text(&editor),
            format!(
                "use demo::Thing;\n{}",
                original.replace("{ cfg }", "{ cfg_server }")
            ),
            "{entry}: {}",
            editor.echo()
        );
        assert_eq!(editor.buffer().unwrap().history().len(), 1);
        editor.execute(Command::History(History::Undo));
        assert_eq!(text(&editor), original);
        stopped(&peer, editor);
    }
}

// Delayed list/resolve replies are delivered after cancellation over owned IPC.
// Leaving and reentering Insert cannot reopen or apply the old invocation.
#[test]
fn modal_late_result() {
    for (mode, method) in [
        ("late", "textDocument/completion"),
        ("resolve-late", "completionItem/resolve"),
    ] {
        let (peer, _document, mut editor) = semantic(mode);
        keys(&mut editor, "i");
        editor.execute(Command::Completion(CompletionCommand::AtPoint));
        if mode == "resolve-late" {
            settle(&mut editor);
            assert!(editor.completion().is_some());
            editor.handle(key(KeyCode::Tab));
        }
        wait(|| {
            editor.expire_input(Instant::now());
            requested(&peer, method)
        });
        let original = text(&editor);
        let point = editor.view().point;
        editor.execute(Command::Modal(ModalCommand::Normal));
        keys(&mut editor, "i");
        wait(|| {
            editor.expire_input(Instant::now());
            peer.0.join("peer.late").exists()
        });
        settle(&mut editor);
        assert_eq!(editor.editing_state(), Some(EditingState::Insert));
        assert!(editor.completion().is_none());
        assert_eq!(text(&editor), original);
        assert_eq!(editor.view().point, point);
        assert!(editor.buffer().unwrap().history().is_empty());
        stopped(&peer, editor);
    }
}
