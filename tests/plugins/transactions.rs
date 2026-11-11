use super::buffers::{Rpc, create, source};
use rev::command::{Command, History, Motion, RegionCommand, Split};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn replacement(start: usize, end: usize, text: &str) -> Value {
    json!({"range":{"start":start,"end":end},"text":text})
}
fn target(buffer: &Value, replacements: &[Value]) -> Value {
    json!({"stamp":buffer["stamp"],"replacements":replacements})
}
fn batch(targets: &[Value]) -> Value {
    json!({"operation":"collect-1","targets":targets,"origin":{"package":"field.notes","operation":"collect-1","depth":0}})
}
fn describe(rpc: &mut Rpc, buffer: &Value) -> Value {
    rpc.ok(
        "buffers.describe",
        &json!({"buffer":buffer["stamp"]["buffer"]}),
    )
}
fn read(rpc: &mut Rpc, buffer: &Value) -> String {
    let current = describe(rpc, buffer);
    rpc.ok("buffers.read", &json!({"buffer":current["stamp"]["buffer"],"range":{"start":0,"end":current["scalar_len"]},"expected_revision":current["stamp"]["revision"]}))["text"].as_str().unwrap().to_owned()
}
fn current_text(rpc: &Rpc) -> String {
    let b = rpc.editor.session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// A real child publishes several replacements in two ordinary sources in both
// profiles. The hidden participant stays hidden and undo/redo remain local.
#[test]
fn local_history() {
    for profile in ["non-modal", "modal"] {
        let mut rpc = Rpc::new(profile);
        let a = source(&mut rpc);
        let b = create(&mut rpc, "Hidden target", "hidden e\u{301}\n", false);
        let selected = rpc.editor.session.windows().selected_id();
        rpc.editor
            .session
            .execute(Command::Motion(Motion::BufferEnd));
        let result = rpc.ok(
            "edits.apply",
            &batch(&[
                target(&a, &[replacement(0, 6, "first"), replacement(7, 8, "🌱")]),
                target(&b, &[replacement(0, 6, "second"), replacement(7, 9, "é")]),
            ]),
        );
        assert_eq!(result["operation"], "collect-1");
        assert_eq!(result["stamps"].as_array().unwrap().len(), 2);
        assert_eq!(rpc.editor.session.windows().selected_id(), selected);
        assert_eq!(current_text(&rpc), "first 🌱\n");
        assert_eq!(read(&mut rpc, &b), "second é\n");
        assert_eq!(rpc.editor.session.view().point.position.0, 8);
        let buffer = rpc.editor.session.buffer().unwrap();
        assert_eq!(buffer.history().len(), 1);
        assert_eq!(buffer.history()[0].changes().len(), 2);
        assert_eq!(
            buffer.stamp().revision.to_string(),
            result["stamps"][0]["revision"]
        );
        rpc.editor.session.execute(Command::History(History::Undo));
        assert_eq!(current_text(&rpc), "source 🦀\n");
        assert_eq!(read(&mut rpc, &b), "second é\n");
        assert!(!rpc.editor.session.buffer().unwrap().is_modified());
        rpc.editor.session.execute(Command::History(History::Redo));
        assert_eq!(current_text(&rpc), "first 🌱\n");
        rpc.switch("Hidden target");
        assert_eq!(rpc.editor.session.buffer().unwrap().history().len(), 1);
        rpc.editor.session.execute(Command::History(History::Undo));
        assert_eq!(current_text(&rpc), "hidden e\u{301}\n");
        assert_eq!(read(&mut rpc, &a), "first 🌱\n");
    }
}

// Each invalid second target refuses the complete batch. Source stamps, saved
// state, history, selection and every live view of the valid first source stay put.
#[test]
fn refused_targets() {
    let mut rpc = Rpc::new("non-modal");
    let original = source(&mut rpc);
    rpc.editor.session.insert('x');
    rpc.editor.session.execute(Command::History(History::Undo));
    let a = describe(&mut rpc, &original);
    let b = create(&mut rpc, "Other target", "second", false);
    let readonly = create(&mut rpc, "Readonly target", "protected", true);
    rpc.editor
        .session
        .execute(Command::Region(RegionCommand::Set));
    rpc.editor
        .session
        .execute(Command::Motion(Motion::ForwardChar));
    let before = rpc.editor.session.view().clone();
    let mark = rpc.editor.session.buffer().unwrap().mark();
    let state = rpc.editor.session.buffer().unwrap().state();
    let mut stale = b.clone();
    stale["stamp"]["revision"] = "0".into();
    let mut foreign = b.clone();
    foreign["stamp"]["buffer"]["session"] = "foreign-host".into();
    let mut missing = b.clone();
    missing["stamp"]["buffer"]["id"] = "buffer-0".into();
    let good = target(&a, &[replacement(0, 6, "must not publish")]);
    let cases = [
        (
            "readonly",
            target(&readonly, &[replacement(0, 1, "x")]),
            "read_only",
        ),
        (
            "stale",
            target(&stale, &[replacement(0, 1, "x")]),
            "stale_revision",
        ),
        (
            "foreign",
            target(&foreign, &[replacement(0, 1, "x")]),
            "missing_buffer",
        ),
        (
            "missing",
            target(&missing, &[replacement(0, 1, "x")]),
            "missing_buffer",
        ),
        (
            "range",
            target(&b, &[replacement(0, 999, "x")]),
            "invalid_range",
        ),
        (
            "overlap",
            target(&b, &[replacement(0, 3, "x"), replacement(2, 4, "y")]),
            "edit_conflict",
        ),
        (
            "equal-start",
            target(&b, &[replacement(1, 1, "x"), replacement(1, 1, "y")]),
            "edit_conflict",
        ),
        ("empty", target(&b, &[]), "invalid_arguments"),
    ];
    for (name, bad, kind) in cases {
        rpc.error("edits.apply", &batch(&[good.clone(), bad]), kind);
        let buffer = rpc.editor.session.buffer().unwrap();
        assert_eq!(
            buffer.stamp().revision.to_string(),
            a["stamp"]["revision"],
            "{name}"
        );
        assert_eq!(buffer.state(), state, "{name}");
        assert_eq!(buffer.mark(), mark, "{name}");
        assert!(buffer.history().is_empty(), "{name}");
        assert_eq!(buffer.redo_history().len(), 1, "{name}");
        assert_eq!(rpc.editor.session.view().point, before.point, "{name}");
        assert_eq!(rpc.editor.session.view().top, before.top, "{name}");
        assert_eq!(read(&mut rpc, &b), "second", "{name}");
        assert_eq!(describe(&mut rpc, &b), b, "{name}");
        assert_eq!(describe(&mut rpc, &readonly), readonly, "{name}");
    }
    rpc.error(
        "edits.apply",
        &batch(&[good.clone(), good]),
        "edit_conflict",
    );
    assert_eq!(current_text(&rpc), "source 🦀\n");
    rpc.editor.session.execute(Command::History(History::Redo));
    assert_eq!(current_text(&rpc), "xsource 🦀\n");
}

// Shared native view points and marks map through each base-coordinate edit.
// A later focus change leaves the explicit source targets and selected scratch intact.
#[test]
fn shared_views() {
    let mut rpc = Rpc::new("non-modal");
    let a = source(&mut rpc);
    let b = create(&mut rpc, "Hidden target", "second", false);
    let first = rpc.editor.session.windows().selected_id();
    rpc.editor
        .session
        .execute(Command::Motion(Motion::ForwardChar));
    rpc.editor
        .session
        .execute(Command::Region(RegionCommand::Set));
    rpc.editor
        .session
        .execute(Command::Motion(Motion::ForwardChar));
    rpc.editor.session.execute(Command::Split(Split::Right));
    let second = rpc
        .editor
        .session
        .windows()
        .iter()
        .map(|(id, _)| id)
        .find(|id| *id != first)
        .unwrap();
    rpc.editor.session.select_view(second).unwrap();
    rpc.editor
        .session
        .execute(Command::Motion(Motion::BufferEnd));
    rpc.editor.session.execute(Command::Split(Split::Below));
    let third = rpc
        .editor
        .session
        .windows()
        .iter()
        .map(|(id, _)| id)
        .find(|id| *id != first && *id != second)
        .unwrap();
    rpc.editor.session.select_view(third).unwrap();
    rpc.switch("*scratch*");
    let scratch = rpc.editor.session.buffer().unwrap().stamp();
    rpc.ok(
        "edits.apply",
        &batch(&[
            target(&a, &[replacement(0, 0, "+")]),
            target(&b, &[replacement(0, 0, "#")]),
        ]),
    );
    assert_eq!(rpc.editor.session.buffer().unwrap().stamp(), scratch);
    assert_eq!(rpc.editor.session.windows().selected_id(), third);
    for (id, position) in [(first, 3), (second, 10)] {
        let view = rpc.editor.session.windows().get(id).unwrap();
        assert_eq!(view.point.position.0, position);
        assert_eq!(view.point.stamp.revision, 2);
    }
    rpc.editor.session.select_view(first).unwrap();
    assert_eq!(rpc.editor.session.buffer().unwrap().mark().unwrap().0, 2);
    assert_eq!(read(&mut rpc, &b), "#second");
}

// Both failure and success keep visiting files untouched. Native save remains
// an explicit later action rather than a side effect of transaction publication.
#[test]
fn no_implicit_save() {
    let mut rpc = Rpc::new("non-modal");
    let path = rpc.editor.fixture.write("visited.txt", "disk text\n");
    rpc.editor.session.load(&path).unwrap();
    let a = source(&mut rpc);
    let b = create(&mut rpc, "Result", "result", false);
    rpc.ok(
        "edits.apply",
        &batch(&[
            target(&a, &[replacement(0, 4, "memory")]),
            target(&b, &[replacement(0, 6, "edited")]),
        ]),
    );
    assert_eq!(current_text(&rpc), "memory text\n");
    assert_eq!(std::fs::read_to_string(path).unwrap(), "disk text\n");
    assert!(rpc.editor.session.buffer().unwrap().is_modified());
    assert_eq!(describe(&mut rpc, &a)["expected_disk"], a["expected_disk"]);
}

// Aggregate replacement bytes and target count are enforced together. The
// admitted maximum produces exactly one independent undo unit in each source.
#[test]
fn finite_batches() {
    let mut rpc = Rpc::new("non-modal");
    let a = source(&mut rpc);
    let mut params = batch(&[target(&a, &[replacement(0, 0, "x")])]);
    params["origin"]["package"] = "other.notes".into();
    rpc.error("edits.apply", &params, "namespace_conflict");
    rpc.error("edits.apply", &batch(&[]), "limit_exceeded");
    let oversized = "x".repeat(256 * 1024 + 1);
    rpc.error(
        "edits.apply",
        &batch(&[target(
            &a,
            &[replacement(0, 0, &oversized), replacement(1, 1, &oversized)],
        )]),
        "limit_exceeded",
    );
    let mut buffers = vec![a];
    for n in 1..16 {
        buffers.push(create(&mut rpc, &format!("Target {n}"), "", false));
    }
    let payload = "x".repeat(32 * 1024);
    let targets: Vec<_> = buffers
        .iter()
        .map(|b| target(b, &[replacement(0, 0, &payload)]))
        .collect();
    let result = rpc.ok("edits.apply", &batch(&targets));
    assert_eq!(result["stamps"].as_array().unwrap().len(), 16);
    for b in &buffers {
        assert_eq!(describe(&mut rpc, b)["stamp"]["revision"], "2");
    }
    assert_eq!(rpc.editor.session.buffer().unwrap().history().len(), 1);
}

// The real supervisor may enqueue work before a native pass. Retirement must
// release that work without publishing text, history, views or a successful reply.
#[test]
fn retired_batch() {
    let mut rpc = Rpc::new("non-modal");
    let a = source(&mut rpc);
    let point = rpc.editor.session.view().point;
    let sequence = rpc.enqueue(
        "edits.apply",
        &batch(&[target(&a, &[replacement(0, 0, "late")])]),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while rpc.editor.status().pending == 0 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    rpc.editor.session.shutdown_plugins().unwrap();
    assert_eq!(current_text(&rpc), "source 🦀\n");
    assert_eq!(rpc.editor.session.view().point, point);
    assert!(rpc.editor.session.buffer().unwrap().history().is_empty());
    assert!(!rpc.editor.path(&format!("response-{sequence}")).exists());
}

// Failed admission preserves an active completion. Successful publication
// invalidates its captured revision and prevents later acceptance from writing.
#[test]
fn completion_retirement() {
    use rev::command::CompletionCommand as C;
    let mut rpc = Rpc::new("non-modal");
    let path = rpc
        .editor
        .fixture
        .write("words.rs", "fn config() {}\nfn cfg_value() {}\ncfg");
    rpc.editor.session.load(&path).unwrap();
    rpc.editor
        .session
        .execute(Command::Motion(Motion::BufferEnd));
    let a = source(&mut rpc);
    let readonly = create(&mut rpc, "Readonly", "protected", true);
    rpc.editor.session.execute(Command::Completion(C::AtPoint));
    rpc.editor.wait(|e| e.session.completion().is_some());
    let end = usize::try_from(a["scalar_len"].as_u64().unwrap()).unwrap();
    let change = target(&a, &[replacement(end - 3, end, "call")]);
    rpc.error(
        "edits.apply",
        &batch(&[change.clone(), target(&readonly, &[replacement(0, 0, "x")])]),
        "read_only",
    );
    assert!(rpc.editor.session.completion().is_some());
    rpc.ok("edits.apply", &batch(&[change]));
    assert!(rpc.editor.session.completion().is_none());
    let text = current_text(&rpc);
    rpc.editor.session.execute(Command::Completion(C::Accept));
    assert_eq!(current_text(&rpc), text);
    assert!(text.ends_with("call"));
}

// Generated inventories and terminals never become text mutation targets.
// Native departure also freezes an otherwise valid captured transaction.
#[test]
fn resources_and_consent() {
    use rev::command::{PluginCommand, TerminalCommand};
    let mut rpc = Rpc::new("non-modal");
    let a = source(&mut rpc);
    rpc.editor
        .session
        .execute(Command::Plugin(PluginCommand::List));
    let generated = rpc.editor.session.buffer().unwrap();
    let mut special = a.clone();
    special["stamp"]["buffer"]["id"] = format!("buffer-{}", generated.id()).into();
    special["stamp"]["revision"] = generated.stamp().revision.to_string().into();
    let good = target(&a, &[replacement(0, 0, "x")]);
    rpc.error(
        "edits.apply",
        &batch(&[good.clone(), target(&special, &[replacement(0, 0, "x")])]),
        "unsupported_resource",
    );
    assert_eq!(describe(&mut rpc, &a), a);
    rpc.editor
        .session
        .execute(Command::Terminal(TerminalCommand::NewBuffer));
    rpc.editor.wait(|e| e.session.terminal_buffer().is_some());
    let terminal = rpc.editor.session.terminal_buffer().unwrap();
    special["stamp"]["buffer"]["id"] = format!("buffer-{}", terminal.id()).into();
    special["stamp"]["revision"] = terminal.stamp().revision.to_string().into();
    rpc.error(
        "edits.apply",
        &batch(&[good, target(&special, &[replacement(0, 0, "x")])]),
        "unsupported_resource",
    );
    assert_eq!(describe(&mut rpc, &a), a);
    rpc.switch(a["name"].as_str().unwrap());
    rpc.editor.session.insert('!');
    let a = describe(&mut rpc, &a);
    rpc.editor.session.execute(Command::Quit);
    let question = rpc.editor.session.echo().to_owned();
    assert!(question.contains("y=save"), "{question}");
    rpc.error(
        "edits.apply",
        &batch(&[target(&a, &[replacement(0, 0, "late")])]),
        "busy",
    );
    assert_eq!(rpc.editor.session.echo(), question);
    assert_eq!(describe(&mut rpc, &a), a);
    rpc.editor.session.execute(Command::KeyboardQuit);
}
