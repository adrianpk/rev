use super::lifecycle::Editor;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    command::{BufferCommand, Command, PluginCommand},
    plugins::lifecycle::State,
};
use serde_json::{Value, json};
use std::fs;

pub(super) struct Rpc {
    pub(super) editor: Editor,
    sequence: usize,
}
impl Rpc {
    pub(super) fn new(profile: &str) -> Self {
        let mut editor = Editor::new(profile, "buffers", "", true);
        editor.wait(|e| e.status().state == State::Active);
        Self::from_editor(editor)
    }
    pub(super) fn from_editor(editor: Editor) -> Self {
        Self {
            editor,
            sequence: 0,
        }
    }
    pub(super) fn enqueue(&mut self, method: &str, params: Value) -> usize {
        let seq = self.sequence;
        self.sequence += 1;
        let path = self.editor.path(&format!("request-{seq}"));
        let temp = self.editor.path("request-tmp");
        fs::write(&temp, format!("{}\n", json!({"jsonrpc":"2.0","id":format!("buffer-request-{seq}"),"method":method,"params":params}))).unwrap();
        fs::rename(temp, path).unwrap();
        seq
    }
    pub(super) fn request(&mut self, method: &str, params: Value) -> Value {
        let seq = self.enqueue(method, params);
        self.editor
            .wait(|e| e.path(&format!("response-{seq}")).exists());
        let value: Value = serde_json::from_slice(
            &fs::read(self.editor.path(&format!("response-{seq}"))).unwrap(),
        )
        .unwrap();
        assert_eq!(value["id"], format!("buffer-request-{seq}"));
        value
    }
    pub(super) fn ok(&mut self, method: &str, params: Value) -> Value {
        let response = self.request(method, params);
        assert!(response.get("error").is_none(), "{response}");
        response["result"].clone()
    }
    pub(super) fn error(&mut self, method: &str, params: Value, kind: &str) {
        let response = self.request(method, params);
        assert_eq!(response["error"]["data"]["kind"], kind, "{response}");
        assert_eq!(response["error"]["data"]["request"], response["id"]);
    }
    pub(super) fn list(&mut self) -> Vec<Value> {
        let mut cursor = Value::Null;
        let mut buffers = Vec::new();
        loop {
            let result = self.ok(
                "buffers.list",
                json!({"page":{"cursor":cursor,"limit":1},"kind":"text"}),
            );
            buffers.extend(result["buffers"].as_array().unwrap().iter().cloned());
            cursor = result["next"].clone();
            if cursor.is_null() {
                return buffers;
            }
            assert!(buffers.len() < 20, "continuation must progress");
        }
    }
    pub(super) fn switch(&mut self, name: &str) {
        self.editor
            .session
            .execute(Command::Buffer(BufferCommand::Switch));
        self.editor.session.handle(Event::Paste(name.into()));
        self.editor.session.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
    }
}
pub(super) fn source(rpc: &mut Rpc) -> Value {
    let name = rpc.editor.session.buffer().unwrap().name().to_owned();
    rpc.list().into_iter().find(|b| b["name"] == name).unwrap()
}
pub(super) fn create(rpc: &mut Rpc, name: &str, text: &str, read_only: bool) -> Value {
    rpc.ok(
        "buffers.create",
        json!({"name":name,"text":text,"read_only":read_only}),
    )
}

