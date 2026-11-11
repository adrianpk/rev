use super::buffers::{Rpc, create, source};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, WindowCommand};
use serde_json::{Value, json};
use std::fs;

fn describe(rpc: &mut Rpc, buffer: &Value) -> Value {
    rpc.ok(
        "buffers.describe",
        &json!({"buffer":buffer["stamp"]["buffer"]}),
    )
}
fn save(rpc: &mut Rpc, buffer: &Value, path: &Value) -> Value {
    rpc.ok("buffers.save", &json!({"buffer":buffer["stamp"]["buffer"], "expected_revision":buffer["stamp"]["revision"], "expected_disk":buffer["expected_disk"], "path":path}))
}
fn context(rpc: &mut Rpc) -> Value {
    rpc.editor.call("field.notes.collect");
    rpc.ok(
        "commands.list",
        &json!({"page":{"cursor":null,"limit":1},"query":"forward-char"}),
    );
    let calls = fs::read_to_string(rpc.editor.path("calls")).unwrap();
    let call: Value = serde_json::from_str(calls.lines().last().unwrap()).unwrap();
    call["params"]["context"].clone()
}
fn invoke(command: &str, context: &Value) -> Value {
    json!({"command":command,"arguments":{},"context":context})
}

// Save-as creates only the explicitly absent destination. Captured hidden text
// and native BOM/CRLF encoding survive focus changes and clean close.
#[test]
fn explicit_save_close() {
    for profile in ["non-modal", "modal"] {
        let mut rpc = Rpc::new(profile);
        let original = rpc.editor.session.buffer().unwrap().id();
        let result = create(&mut rpc, "Saved result", "🦀\r\n", false);
        rpc.error("buffers.close", &json!({"buffer":result["stamp"]["buffer"],"expected_revision":result["stamp"]["revision"]}), "confirmation_required");
        let path = rpc.editor.fixture.0.join("saved.txt");
        let saved = save(&mut rpc, &result, &json!(path));
        assert_eq!(saved["stamp"], result["stamp"]);
        assert_eq!(fs::read_to_string(&path).unwrap(), "🦀\r\n");
        let current = describe(&mut rpc, &result);
        assert_eq!(current["modified"], false);
        assert_eq!(current["expected_disk"], saved["expected_disk"]);
        assert_eq!(rpc.editor.session.buffer().unwrap().id(), original);
        rpc.ok("buffers.close", &json!({"buffer":result["stamp"]["buffer"],"expected_revision":result["stamp"]["revision"]}));
        rpc.error(
            "buffers.describe",
            &json!({"buffer":result["stamp"]["buffer"]}),
            "missing_buffer",
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "🦀\r\n");
    }
}

// Every refusal preserves disk bytes and the native unsaved state. An external
// observation is never refreshed into overwrite authority by a plugin save.
#[test]
fn save_preconditions() {
    let mut rpc = Rpc::new("non-modal");
    let path = rpc.editor.fixture.write("existing.txt", "original\n");
    let opened = rpc.ok("buffers.open", &json!({"path":path}));
    rpc.switch(opened["name"].as_str().unwrap());
    rpc.editor.session.handle(Event::Paste("edited ".into()));
    let changed = describe(&mut rpc, &opened);
    for (label, revision, disk, kind) in [
        (
            "source",
            opened["stamp"]["revision"].clone(),
            opened["expected_disk"].clone(),
            "stale_revision",
        ),
        (
            "disk",
            changed["stamp"]["revision"].clone(),
            json!({"kind":"absent"}),
            "state_conflict",
        ),
    ] {
        rpc.error("buffers.save", &json!({"buffer":opened["stamp"]["buffer"],"expected_revision":revision,"expected_disk":disk,"path":null}), kind);
        assert_eq!(fs::read_to_string(&path).unwrap(), "original\n", "{label}");
    }
    fs::write(&path, "external\n").unwrap();
    rpc.error("buffers.save", &json!({"buffer":changed["stamp"]["buffer"],"expected_revision":changed["stamp"]["revision"],"expected_disk":changed["expected_disk"],"path":null}), "state_conflict");
    assert_eq!(fs::read_to_string(&path).unwrap(), "external\n");
    assert_eq!(describe(&mut rpc, &opened)["modified"], true);
    let unsaved = create(&mut rpc, "Unsaved", "owned", false);
    rpc.error("buffers.save", &json!({"buffer":unsaved["stamp"]["buffer"],"expected_revision":unsaved["stamp"]["revision"],"expected_disk":{"kind":"absent"},"path":path}), "confirmation_required");
    assert_eq!(fs::read_to_string(&path).unwrap(), "external\n");
    let readonly = create(&mut rpc, "Read only", "keep", true);
    let absent = rpc.editor.fixture.0.join("absent.txt");
    rpc.error("buffers.save", &json!({"buffer":readonly["stamp"]["buffer"],"expected_revision":readonly["stamp"]["revision"],"expected_disk":{"kind":"absent"},"path":absent}), "read_only");
    assert!(!absent.exists());
}

