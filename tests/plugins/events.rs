use super::{
    buffers::{Rpc, create, source},
    lifecycle::Editor,
};
use rev::{
    command::{Command, History, PluginCommand},
    plugins::lifecycle::State,
};
use serde_json::{Value, json};
use std::{
    fs,
    time::{Duration, Instant},
};

const IDS: [&str; 8] = [
    "field.notes",
    "other.notes",
    "third.notes",
    "fourth.notes",
    "fifth.notes",
    "sixth.notes",
    "seventh.notes",
    "eighth.notes",
];
fn lines(e: &Editor, id: &str, file: &str) -> Vec<Value> {
    fs::read_to_string(e.fixture.0.join(id).join(file))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
fn events(e: &Editor, id: &str) -> Vec<Value> {
    lines(e, id, "events")
        .into_iter()
        .map(|v| v["params"].clone())
        .collect()
}
fn rpc(profile: &str) -> Rpc {
    let mut e = Editor::events(profile, 1);
    e.wait(|e| e.status().state == State::Active);
    Rpc::from_editor(e)
}
fn subscribe(rpc: &mut Rpc, kinds: &[&str], buffers: Vec<Value>) -> Value {
    rpc.ok("events.subscribe", json!({"kinds":kinds,"buffers":buffers}))
}
fn edit(event: &Value, stamp: &Value) -> Value {
    let mut origin = event["origin"].clone();
    origin["package"] = "field.notes".into();
    json!({"operation":"callback-edit", "origin":origin, "targets":[{"stamp":stamp,"replacements":[{"range":{"start":0,"end":0},"text":"!"}]}]})
}
fn wire(e: &mut Editor, id: &str, sequence: usize, method: &str, params: Value) -> Value {
    let directory = e.fixture.0.join(id);
    fs::write(directory.join(format!("request-{sequence}")), format!("{}\n",json!({"jsonrpc":"2.0","id":format!("buffer-request-{sequence}"),"method":method,"params":params}))).unwrap();
    e.wait(|_| directory.join(format!("response-{sequence}")).exists());
    let value: Value =
        serde_json::from_slice(&fs::read(directory.join(format!("response-{sequence}"))).unwrap())
            .unwrap();
    assert!(value.get("error").is_none(), "{value}");
    value["result"].clone()
}

// A real child sees every revision, including undo/redo and hidden atomic
// participants. Notifications name committed stamps without taking focus.
#[test]
fn publications() {
    for profile in ["non-modal", "modal"] {
        let mut rpc = rpc(profile);
        let a = source(&mut rpc);
        let sub = subscribe(&mut rpc, &["opened", "changed", "closed"], vec![]);
        let b = create(&mut rpc, "Hidden event target", "hidden", false);
        let params = json!({"operation":"native-batch","origin":{"package":"field.notes","operation":"native-batch","depth":0},"targets":[
            {"stamp":a["stamp"],"replacements":[{"range":{"start":0,"end":0},"text":"λ"}]},
            {"stamp":b["stamp"],"replacements":[{"range":{"start":0,"end":0},"text":"β"}]}]});
        let result = rpc.ok("edits.apply", params);
        rpc.editor.wait(|e| events(e, IDS[0]).len() >= 3);
        let observed = events(&rpc.editor, IDS[0]);
        assert_eq!(observed[0]["kind"], "opened");
        assert_eq!(observed[0]["origin"]["package"], "field.notes");
        assert_eq!(observed[0]["stamp"], b["stamp"]);
        for stamp in result["stamps"].as_array().unwrap() {
            assert!(observed.iter().any(|v| v["kind"] == "changed"
                && &v["stamp"] == stamp
                && v["origin"]["package"] == "field.notes"));
        }
        assert_eq!(
            rpc.editor
                .session
                .buffer()
                .unwrap()
                .stamp()
                .revision
                .to_string(),
            result["stamps"][0]["revision"]
        );
        assert_eq!(
            rpc.editor.session.windows().selected().point.stamp,
            rpc.editor.session.buffer().unwrap().stamp()
        );
        rpc.editor.session.execute(Command::History(History::Undo));
        rpc.editor.session.execute(Command::History(History::Redo));
        rpc.editor.wait(|e| events(e, IDS[0]).len() >= 5);
        // A clean hidden buffer can close through the explicit native policy.
        let clean = create(&mut rpc, "Clean close", "", false);
        rpc.ok("buffers.close", json!({"buffer":clean["stamp"]["buffer"],"expected_revision":clean["stamp"]["revision"]}));
        rpc.editor
            .wait(|e| events(e, IDS[0]).iter().any(|v| v["kind"] == "closed"));
        let observed = events(&rpc.editor, IDS[0]);
        let mut previous = sub["sequence"].as_str().unwrap().parse::<u64>().unwrap();
        for event in &observed {
            let next = event["sequence"].as_str().unwrap().parse::<u64>().unwrap();
            assert!(next > previous);
            previous = next;
            assert_eq!(event["subscription"], sub["subscription"]);
            assert_eq!(
                event["generation"].as_str().unwrap(),
                rpc.editor.status().generation.unwrap().as_str()
            );
        }
    }
}

// Filters and ownership apply before subscription publication. Unsubscribe
// revokes previously delivered callback authority, including already queued IO.
#[test]
fn subscriptions() {
    let mut rpc = rpc("non-modal");
    let a = source(&mut rpc);
    let handle = a["stamp"]["buffer"].clone();
    for (kinds, buffers, kind) in [
        (json!([]), json!([]), "invalid_arguments"),
        (
            json!(["changed", "changed"]),
            json!([]),
            "invalid_arguments",
        ),
        (
            json!(["changed"]),
            json!([handle.clone(), handle.clone()]),
            "invalid_arguments",
        ),
        (
            json!(["changed"]),
            json!([{"session":"foreign","id":"buffer-1"}]),
            "missing_buffer",
        ),
    ] {
        rpc.error(
            "events.subscribe",
            json!({"kinds":kinds,"buffers":buffers}),
            kind,
        );
    }
    let sub = subscribe(&mut rpc, &["changed"], vec![handle]);
    create(&mut rpc, "Filtered", "", false);
    rpc.editor.session.insert('x');
    rpc.editor.wait(|e| events(e, IDS[0]).len() == 1);
    let event = events(&rpc.editor, IDS[0])[0].clone();
    assert_eq!(event["stamp"]["buffer"], a["stamp"]["buffer"]);
    let context = json!({"invocation":format!("event:{}",event["sequence"].as_str().unwrap()),"generation":event["generation"],"origin":event["origin"],"stamp":event["stamp"],"pane":null,"point":null,"selection":null,"prefix":null});
    for (field, value) in [
        ("generation", json!("9999")),
        ("invocation", json!("event:999999")),
    ] {
        let mut forged = context.clone();
        forged[field] = value;
        rpc.error(
            "commands.invoke",
            json!({"command":"field.notes.collect","arguments":{},"context":forged}),
            "unsupported_invocation",
        );
    }
    rpc.ok(
        "commands.invoke",
        json!({"command":"field.notes.collect","arguments":{},"context":context}),
    );
    assert_eq!(
        lines(&rpc.editor, IDS[0], "calls")[0]["params"]["context"]["origin"]["depth"],
        1
    );
    // Removing one of two matching subscriptions must preserve the other's
    // authority for the same delivered causal root.
    let extra = subscribe(&mut rpc, &["changed"], vec![a["stamp"]["buffer"].clone()]);
    rpc.editor.session.insert('y');
    rpc.editor.wait(|e| events(e, IDS[0]).len() == 3);
    let shared = events(&rpc.editor, IDS[0]).last().unwrap().clone();
    rpc.ok(
        "events.unsubscribe",
        json!({"subscription":extra["subscription"]}),
    );
    let current = source(&mut rpc)["stamp"].clone();
    rpc.ok("edits.apply", edit(&shared, &current));
    rpc.ok(
        "events.unsubscribe",
        json!({"subscription":sub["subscription"]}),
    );
    let stamp = source(&mut rpc)["stamp"].clone();
    rpc.error("edits.apply", edit(&event, &stamp), "cancelled");
    rpc.error(
        "events.unsubscribe",
        json!({"subscription":sub["subscription"]}),
        "invalid_arguments",
    );
    for _ in 0..16 {
        subscribe(&mut rpc, &["changed"], vec![]);
    }
    rpc.error(
        "events.subscribe",
        json!({"kinds":["changed"],"buffers":[]}),
        "limit_exceeded",
    );
}

// Competing Rust processes alternate event-triggered edits along one root.
// The ninth response is refused rather than resetting depth at each package.
#[test]
fn causal_chain() {
    let mut e = Editor::events("non-modal", 2);
    for id in &IDS[..2] {
        wire(
            &mut e,
            id,
            0,
            "events.subscribe",
            json!({"kinds":["changed"],"buffers":[]}),
        );
        fs::write(e.fixture.0.join(id).join("react"), "").unwrap();
    }
    e.session.insert('x');
    e.wait(|e| {
        IDS[..2].iter().any(|id| {
            lines(e, id, "reactions")
                .iter()
                .any(|v| v["error"]["data"]["kind"] == "limit_exceeded")
        })
    });
    let mut chain = events(&e, IDS[0]);
    chain.sort_by_key(|v| v["origin"]["depth"].as_u64().unwrap());
    assert_eq!(
        chain
            .iter()
            .map(|v| v["origin"]["depth"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        (0..=8).collect::<Vec<_>>()
    );
    let root = &chain[0]["origin"]["operation"];
    assert!(chain.iter().all(|v| &v["origin"]["operation"] == root));
    assert_eq!(
        e.session.buffer().unwrap().len_chars(),
        "source 🦀\n".chars().count() + 9
    );
}

// Event-to-command-to-edit chains retain one root through a second process.
// Both nested command admission and its generated publication consume depth.
#[test]
fn nested_commands() {
    let mut e = Editor::events("non-modal", 2);
    wire(
        &mut e,
        IDS[0],
        0,
        "events.subscribe",
        json!({"kinds":["changed"],"buffers":[]}),
    );
    fs::write(e.fixture.0.join(IDS[0]).join("nested-react"), "").unwrap();
    fs::write(e.fixture.0.join(IDS[1]).join("command-edit"), "").unwrap();
    e.session.insert('x');
    e.wait(|e| {
        lines(e, IDS[0], "reactions")
            .iter()
            .any(|v| v["error"]["data"]["kind"] == "limit_exceeded")
    });
    let observed = events(&e, IDS[0]);
    assert_eq!(
        observed
            .iter()
            .map(|v| v["origin"]["depth"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 2, 4, 6, 8]
    );
    let calls = lines(&e, IDS[1], "calls");
    assert_eq!(
        calls
            .iter()
            .map(|v| v["params"]["context"]["origin"]["depth"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 3, 5, 7]
    );
    assert!(calls.iter().all(
        |v| v["params"]["context"]["origin"]["operation"] == observed[0]["origin"]["operation"]
    ));
}

// A burst larger than the native metadata queue invalidates the whole stream.
// Resnapshot is explicit even for a changed-only subscriber, then re-subscribe
// establishes a new baseline instead of silently continuing incomplete deltas.
#[test]
fn resnapshot() {
    let mut rpc = rpc("non-modal");
    let sub = subscribe(&mut rpc, &["changed"], vec![]);
    rpc.editor.session.insert('a');
    rpc.editor.wait(|e| events(e, IDS[0]).len() == 1);
    let old = events(&rpc.editor, IDS[0])[0].clone();
    for _ in 0..300 {
        rpc.editor.session.insert('x');
    }
    rpc.editor
        .wait(|e| events(e, IDS[0]).iter().any(|v| v["kind"] == "resnapshot"));
    let invalidated = events(&rpc.editor, IDS[0]);
    assert_eq!(invalidated.last().unwrap()["kind"], "resnapshot");
    assert!(invalidated.last().unwrap()["stamp"].is_null());
    let stamp = source(&mut rpc)["stamp"].clone();
    rpc.error("edits.apply", edit(&old, &stamp), "cancelled");
    rpc.ok(
        "events.unsubscribe",
        json!({"subscription":sub["subscription"]}),
    );
    subscribe(&mut rpc, &["changed"], vec![]);
    let before = events(&rpc.editor, IDS[0]).len();
    rpc.editor.session.insert('r');
    rpc.editor.wait(|e| events(e, IDS[0]).len() == before + 1);
    assert_eq!(
        events(&rpc.editor, IDS[0]).last().unwrap()["kind"],
        "changed"
    );
}

// Paused real pipes retain finite frames. Eight consumers exceed the aggregate
// budget before any package ceiling; ordinary input and fair native RPC still
// progress. Once resumed, each affected stream receives explicit invalidation.
#[test]
fn slow_consumers() {
    for count in [1, 8] {
        let mut e = Editor::events("non-modal", count);
        for id in &IDS[..count] {
            wire(
                &mut e,
                id,
                0,
                "events.subscribe",
                json!({"kinds":["changed"],"buffers":[]}),
            );
            fs::write(e.fixture.0.join(id).join("pause"), "").unwrap();
        }
        e.session.insert('p');
        e.wait(|e| {
            IDS[..count]
                .iter()
                .all(|id| e.fixture.0.join(id).join("paused").exists())
        });
        for _ in 0..1024 {
            e.session.handle(crossterm::event::Event::Paste("x".into()));
            // Drain native work, while the actual child pipe stays paused.
            for _ in 0..12 {
                e.session.expire_input(Instant::now());
            }
        }
        assert_eq!(
            e.session.buffer().unwrap().len_chars(),
            "source 🦀\n".chars().count() + 1025
        );
        for id in &IDS[..count] {
            fs::remove_file(e.fixture.0.join(id).join("pause")).unwrap();
        }
        e.wait(|e| {
            IDS[..count]
                .iter()
                .any(|id| events(e, id).iter().any(|v| v["kind"] == "resnapshot"))
        });
        for id in &IDS[..count] {
            let result = wire(
                &mut e,
                id,
                1,
                "buffers.list",
                json!({"page":{"cursor":null,"limit":64},"kind":"text"}),
            );
            assert!(!result["buffers"].as_array().unwrap().is_empty());
            assert_eq!(e.session.plugin_status(id).unwrap().state, State::Active);
        }
        assert_eq!(
            e.session.buffer().unwrap().len_chars(),
            "source 🦀\n".chars().count() + 1025
        );
    }
}

// Unsubscribe revokes a command already delivered to the child. Its late edit
// is refused and the waiting RPC receives cancellation without a 60-second leak.
#[test]
fn late_callback() {
    let mut rpc = rpc("non-modal");
    let sub = subscribe(&mut rpc, &["changed"], vec![]);
    rpc.editor.session.insert('a');
    rpc.editor.wait(|e| events(e, IDS[0]).len() == 1);
    let event = events(&rpc.editor, IDS[0])[0].clone();
    fs::write(rpc.editor.path("hold-command"), "").unwrap();
    let pending = rpc.enqueue("commands.invoke", json!({"command":"field.notes.collect","arguments":{},"context":{
        "invocation":format!("event:{}",event["sequence"].as_str().unwrap()),"generation":event["generation"],"origin":event["origin"],
        "stamp":event["stamp"],"pane":null,"point":null,"selection":null,"prefix":null}}));
    rpc.editor.wait(|e| lines(e, IDS[0], "calls").len() == 1);
    rpc.ok(
        "events.unsubscribe",
        json!({"subscription":sub["subscription"]}),
    );
    fs::write(rpc.editor.path("release-command"), "").unwrap();
    rpc.editor.wait(|e| {
        e.path(&format!("response-{pending}")).exists()
            && lines(e, IDS[0], "reactions")
                .iter()
                .any(|v| v["error"]["data"]["kind"] == "cancelled")
    });
    let response: Value =
        serde_json::from_slice(&fs::read(rpc.editor.path(&format!("response-{pending}"))).unwrap())
            .unwrap();
    assert_eq!(response["error"]["data"]["kind"], "cancelled");
    rpc.editor.wait(|e| e.status().pending == 0);
    assert_eq!(
        rpc.editor
            .session
            .buffer()
            .unwrap()
            .stamp()
            .revision
            .to_string(),
        event["stamp"]["revision"]
    );
}

// A connection that cannot deliver its invalidation retires on the real pipe
// deadline. Source text survives the owned child cleanup.
#[test]
fn invalidation_timeout() {
    let mut e = Editor::events("non-modal", 1);
    wire(
        &mut e,
        IDS[0],
        0,
        "events.subscribe",
        json!({"kinds":["changed"],"buffers":[]}),
    );
    fs::write(e.path("pause"), "").unwrap();
    e.session.insert('p');
    e.wait(|e| e.path("paused").exists());
    for _ in 0..1024 {
        e.session.insert('x');
        for _ in 0..4 {
            e.session.expire_input(Instant::now());
        }
    }
    e.wait(|e| e.status().state == State::Failed);
    assert!(
        e.status()
            .diagnostic
            .contains("event invalidation/delivery deadline"),
        "{}",
        e.status().diagnostic
    );
    assert_eq!(
        e.session.buffer().unwrap().len_chars(),
        "source 🦀\n".chars().count() + 1025
    );
}

// An old event cannot authorize commands or edits in a replacement generation.
// The new process has no implicit subscriptions, and settled idle owns no poll.
#[test]
fn reconnect() {
    let mut rpc = rpc("non-modal");
    let sub = subscribe(&mut rpc, &["changed"], vec![]);
    rpc.editor.session.insert('a');
    rpc.editor.wait(|e| events(e, IDS[0]).len() == 1);
    let old = events(&rpc.editor, IDS[0])[0].clone();
    let old_generation = rpc.editor.status().generation;
    // Clear scripted requests before starting the replacement process.
    for entry in fs::read_dir(rpc.editor.fixture.0.join(IDS[0])).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_string_lossy().starts_with("request-")
            || entry.file_name().to_string_lossy().starts_with("response-")
        {
            fs::remove_file(entry.path()).unwrap();
        }
    }
    rpc.editor.manage(PluginCommand::Restart);
    rpc.editor
        .wait(|e| e.status().state == State::Active && e.status().generation != old_generation);
    let mut rpc = Rpc::from_editor(rpc.editor);
    let stamp = source(&mut rpc)["stamp"].clone();
    rpc.error("edits.apply", edit(&old, &stamp), "unsupported_invocation");
    rpc.error(
        "events.unsubscribe",
        json!({"subscription":sub["subscription"]}),
        "invalid_arguments",
    );
    subscribe(&mut rpc, &["changed"], vec![]);
    rpc.editor.session.insert('b');
    rpc.editor.wait(|e| events(e, IDS[0]).len() == 2);
    let event = events(&rpc.editor, IDS[0])[1].clone();
    assert_ne!(event["generation"], old["generation"]);
    rpc.editor.wait(|e| {
        e.status().pending == 0 && e.session.remaining_input(Instant::now()) != Some(Duration::ZERO)
    });
}