// Actual child RPC reads hidden unsaved Unicode without changing native focus.
// Created results are ordinary navigable buffers with persistent provenance.
#[test]
fn hidden_sources() {
    for profile in ["non-modal", "modal"] {
        let mut rpc = Rpc::new(profile);
        let original = rpc.editor.session.buffer().unwrap().id();
        let source = source(&mut rpc);
        let result = create(&mut rpc, "Collected notes", "result e\u{301}🦀\r\n", false);
        assert_eq!(rpc.editor.session.buffer().unwrap().id(), original);
        assert_eq!(result["owner"], "field.notes");
        assert_eq!(result["modified"], true);
        assert!(result["path"].is_null());
        assert_eq!(result["kind"], "text");
        let read = rpc.ok("buffers.read", json!({"buffer":source["stamp"]["buffer"],"range":{"start":7,"end":8},"expected_revision":source["stamp"]["revision"]}));
        assert_eq!(read["text"], "🦀");
        assert_eq!(read["stamp"], source["stamp"]);
        let read = rpc.ok("buffers.read", json!({"buffer":result["stamp"]["buffer"],"range":{"start":7,"end":12},"expected_revision":null}));
        assert_eq!(read["text"], "e\u{301}🦀\r\n");
        assert_eq!(rpc.editor.session.buffer().unwrap().id(), original);
        let created = rpc.list();
        assert!(created.iter().any(|b| b["stamp"] == result["stamp"]));
        rpc.switch("Collected notes");
        assert_eq!(
            rpc.editor
                .session
                .buffer()
                .unwrap()
                .plugin_owner()
                .unwrap()
                .as_str(),
            "field.notes"
        );
        rpc.editor.session.insert('!');
        rpc.editor.manage(PluginCommand::Deactivate);
        rpc.editor.wait(|e| e.status().state == State::Inactive);
        let buffer = rpc.editor.session.buffer().unwrap();
        assert_eq!(buffer.plugin_owner().unwrap().as_str(), "field.notes");
        assert!(buffer.is_modified());
        assert!(!buffer.is_transient());
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap(),
            "!result e\u{301}🦀\r\n"
        );
    }
}

// Reads compare captured revisions and scalar bounds, never current focus or
// byte offsets. Forged, foreign and closed handles cannot name another source.
#[test]
fn handle_and_range_errors() {
    let mut rpc = Rpc::new("non-modal");
    let source = source(&mut rpc);
    rpc.editor.session.handle(Event::Paste("!".into()));
    rpc.error("buffers.read", json!({"buffer":source["stamp"]["buffer"],"range":{"start":0,"end":1},"expected_revision":source["stamp"]["revision"]}), "stale_revision");
    for range in [
        json!({"start":0,"end":999}),
        json!({"start":u64::MAX,"end":u64::MAX}),
    ] {
        rpc.error(
            "buffers.read",
            json!({"buffer":source["stamp"]["buffer"],"range":range,"expected_revision":null}),
            "invalid_range",
        );
    }
    let read = rpc.ok("buffers.read", json!({"buffer":source["stamp"]["buffer"],"range":{"start":0,"end":1},"expected_revision":null}));
    assert_eq!(read["text"], "!");
    assert_ne!(read["stamp"]["revision"], source["stamp"]["revision"]);
    let mut foreign = source["stamp"]["buffer"].clone();
    foreign["session"] = "other-host".into();
    rpc.error(
        "buffers.describe",
        json!({"buffer":foreign}),
        "missing_buffer",
    );
    let mut missing = source["stamp"]["buffer"].clone();
    missing["id"] = "buffer-0".into();
    rpc.error(
        "buffers.describe",
        json!({"buffer":missing}),
        "missing_buffer",
    );
    let empty = create(&mut rpc, "Empty result", "", false);
    rpc.switch("Empty result");
    rpc.editor
        .session
        .execute(Command::Buffer(BufferCommand::Kill));
    rpc.editor.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    rpc.error(
        "buffers.describe",
        json!({"buffer":empty["stamp"]["buffer"]}),
        "missing_buffer",
    );
    rpc.error(
        "buffers.list",
        json!({"page":{"cursor":"foreign-cursor","limit":1},"kind":null}),
        "invalid_arguments",
    );
    rpc.error(
        "buffers.list",
        json!({"page":{"cursor":null,"limit":257},"kind":null}),
        "limit_exceeded",
    );
    rpc.error(
        "buffers.list",
        json!({"page":{"cursor":null,"limit":1},"kind":"terminal"}),
        "unsupported_resource",
    );
    rpc.error(
        "buffers.create",
        json!({"name":"bad\nname","text":"", "read_only":false}),
        "invalid_arguments",
    );
    rpc.error("buffers.read", json!({"buffer":source["stamp"]["buffer"],"range":{"start":0,"end":0},"expected_revision":null,"force":true}), "invalid_arguments");
}