// Paginated metadata states the admission boundary. A motion acts on the
// captured pane even after another pane receives native focus.
#[test]
fn captured_native_motion() {
    for profile in ["non-modal", "modal"] {
        let mut rpc = Rpc::new(profile);
        let source = source(&mut rpc);
        let captured = context(&mut rpc);
        let catalog = rpc.ok(
            "commands.list",
            &json!({"page":{"cursor":null,"limit":1},"query":"forward-char"}),
        );
        assert_eq!(catalog["commands"][0]["noninteractive"], true);
        assert!(!catalog["next"].is_null());
        let second = rpc.ok(
            "commands.list",
            &json!({"page":{"cursor":catalog["next"],"limit":1},"query":"forward-char"}),
        );
        assert_ne!(second["commands"][0]["id"], catalog["commands"][0]["id"]);
        let captured_pane = rpc.editor.session.windows().selected_id();
        rpc.editor
            .session
            .execute(Command::Split(rev::window::Split::Right));
        rpc.editor
            .session
            .execute(Command::Window(WindowCommand::Other));
        let selected = rpc.editor.session.windows().selected_id();
        assert_ne!(captured_pane, selected);
        rpc.ok("commands.invoke", &invoke("end-of-buffer", &captured));
        assert_eq!(rpc.editor.session.windows().selected_id(), selected);
        assert_eq!(
            rpc.editor
                .session
                .windows()
                .get(captured_pane)
                .unwrap()
                .point
                .position
                .0 as u64,
            source["scalar_len"].as_u64().unwrap()
        );
        assert_eq!(rpc.editor.session.windows().selected().point.position.0, 0);
        for (command, kind) in [
            ("save-buffer", "confirmation_required"),
            ("execute-extended-command", "unsupported_invocation"),
            ("missing-command", "command_unavailable"),
        ] {
            rpc.error("commands.invoke", &invoke(command, &captured), kind);
        }
        let mut invalid = invoke("forward-char", &captured);
        invalid["arguments"] = json!({"text":"not admitted"});
        rpc.error("commands.invoke", &invalid, "invalid_arguments");
    }
}

// Nested qualified calls preserve typed arguments, actual results and origin.
// Untrusted parents and stale source captures cannot refresh their authority.
#[test]
fn qualified_calls() {
    let mut rpc = Rpc::new("non-modal");
    let captured = context(&mut rpc);
    fs::write(rpc.editor.path("reply-calls"), "ready").unwrap();
    let mut params = invoke("field.notes.collect", &captured);
    params["arguments"] = json!({"label":"🦀"});
    let result = rpc.ok("commands.invoke", &params);
    assert_eq!(result["value"]["arguments"], params["arguments"]);
    assert_eq!(result["value"]["context"]["origin"]["depth"], 1);
    assert_eq!(
        result["value"]["context"]["origin"]["operation"],
        captured["origin"]["operation"]
    );
    assert_eq!(
        result["value"]["context"]["origin"]["package"],
        "field.notes"
    );
    params["arguments"] = json!({"label":123});
    rpc.error("commands.invoke", &params, "invalid_arguments");
    for field in ["generation", "invocation"] {
        let mut forged = captured.clone();
        forged[field] = json!("9999");
        rpc.error(
            "commands.invoke",
            &invoke("forward-char", &forged),
            "unsupported_invocation",
        );
    }
    let mut forged = captured.clone();
    forged["origin"]["depth"] = json!(0);
    forged["origin"]["operation"] = json!("forged");
    rpc.error(
        "commands.invoke",
        &invoke("forward-char", &forged),
        "unsupported_invocation",
    );
    for (field, value, kind) in [
        ("session", json!("foreign"), "missing_buffer"),
        (
            "pane",
            json!({"session":captured["pane"]["session"],"id":"pane-missing"}),
            "unsupported_invocation",
        ),
        ("point", json!(9999), "invalid_range"),
        ("selection", json!({"start":0,"end":9999}), "invalid_range"),
    ] {
        let mut invalid = captured.clone();
        if field == "session" {
            invalid["stamp"]["buffer"]["session"] = value.clone();
            invalid["pane"]["session"] = value;
        } else {
            invalid[field] = value;
        }
        rpc.error("commands.invoke", &invoke("forward-char", &invalid), kind);
    }
    rpc.editor.session.handle(Event::Paste("!".into()));
    rpc.error(
        "commands.invoke",
        &invoke("forward-char", &captured),
        "stale_revision",
    );
}

