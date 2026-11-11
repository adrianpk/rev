use super::lifecycle::Editor;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    command::{Command, History, MacroCommand, MacroName, PluginCommand},
    input::keymap::Sequence,
    macros::{Status, StopReason, Wait},
    plugins::lifecycle::State,
};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};

const SEMANTIC: &str = "parameters=[{name='label', kind='string', required=false}, {name='flag', kind='boolean', required=false}, {name='data', kind='object', required=false}, {name='nothing', kind='null', required=false}]\nprefix='integer'\nrecording='semantic'";
fn press(e: &mut Editor, sequence: &str) {
    for key in Sequence::parse(sequence).unwrap().chords() {
        e.session
            .handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
    }
}
fn enter(e: &mut Editor, value: &str) {
    e.session.handle(Event::Paste(value.into()));
    e.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}
fn named(e: &mut Editor, command: &str) {
    press(e, "M-x");
    enter(e, command);
}
fn calls(e: &Editor) -> Vec<Value> {
    fs::read_to_string(e.path("calls"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
fn text(e: &Editor) -> String {
    let b = e.session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn complete(e: &mut Editor, count: usize) {
    e.wait(|e| {
        calls(e).len() == count
            && e.status().pending == 0
            && e.session.macro_wait() != Some(Wait::Plugin)
    });
}
fn record(e: &mut Editor) {
    press(e, "F3");
    assert_eq!(e.session.macro_status(), Status::Recording);
    press(e, "M-3 F8");
    complete(e, 1);
    press(e, "F4");
    assert_eq!(
        e.session.macro_status(),
        Status::Idle,
        "{}",
        e.session.echo()
    );
}
fn name(e: &mut Editor, value: &str) {
    e.session.execute(Command::Macro(MacroCommand::Name));
    enter(e, value);
}
fn save(e: &mut Editor, value: &str) {
    e.session.execute(Command::Macro(MacroCommand::Save));
    enter(e, value);
    press(e, "y");
}
fn library(e: &Editor) -> std::path::PathBuf {
    e.fixture.0.join("state/macros/library.toml")
}
fn restore(e: &mut Editor, value: &str) {
    let parent = library(e).parent().unwrap().to_owned();
    fs::create_dir_all(&parent).unwrap();
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(library(e), value).unwrap();
    fs::set_permissions(library(e), fs::Permissions::from_mode(0o600)).unwrap();
    e.session.execute(Command::Macro(MacroCommand::Load));
}
fn call(e: &mut Editor, value: &str) {
    e.session
        .execute(Command::MacroNamed(MacroName::new(value).unwrap()));
}
fn saved() -> String {
    let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    record(&mut e);
    name(&mut e, "stored-call");
    save(&mut e, "stored-call");
    fs::read_to_string(library(&e)).unwrap()
}

// Each native prefix is consumed by one callback, including zero and negative
// values. A declaration without integer policy refuses before starting a peer.
#[test]
fn prefix_policies() {
    for profile in ["non-modal", "modal"] {
        let mut e = Editor::recording(profile, "recording", Some(SEMANTIC));
        press(&mut e, "F8");
        complete(&mut e, 1);
        press(&mut e, "M-3 F8");
        complete(&mut e, 2);
        press(&mut e, "M-- F8");
        complete(&mut e, 3);
        press(&mut e, "M-0");
        named(&mut e, "field.notes.collect");
        complete(&mut e, 4);
        assert_eq!(
            calls(&e)
                .iter()
                .map(|c| c["params"]["context"]["prefix"].clone())
                .collect::<Vec<_>>(),
            vec![Value::Null, json!(3), json!(-1), json!(0)]
        );
        press(&mut e, "F8");
        complete(&mut e, 5);
        assert!(calls(&e)[4]["params"]["context"]["prefix"].is_null());
        let mut refused = Editor::recording(
            profile,
            "recording",
            Some("parameters=[]\nprefix='none'\nrecording='reject'"),
        );
        press(&mut refused, "M-3 F8");
        assert_eq!(refused.status().state, State::Installed);
        assert!(!refused.path("starts").exists());
        press(&mut refused, "F3 F8");
        assert_eq!(refused.session.macro_status(), Status::Idle);
        assert_eq!(
            refused.session.macro_stop().unwrap().reason,
            StopReason::NativeFailure
        );
        assert!(!refused.path("starts").exists());
    }
}

// A semantic callback may publish checked native edits during capture and replay.
// Exactly one plugin step survives; generated edits are never captured twice.
#[test]
fn semantic_calls() {
    for profile in ["non-modal", "modal"] {
        let mut e = Editor::recording(profile, "recording", Some(SEMANTIC));
        fs::write(e.path("edit"), "").unwrap();
        record(&mut e);
        assert_eq!(text(&e), "λsource 🦀\n");
        press(&mut e, "M-3 F4");
        e.wait(|e| e.session.macro_status() == Status::Idle);
        assert!(e.session.macro_stop().is_none(), "{}", e.session.echo());
        assert_eq!(text(&e), "λλλλsource 🦀\n");
        assert_eq!(calls(&e).len(), 4);
        e.session.execute(Command::History(History::Undo));
        assert_eq!(text(&e), "λλλsource 🦀\n");
        e.session.execute(Command::History(History::Redo));
        assert_eq!(text(&e), "λλλλsource 🦀\n");
        assert!(
            calls(&e)
                .iter()
                .all(|c| c["params"]["context"]["prefix"] == 3)
        );
        name(&mut e, "stored-call");
        save(&mut e, "stored-call");
        let data: toml::Value = toml::from_str(&fs::read_to_string(library(&e)).unwrap()).unwrap();
        assert_eq!(data["schema_version"].as_integer(), Some(2));
        let steps = data["macros"][0]["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["kind"].as_str(), Some("plugin-call"));
        assert_eq!(steps[0]["command"].as_str(), Some("field.notes.collect"));
        assert!(MacroName::new("field.notes.collect").is_err());
    }
}

// Explicit mixed-library saves upgrade the aggregate without changing legacy
// record identities/content. Loading either schema is read-only and inert.
#[test]
fn library_compatibility() {
    let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    press(&mut e, "F3 C-f F4");
    name(&mut e, "legacy-call");
    save(&mut e, "legacy-call");
    let old_bytes = fs::read(library(&e)).unwrap();
    let old: toml::Value = toml::from_str(std::str::from_utf8(&old_bytes).unwrap()).unwrap();
    assert_eq!(old["schema_version"].as_integer(), Some(1));
    e.session.execute(Command::Macro(MacroCommand::Load));
    assert_eq!(fs::read(library(&e)).unwrap(), old_bytes);
    record(&mut e);
    name(&mut e, "stored-call");
    save(&mut e, "stored-call");
    let bytes = fs::read(library(&e)).unwrap();
    let new: toml::Value = toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap();
    assert_eq!(new["schema_version"].as_integer(), Some(2));
    assert_eq!(new["macros"][0], old["macros"][0]);
    let mut fresh = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    restore(&mut fresh, std::str::from_utf8(&bytes).unwrap());
    assert_eq!(fs::read(library(&fresh)).unwrap(), bytes);
    assert_eq!(fresh.status().state, State::Installed);
    assert!(!fresh.path("starts").exists());
    call(&mut fresh, "legacy-call");
    fresh.wait(|e| e.session.macro_status() == Status::Idle);
    assert!(
        fresh.session.macro_stop().is_none(),
        "{}",
        fresh.session.echo()
    );
    assert_eq!(fresh.session.view().point.position.0, 1);
    call(&mut fresh, "stored-call");
    fresh.wait(|e| e.session.macro_status() == Status::Idle);
    assert_eq!(calls(&fresh).len(), 1);
}

// Stored JSON remains literal data, including nested null/boolean values. The
// current package declaration must admit the complete arguments and prefix.
#[test]
fn literal_arguments() {
    let original = saved();
    let arguments =
        json!({"label":"λ","flag":true,"data":{"null":null,"values":[3,false]},"nothing":null});
    let bytes = original
        .replace(
            "arguments_json = \"{}\"",
            &format!(
                "arguments_json = {}",
                toml_edit::Value::from(arguments.to_string())
            ),
        )
        .replace("prefix = 3", "prefix = -7");
    assert_ne!(bytes, original);
    let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    restore(&mut e, &bytes);
    call(&mut e, "stored-call");
    e.wait(|e| e.session.macro_status() == Status::Idle);
    assert!(e.session.macro_stop().is_none(), "{}", e.session.echo());
    let call = calls(&e).remove(0);
    assert_eq!(call["params"]["arguments"], arguments);
    assert_eq!(call["params"]["context"]["prefix"], -7);
    assert_eq!(fs::read_to_string(library(&e)).unwrap(), bytes);
}

// Absent packages and changed command contracts stop before activation. Unknown
// saved variants reject the whole aggregate without executing inert records.
#[test]
fn replay_admission() {
    let bytes = saved();
    for declaration in [
        None,
        Some("parameters=[]\nprefix='none'\nrecording='semantic'"),
        Some("parameters=[]\nprefix='integer'\nrecording='reject'"),
        Some(
            "parameters=[{name='required',kind='string',required=true}]\nprefix='integer'\nrecording='semantic'",
        ),
    ] {
        let mut e = Editor::recording("non-modal", "recording", declaration);
        restore(&mut e, &bytes);
        call(&mut e, "stored-call");
        e.wait(|e| e.session.macro_status() == Status::Idle);
        assert!(e.session.macro_stop().is_some());
        assert!(!e.path("starts").exists());
        assert_eq!(text(&e), "source 🦀\n");
        assert_eq!(fs::read_to_string(library(&e)).unwrap(), bytes);
    }
    for variant in ["unknown", "schema-one", "foreign-identity"] {
        let bytes = match variant {
            "unknown" => bytes.replace("kind = \"plugin-call\"", "kind = \"unknown-call\""),
            "schema-one" => bytes.replace("schema_version = 2", "schema_version = 1"),
            _ => bytes.replace("field.notes.collect", "rev.native.collect"),
        };
        let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
        restore(&mut e, &bytes);
        assert!(
            e.session.echo().contains("Invalid macro library"),
            "{}",
            e.session.echo()
        );
        assert!(!e.path("starts").exists());
    }
}

// A delayed callback suspends automatic steps without a busy wake. C-g retires
// its authority; late completion cannot continue a canceled macro.
#[test]
fn cooperative_cancel() {
    let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    record(&mut e);
    name(&mut e, "stored-call");
    e.manage(PluginCommand::Deactivate);
    e.wait(|e| e.status().state == State::Inactive);
    fs::write(e.path("mode"), "slow-call").unwrap();
    call(&mut e, "stored-call");
    e.wait(|e| calls(e).len() == 2);
    assert_eq!(e.session.macro_status(), Status::Playing);
    for _ in 0..4 {
        e.session.expire_input(Instant::now());
    }
    assert_ne!(
        e.session.remaining_input(Instant::now()),
        Some(Duration::ZERO)
    );
    press(&mut e, "x");
    assert_eq!(text(&e), "source 🦀\n");
    press(&mut e, "C-g");
    assert_eq!(e.session.macro_status(), Status::Idle);
    assert_eq!(e.session.macro_stop().unwrap().reason, StopReason::Canceled);
    fs::write(e.path("complete"), "").unwrap();
    e.wait(|e| e.status().state == State::Inactive);
    assert_eq!(calls(&e).len(), 2);
    assert_eq!(text(&e), "source 🦀\n");
}

// Nested semantic callbacks retain one causal operation and stop at depth eight;
// RPC recursion is not re-recorded as additional user commands.
#[test]
fn nested_bounds() {
    let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    record(&mut e);
    name(&mut e, "stored-call");
    fs::write(e.path("nested"), "other.notes.collect").unwrap();
    fs::write(
        e.fixture.0.join("other.notes/nested"),
        "field.notes.collect",
    )
    .unwrap();
    call(&mut e, "stored-call");
    e.wait(|e| e.session.macro_status() == Status::Idle);
    assert_eq!(
        e.session.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
    assert!(
        e.session.echo().contains("LimitExceeded"),
        "{}",
        e.session.echo()
    );
    let mut calls = calls(&e).into_iter().skip(1).collect::<Vec<_>>();
    calls.extend(
        fs::read_to_string(e.fixture.0.join("other.notes/calls"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap()),
    );
    calls.sort_by_key(|call| {
        call["params"]["context"]["origin"]["depth"]
            .as_u64()
            .unwrap()
    });
    assert_eq!(calls.len(), 9);
    let origin = &calls[0]["params"]["context"]["origin"]["operation"];
    for (depth, call) in calls.iter().enumerate() {
        assert_eq!(call["params"]["context"]["origin"]["depth"], depth);
        assert_eq!(&call["params"]["context"]["origin"]["operation"], origin);
    }
    assert_eq!(text(&e), "source 🦀\n");
}

// An otherwise valid unsolicited edit has no authority inside a semantic call.
// Its refusal leaves source and recording intact until explicit cancellation.
#[test]
fn unrelated_edits() {
    use super::buffers::{Rpc, source};
    let mut e = Editor::recording("non-modal", "buffers", Some(SEMANTIC));
    e.manage(PluginCommand::Activate);
    e.wait(|e| e.status().state == State::Active);
    let mut rpc = Rpc::from_editor(e);
    let captured = source(&mut rpc);
    press(&mut rpc.editor, "F3 F8");
    rpc.error("edits.apply", &json!({"operation":"unrelated", "origin":{"package":null,"operation":"unrelated","depth":0}, "targets":[{"stamp":captured["stamp"], "replacements":[{"range":{"start":0,"end":0},"text":"wrong"}]}]}), "busy");
    assert_eq!(text(&rpc.editor), "source 🦀\n");
    assert_eq!(rpc.editor.session.macro_status(), Status::Recording);
    press(&mut rpc.editor, "C-g");
    assert_eq!(
        rpc.editor.session.macro_stop().unwrap().reason,
        StopReason::Canceled
    );
}

// Async completion cannot bless an unrelated native source change. The runner
// retains its expected source point across the wait and stops before continuing.
#[test]
fn replay_owner() {
    let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    record(&mut e);
    name(&mut e, "stored-call");
    e.manage(PluginCommand::Deactivate);
    e.wait(|e| e.status().state == State::Inactive);
    fs::write(e.path("mode"), "slow-call").unwrap();
    call(&mut e, "stored-call");
    e.wait(|e| calls(e).len() == 2);
    e.session.replace(0..0, "external ").unwrap();
    fs::write(e.path("complete"), "").unwrap();
    e.wait(|e| e.session.macro_status() == Status::Idle);
    assert_eq!(
        e.session.macro_stop().unwrap().reason,
        StopReason::OwnerChanged
    );
    assert_eq!(text(&e), "external source 🦀\n");
    assert_eq!(calls(&e).len(), 2);
}

// A nested callee owns its generated edits even when another package originated
// the call. The host derives causal authority without accepting a foreign owner.
#[test]
fn nested_edits() {
    let mut e = Editor::recording("non-modal", "recording", Some(SEMANTIC));
    fs::write(e.path("nested"), "other.notes.collect").unwrap();
    fs::write(e.fixture.0.join("other.notes/edit"), "").unwrap();
    record(&mut e);
    assert_eq!(text(&e), "λsource 🦀\n");
    press(&mut e, "F4");
    e.wait(|e| e.session.macro_status() == Status::Idle);
    assert!(e.session.macro_stop().is_none(), "{}", e.session.echo());
    assert_eq!(text(&e), "λλsource 🦀\n");
    assert_eq!(calls(&e).len(), 2);
    let nested = fs::read_to_string(e.fixture.0.join("other.notes/calls")).unwrap();
    assert_eq!(nested.lines().count(), 2);
    for line in nested.lines() {
        let call: Value = serde_json::from_str(line).unwrap();
        assert_eq!(call["params"]["command"], "other.notes.collect");
        assert_eq!(call["params"]["context"]["origin"]["depth"], 1);
        assert_eq!(call["params"]["context"]["prefix"], 3);
    }
}