// Canonical aliases reuse native unsaved source identity; new and missing file
// visits remain hidden and do not write disk or select another source.
#[test]
fn canonical_open() {
    let mut rpc = Rpc::new("non-modal");
    let path = rpc.editor.fixture.write("source.txt", "disk content\n");
    rpc.editor.session.load(&path).unwrap();
    rpc.editor.session.handle(Event::Paste("unsaved ".into()));
    let native = source(&mut rpc);
    let alias = rpc.editor.fixture.0.join("alias.txt");
    std::os::unix::fs::symlink(&path, &alias).unwrap();
    let opened = rpc.ok("buffers.open", json!({"path":alias}));
    assert_eq!(opened["stamp"], native["stamp"]);
    let other = rpc.editor.fixture.write("other.txt", "hidden\n");
    let result = rpc.ok("buffers.open", json!({"path":other}));
    assert_ne!(result["stamp"]["buffer"], native["stamp"]["buffer"]);
    assert_eq!(source(&mut rpc)["stamp"], native["stamp"]);
    assert_eq!(fs::read_to_string(path).unwrap(), "disk content\n");
    let missing = rpc.editor.fixture.0.join("not-created.txt");
    let result = rpc.ok("buffers.open", json!({"path":missing}));
    assert_eq!(result["scalar_len"], 0);
    assert_eq!(result["expected_disk"]["kind"], "absent");
    assert!(!missing.exists());
    rpc.error(
        "buffers.open",
        json!({"path":rpc.editor.fixture.0}),
        "unsupported_resource",
    );
    let binary = rpc.editor.fixture.write("binary.txt", "bad\0input");
    rpc.error(
        "buffers.open",
        json!({"path":binary}),
        "unsupported_resource",
    );
}

// Display uses explicit native pane identities and fit checks. Transient input
// is never canceled by an asynchronous display result.
#[test]
fn native_placement() {
    let mut rpc = Rpc::new("non-modal");
    let original = rpc.editor.session.windows().selected_id();
    let pane = json!({"session":source(&mut rpc)["stamp"]["buffer"]["session"],"id":format!("pane-{original}")});
    let result = create(&mut rpc, "Display result", "visible", true);
    assert_eq!(result["read_only"], true);
    for placement in ["split_horizontal", "split_vertical"] {
        let shown = rpc.ok(
            "buffers.show",
            json!({"buffer":result["stamp"]["buffer"],"pane":pane,"placement":placement}),
        );
        assert_ne!(shown["pane"], pane);
        assert_eq!(rpc.editor.session.windows().selected_id(), original);
    }
    assert_eq!(rpc.editor.session.windows().iter().len(), 3);
    rpc.editor
        .session
        .execute(Command::Buffer(BufferCommand::Switch));
    let prompt = rpc.editor.session.echo().to_owned();
    rpc.error(
        "buffers.show",
        json!({"buffer":result["stamp"]["buffer"],"pane":pane,"placement":"current"}),
        "unsupported_invocation",
    );
    assert_eq!(rpc.editor.session.echo(), prompt);
    rpc.editor.session.execute(Command::KeyboardQuit);
    let mut foreign = pane.clone();
    foreign["session"] = "foreign-host".into();
    rpc.error(
        "buffers.show",
        json!({"buffer":result["stamp"]["buffer"],"pane":foreign,"placement":"current"}),
        "unsupported_invocation",
    );
    let closed = rpc
        .editor
        .session
        .windows()
        .iter()
        .map(|(id, _)| id)
        .find(|id| *id != original)
        .unwrap();
    rpc.editor.session.select_view(closed).unwrap();
    rpc.editor
        .session
        .execute(Command::Window(rev::command::WindowCommand::Delete));
    rpc.error("buffers.show", json!({"buffer":result["stamp"]["buffer"],"pane":{"session":pane["session"],"id":format!("pane-{closed}")},"placement":"current"}), "unsupported_invocation");
    rpc.editor.session.select_view(original).unwrap();
    let shown = rpc.ok(
        "buffers.show",
        json!({"buffer":result["stamp"]["buffer"],"pane":pane,"placement":"current"}),
    );
    assert_eq!(shown["pane"], pane);
    assert_eq!(
        rpc.editor.session.buffer().unwrap().name(),
        "Display result"
    );
    assert!(rpc.editor.session.buffer().unwrap().is_readonly());
}