// Recursive handlers cannot escape combined pending/call-depth bounds. The
// failure traverses real correlated calls without replacing the native host.
#[test]
fn nested_bound() {
    let mut rpc = Rpc::new("non-modal");
    let captured = context(&mut rpc);
    fs::write(rpc.editor.path("recursive"), "ready").unwrap();
    let response = rpc.request("commands.invoke", &invoke("field.notes.collect", &captured));
    assert!(
        ["limit_exceeded", "busy"].contains(&response["error"]["data"]["kind"].as_str().unwrap()),
        "{response}"
    );
    let calls = fs::read_to_string(rpc.editor.path("calls")).unwrap();
    assert!((2..=9).contains(&calls.lines().count()));
    for line in calls.lines() {
        let call: Value = serde_json::from_str(line).unwrap();
        assert!(
            call["params"]["context"]["origin"]["depth"]
                .as_u64()
                .unwrap()
                <= 8
        );
    }
}

// Native prompts retain input, and C-g retires queued work before any save,
// close or command can acquire authority. Dirty content survives cancellation.
#[test]
fn consent_and_cancel() {
    let mut rpc = Rpc::new("non-modal");
    let captured = context(&mut rpc);
    let result = create(&mut rpc, "Kept", "keep", false);
    rpc.editor.session.execute(Command::Quit);
    rpc.error(
        "commands.invoke",
        &invoke("forward-char", &captured),
        "unsupported_invocation",
    );
    rpc.error(
        "buffers.close",
        &json!({"buffer":result["stamp"]["buffer"],"expected_revision":result["stamp"]["revision"]}),
        "busy",
    );
    rpc.editor.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    let path = rpc.editor.fixture.0.join("cancelled.txt");
    rpc.enqueue("buffers.save", &json!({"buffer":result["stamp"]["buffer"],"expected_revision":result["stamp"]["revision"],"expected_disk":result["expected_disk"],"path":path}));
    rpc.editor.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    rpc.editor
        .wait(|e| e.status().state == rev::plugins::lifecycle::State::Inactive);
    assert!(!path.exists());
}

// Associated saves preserve native encoding and remain independent of focus.
// Closing a clean source replaces all of its shared views through native removal.
#[test]
fn associated_save() {
    let mut rpc = Rpc::new("non-modal");
    let path = rpc.editor.fixture.write("encoded.txt", "\u{feff}old\r\n");
    let opened = rpc.ok("buffers.open", &json!({"path":path}));
    rpc.switch(opened["name"].as_str().unwrap());
    let id = rpc.editor.session.buffer().unwrap().id();
    rpc.editor.session.handle(Event::Paste("new ".into()));
    let changed = describe(&mut rpc, &opened);
    rpc.switch("*scratch*");
    let selected = rpc.editor.session.buffer().unwrap().id();
    save(&mut rpc, &changed, &Value::Null);
    assert_eq!(rpc.editor.session.buffer().unwrap().id(), selected);
    assert_eq!(fs::read_to_string(&path).unwrap(), "\u{feff}new old\r\n");
    rpc.switch(opened["name"].as_str().unwrap());
    rpc.editor
        .session
        .execute(Command::Split(rev::window::Split::Right));
    assert_eq!(
        rpc.editor
            .session
            .windows()
            .iter()
            .filter(|(_, v)| v.point.stamp.buffer == id)
            .count(),
        2
    );
    rpc.ok("buffers.close", &json!({"buffer":changed["stamp"]["buffer"],"expected_revision":changed["stamp"]["revision"]}));
    assert!(
        rpc.editor
            .session
            .windows()
            .iter()
            .all(|(_, v)| v.point.stamp.buffer != id)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "\u{feff}new old\r\n");
}