// Generated native inventories are excluded even if a peer guesses a live ID.
// Other public schema methods remain unavailable until their approved slices.
#[test]
fn unsupported_resources() {
    let mut rpc = Rpc::new("non-modal");
    let source = source(&mut rpc);
    rpc.editor
        .session
        .execute(Command::Plugin(PluginCommand::List));
    let generated = rpc.editor.session.buffer().unwrap().id();
    let handle =
        json!({"session":source["stamp"]["buffer"]["session"],"id":format!("buffer-{generated}")});
    rpc.error(
        "buffers.describe",
        json!({"buffer":handle}),
        "unsupported_resource",
    );
    assert!(
        !rpc.list()
            .iter()
            .any(|b| b["stamp"]["buffer"]["id"] == format!("buffer-{generated}"))
    );
    rpc.editor
        .session
        .execute(Command::Terminal(rev::command::TerminalCommand::NewBuffer));
    rpc.editor.wait(|e| e.session.terminal_buffer().is_some());
    let terminal = rpc.editor.session.terminal_buffer().unwrap().id();
    let handle =
        json!({"session":source["stamp"]["buffer"]["session"],"id":format!("buffer-{terminal}")});
    rpc.error(
        "buffers.describe",
        json!({"buffer":handle}),
        "unsupported_resource",
    );
    assert!(
        !rpc.list()
            .iter()
            .any(|b| b["stamp"]["buffer"]["id"] == format!("buffer-{terminal}"))
    );
    rpc.error("buffers.save", json!({}), "invalid_arguments");
    rpc.error("events.subscribe", json!({}), "invalid_arguments");
}

// The byte ceiling is checked on a native scalar slice before copying it. Large
// file ingress stays hidden; adjacent reads can recover the entire source.
#[test]
fn read_byte_limit() {
    let mut rpc = Rpc::new("non-modal");
    let path = rpc.editor.fixture.write("large.txt", &"🦀".repeat(140000));
    let opened = rpc.ok("buffers.open", json!({"path":path}));
    rpc.error("buffers.read", json!({"buffer":opened["stamp"]["buffer"],"range":{"start":0,"end":140000},"expected_revision":null}), "limit_exceeded");
    for (start, end) in [(0, 131072), (131072, 140000), (140000, 140000)] {
        let result = rpc.ok("buffers.read", json!({"buffer":opened["stamp"]["buffer"],"range":{"start":start,"end":end},"expected_revision":opened["stamp"]["revision"]}));
        assert_eq!(
            result["text"].as_str().unwrap().chars().count(),
            end - start
        );
    }
    let oversized = rpc.editor.fixture.0.join("oversized.txt");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(16 * 1024 * 1024 + 1)
        .unwrap();
    rpc.error("buffers.open", json!({"path":oversized}), "limit_exceeded");
    assert!(
        !rpc.list()
            .iter()
            .any(|b| b["path"] == oversized.to_str().unwrap())
    );
}

// A callback's original scalar selection, pane and revision survive a focus
// change. An explicit handle read still resolves its original source.
#[test]
fn captured_context() {
    use rev::command::{Motion, RegionCommand};
    let mut rpc = Rpc::new("non-modal");
    rpc.editor
        .session
        .execute(Command::Motion(Motion::ForwardChar));
    rpc.editor
        .session
        .execute(Command::Region(RegionCommand::Set));
    rpc.editor
        .session
        .execute(Command::Motion(Motion::ForwardChar));
    let captured = source(&mut rpc);
    let pane = rpc.editor.session.windows().selected_id();
    rpc.editor.call("field.notes.collect");
    let other = rpc.editor.fixture.write("focus.txt", "other\n");
    rpc.editor.session.load(&other).unwrap();
    let read = rpc.ok("buffers.read", json!({"buffer":captured["stamp"]["buffer"],"range":{"start":0,"end":6},"expected_revision":captured["stamp"]["revision"]}));
    assert_eq!(read["text"], "source");
    rpc.editor.wait(|e| e.path("calls").exists());
    let calls = fs::read_to_string(rpc.editor.path("calls")).unwrap();
    let call: Value = serde_json::from_str(calls.lines().next().unwrap()).unwrap();
    assert_eq!(call["params"]["context"]["stamp"], captured["stamp"]);
    assert_eq!(
        call["params"]["context"]["pane"]["id"],
        format!("pane-{pane}")
    );
    assert_eq!(call["params"]["context"]["point"], 2);
    assert_eq!(
        call["params"]["context"]["selection"],
        json!({"start":1,"end":2})
    );
    assert_eq!(
        rpc.editor.session.buffer().unwrap().file().unwrap().path(),
        other
    );
}

// A generation stopped with an admitted native request cannot publish that
// queued creation. Retirement releases permits before a new generation starts.
#[test]
fn retired_creation() {
    use std::time::{Duration, Instant};
    let mut rpc = Rpc::new("non-modal");
    fs::write(rpc.editor.path("request-0"), format!("{}\n", json!({"jsonrpc":"2.0","id":"queued-creation","method":"buffers.create","params":{"name":"Retired result","text":"must not appear","read_only":false}}))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while rpc.editor.status().pending == 0 {
        assert!(Instant::now() < deadline, "child must submit the request");
        std::thread::sleep(Duration::from_millis(1));
    }
    // This noninteractive barrier retires before running another native pass.
    rpc.editor.session.shutdown_plugins().unwrap();
    assert!(!rpc.editor.path("response-0").exists());
    let source = rpc.editor.session.buffer().unwrap().id();
    rpc.switch("Retired result");
    assert!(
        rpc.editor
            .session
            .echo()
            .contains("No buffer with that name")
    );
    assert_eq!(rpc.editor.session.buffer().unwrap().id(), source);
}

// Departure consent captures a finite source set. New hidden output must not
// bypass it, while nonmutating reads remain usable and C-g retains live text.
#[test]
fn departure_ownership() {
    let mut rpc = Rpc::new("non-modal");
    let source = source(&mut rpc);
    rpc.editor.session.insert('!');
    rpc.editor.session.execute(Command::Quit);
    let question = rpc.editor.session.echo().to_owned();
    assert!(question.contains("y=save"), "{question}");
    rpc.error(
        "buffers.create",
        json!({"name":"Late output","text":"unsaved","read_only":false}),
        "busy",
    );
    let path = rpc.editor.fixture.write("late-open.txt", "late");
    rpc.error("buffers.open", json!({"path":path}), "busy");
    let read = rpc.ok("buffers.read", json!({"buffer":source["stamp"]["buffer"],"range":{"start":0,"end":1},"expected_revision":null}));
    assert_eq!(read["text"], "!");
    assert_eq!(rpc.editor.session.echo(), question);
    rpc.editor.session.execute(Command::KeyboardQuit);
    let result = create(&mut rpc, "Late output", "retained", false);
    assert_eq!(result["modified"], true);
}
