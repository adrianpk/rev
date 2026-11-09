use super::{
    buffers::{Rpc, create, source},
    lifecycle::Editor,
};
use crossterm::event::{Event, KeyEvent};
use rev::{
    command::{Command, Motion, PluginCommand},
    input::keymap::Sequence,
    plugins::lifecycle::State,
};
use serde_json::{Value, json};
use std::fs;
fn binding(id: &str, keys: &str, context: &str, scope: &str, selector: &str) -> String {
    format!(
        "\n[[bindings]]\ncommand='{id}.collect'\nkeys='{keys}'\ncontext='{context}'\nscope='{scope}'\n{selector}\n"
    )
}
fn press(editor: &mut Editor, keys: &str) {
    if keys == "Escape" {
        editor.session.handle(Event::Key(KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        )));
        return;
    }
    for key in Sequence::parse(keys).unwrap().chords() {
        editor
            .session
            .handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
    }
}
fn calls(editor: &Editor, package: &str) -> usize {
    fs::read_to_string(editor.fixture.0.join(package).join("calls"))
        .unwrap_or_default()
        .lines()
        .count()
}
fn text(editor: &Editor) -> String {
    let b = editor.session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn bind(rpc: &mut Rpc, key: &str, context: &str, scope: Value) -> Value {
    rpc.ok(
        "keymaps.bind",
        json!({"command":"field.notes.collect","keys":key,"context":context,"scope":scope}),
    )
}
fn pump(rpc: &mut Rpc) {
    rpc.list();
}
fn other(rpc: &mut Rpc, seq: usize, method: &str, params: Value) -> Value {
    let root = rpc.editor.fixture.0.join("other.notes");
    fs::write(
        root.join(format!("request-{seq}")),
        format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":format!("other-{seq}"),"method":method,"params":params})
        ),
    )
    .unwrap();
    rpc.editor
        .wait(|_| root.join(format!("response-{seq}")).exists());
    serde_json::from_slice::<Value>(&fs::read(root.join(format!("response-{seq}"))).unwrap())
        .unwrap()
}

// Passive bindings activate lazily in their declared native profile; prompts,
// modal transitions and palette hints retain their actual resolver ownership.
#[test]
fn passive_profiles() {
    for (profile, context) in [("non-modal", "editor"), ("modal", "vim_normal")] {
        let defaults = binding("field.notes", "C-c n", context, "global", "");
        let mut e = Editor::keymaps(profile, &[("field.notes", &defaults)], "", false);
        assert_eq!(e.status().state, State::Installed);
        e.session.execute(Command::ExecuteExtended);
        e.session.handle(Event::Paste("field.notes.collect".into()));
        let frame = e.session.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .any(|r| r.text().contains("field.notes.collect") && r.text().contains("C-c n")),
            "{}",
            frame
                .rows()
                .iter()
                .map(|r| r.text())
                .collect::<Vec<_>>()
                .join("\n")
        );
        e.session.execute(Command::KeyboardQuit);
        e.session.execute(Command::FindFile);
        press(&mut e, "C-c n");
        assert_eq!(e.status().state, State::Installed);
        e.session.execute(Command::KeyboardQuit);
        press(&mut e, "C-c n");
        e.wait(|e| calls(e, "field.notes") == 1 && e.status().pending == 0);
        assert_eq!(text(&e), "source 🦀\n");
        if profile == "modal" {
            press(&mut e, "i");
            press(&mut e, "x");
            press(&mut e, "Escape");
            assert!(text(&e).starts_with('x'));
        }
    }
}

// Within global scope, explicit native user configuration precedes defaults.
// Competing plugin defaults have no order-dependent winner or active hint.
#[test]
fn global_precedence() {
    for reverse in [false, true] {
        let a = binding("field.notes", "C-e", "editor", "global", "");
        let b = binding("other.notes", "C-e", "editor", "global", "");
        let specs = if reverse {
            vec![("other.notes", b.as_str()), ("field.notes", a.as_str())]
        } else {
            vec![("field.notes", a.as_str()), ("other.notes", b.as_str())]
        };
        let mut e = Editor::keymaps("non-modal", &specs, "", false);
        press(&mut e, "C-e");
        assert_eq!(calls(&e, "field.notes") + calls(&e, "other.notes"), 0);
        assert_eq!(e.session.view().point.position.0, 8);
        e.session.execute(Command::ExecuteExtended);
        e.session.handle(Event::Paste("field.notes.collect".into()));
        assert!(
            !e.session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("field.notes.collect") && r.text().contains("C-e"))
        );
        e.session.execute(Command::KeyboardQuit);
        e.session.execute(Command::Plugin(PluginCommand::List));
        assert_eq!(text(&e).matches("Conflicting").count(), 2);
        assert!(
            text(&e).contains("field.notes.collect") && text(&e).contains("other.notes.collect")
        );
    }
    let a = binding("field.notes", "C-e", "editor", "global", "");
    let mut e = Editor::keymaps(
        "non-modal",
        &[("field.notes", &a)],
        "[keys.editor]\n'C-e'='beginning-of-buffer'",
        false,
    );
    e.session.execute(Command::Motion(Motion::BufferEnd));
    press(&mut e, "C-e");
    assert_eq!(e.session.view().point.position.0, 0);
    assert_eq!(e.status().state, State::Installed);
    e.session.execute(Command::Plugin(PluginCommand::List));
    assert!(text(&e).contains("Shadowed"));
}

// Exact buffer ownership is local scope: it precedes customized globals but
// never follows focus into another source. Unbind reveals the current native map.
#[test]
fn local_scope() {
    for (profile, context) in [("non-modal", "editor"), ("modal", "vim_normal")] {
        let config = format!("[keys.{context}]\n'C-e'='beginning-of-buffer'");
        let e = Editor::keymaps(profile, &[("field.notes", "")], &config, true);
        let mut rpc = Rpc::from_editor(e);
        let original = source(&mut rpc);
        let result = create(&mut rpc, "Bound result", "result", false);
        let local = bind(
            &mut rpc,
            "C-e",
            context,
            json!({"kind":"buffer","buffer":result["stamp"]["buffer"]}),
        );
        assert_eq!(local["status"], "effective");
        assert_eq!(local["underlying"][0]["provenance"], "user");
        rpc.editor
            .session
            .execute(Command::Motion(Motion::BufferEnd));
        press(&mut rpc.editor, "C-e");
        assert_eq!(rpc.editor.session.view().point.position.0, 0);
        assert_eq!(calls(&rpc.editor, "field.notes"), 0);
        rpc.switch("Bound result");
        press(&mut rpc.editor, "C-e");
        pump(&mut rpc);
        assert_eq!(calls(&rpc.editor, "field.notes"), 1);
        rpc.ok(
            "keymaps.unbind",
            json!({"registrations":[local["registration"]]}),
        );
        rpc.editor
            .session
            .execute(Command::Motion(Motion::BufferEnd));
        press(&mut rpc.editor, "C-e");
        assert_eq!(rpc.editor.session.view().point.position.0, 0);
        assert_eq!(calls(&rpc.editor, "field.notes"), 1);
        rpc.ok(
            "buffers.describe",
            json!({"buffer":original["stamp"]["buffer"]}),
        );
    }
}

// Explicit persistent selectors intersect filetype and package provenance. A
// user choice in that local scope resolves competing package defaults.
#[test]
fn user_selectors() {
    let a = binding(
        "field.notes",
        "F8",
        "editor",
        "buffer",
        "buffer_owner='field.notes'",
    );
    let b = binding(
        "other.notes",
        "F8",
        "editor",
        "buffer",
        "buffer_owner='field.notes'",
    );
    let config = "[[plugins.keybindings]]\ncommand='field.notes.collect'\nkeys='F8'\ncontext='editor'\nscope='buffer'\nfiletypes=['rust']\nbuffer_owner='field.notes'";
    let e = Editor::keymaps(
        "non-modal",
        &[("field.notes", &a), ("other.notes", &b)],
        config,
        true,
    );
    let mut rpc = Rpc::from_editor(e);
    let result = create(&mut rpc, "Typed result", "fn main() {}\n", false);
    rpc.switch("Typed result");
    press(&mut rpc.editor, "F8");
    pump(&mut rpc);
    assert_eq!(calls(&rpc.editor, "field.notes"), 0);
    let path = rpc.editor.fixture.0.join("result.rs");
    rpc.ok("buffers.save",json!({"buffer":result["stamp"]["buffer"],"path":path,"expected_revision":result["stamp"]["revision"],"expected_disk":result["expected_disk"]}));
    press(&mut rpc.editor, "F8");
    pump(&mut rpc);
    assert_eq!(calls(&rpc.editor, "field.notes"), 1);
    assert_eq!(calls(&rpc.editor, "other.notes"), 0);
    let raw = rpc.editor.fixture.write("unowned.rs", "fn raw() {}\n");
    let opened = rpc.ok("buffers.open", json!({"path":raw}));
    rpc.switch(opened["name"].as_str().unwrap());
    press(&mut rpc.editor, "F8");
    pump(&mut rpc);
    assert_eq!(calls(&rpc.editor, "field.notes"), 1);
}

// Normalization and native prefix/recovery checks happen before ownership is
// installed. Foreign handles/commands and atomic unbind failures retain mappings.
#[test]
fn admission_and_retirement() {
    let e = Editor::keymaps("modal", &[("field.notes", "")], "", true);
    let mut rpc = Rpc::from_editor(e);
    let source = source(&mut rpc);
    for (keys, context) in [
        ("C-g", "vim_normal"),
        ("C-c C-g", "editor"),
        ("M-x", "editor"),
        (":", "vim_normal"),
        ("i", "vim_normal"),
        ("2", "vim_normal"),
        ("C-f x", "editor"),
        ("C-x", "editor"),
        ("M-1", "editor"),
        ("Escape", "vim_insert"),
    ] {
        rpc.error("keymaps.bind",json!({"command":"field.notes.collect","keys":keys,"context":context,"scope":{"kind":"global"}}),"invalid_arguments");
    }
    let mut foreign = source["stamp"]["buffer"].clone();
    foreign["session"] = json!("foreign");
    rpc.error("keymaps.bind",json!({"command":"field.notes.collect","keys":"F8","context":"vim_normal","scope":{"kind":"buffer","buffer":foreign}}),"missing_buffer");
    rpc.error("keymaps.bind",json!({"command":"other.notes.collect","keys":"F8","context":"vim_normal","scope":{"kind":"global"}}),"namespace_conflict");
    let bound = bind(&mut rpc, "F8", "vim_normal", json!({"kind":"global"}));
    rpc.error(
        "keymaps.unbind",
        json!({"registrations":[bound["registration"],"missing"]}),
        "invalid_arguments",
    );
    press(&mut rpc.editor, "F8");
    pump(&mut rpc);
    assert_eq!(calls(&rpc.editor, "field.notes"), 1);
    rpc.editor.manage(PluginCommand::Deactivate);
    rpc.editor.wait(|e| e.status().state == State::Inactive);
    press(&mut rpc.editor, "F8");
    assert_eq!(calls(&rpc.editor, "field.notes"), 1);
    assert_eq!(rpc.editor.status().state, State::Inactive);
}

// Arrival order cannot resolve equal-priority runtime defaults. Removing a
// competitor restores the surviving contribution; a foreign unbind cannot do so.
#[test]
fn dynamic_conflicts() {
    for reverse in [false, true] {
        let e = Editor::keymaps(
            "non-modal",
            &[("field.notes", ""), ("other.notes", "")],
            "",
            true,
        );
        let mut rpc = Rpc::from_editor(e);
        let params = json!({"command":"other.notes.collect","keys":"F8","context":"editor","scope":{"kind":"global"}});
        let (mine, theirs) = if reverse {
            let theirs = other(&mut rpc, 0, "keymaps.bind", params);
            (
                bind(&mut rpc, "F8", "editor", json!({"kind":"global"})),
                theirs,
            )
        } else {
            let mine = bind(&mut rpc, "F8", "editor", json!({"kind":"global"}));
            (mine, other(&mut rpc, 0, "keymaps.bind", params))
        };
        assert_eq!(
            if reverse {
                &mine["status"]
            } else {
                &theirs["result"]["status"]
            },
            "conflicting"
        );
        press(&mut rpc.editor, "F8");
        pump(&mut rpc);
        assert_eq!(
            calls(&rpc.editor, "field.notes") + calls(&rpc.editor, "other.notes"),
            0
        );
        rpc.error(
            "keymaps.unbind",
            json!({"registrations":[theirs["result"]["registration"]]}),
            "namespace_conflict",
        );
        let response = other(
            &mut rpc,
            1,
            "keymaps.unbind",
            json!({"registrations":[theirs["result"]["registration"]]}),
        );
        assert!(response.get("error").is_none(), "{response}");
        press(&mut rpc.editor, "F8");
        pump(&mut rpc);
        assert_eq!(calls(&rpc.editor, "field.notes"), 1);
    }
}

// A conflicted local layer reveals the current global policy. Normalized Meta
// aliases conflict identically and cannot activate an arrival-order winner.
#[test]
fn fallback_and_aliases() {
    let a = binding("field.notes", "F8", "editor", "global", "")
        + &binding(
            "field.notes",
            "F8",
            "editor",
            "buffer",
            "buffer_owner='field.notes'",
        );
    let b = binding(
        "other.notes",
        "F8",
        "editor",
        "buffer",
        "buffer_owner='field.notes'",
    );
    let e = Editor::keymaps(
        "non-modal",
        &[("field.notes", &a), ("other.notes", &b)],
        "",
        true,
    );
    let mut rpc = Rpc::from_editor(e);
    create(&mut rpc, "Local fallback", "kept", false);
    rpc.switch("Local fallback");
    press(&mut rpc.editor, "F8");
    pump(&mut rpc);
    assert_eq!(calls(&rpc.editor, "field.notes"), 1);
    assert_eq!(calls(&rpc.editor, "other.notes"), 0);
    rpc.editor
        .session
        .execute(Command::Plugin(PluginCommand::List));
    assert_eq!(text(&rpc.editor).matches("Conflicting").count(), 2);
    assert_eq!(text(&rpc.editor).matches("Effective").count(), 1);
    let a = binding("field.notes", "M-n", "editor", "global", "");
    let b = binding("other.notes", "ESC n", "editor", "global", "");
    let mut e = Editor::keymaps(
        "non-modal",
        &[("field.notes", &a), ("other.notes", &b)],
        "",
        false,
    );
    press(&mut e, "M-n");
    assert_eq!(calls(&e, "field.notes") + calls(&e, "other.notes"), 0);
    e.session.execute(Command::Plugin(PluginCommand::List));
    assert_eq!(text(&e).matches("Conflicting").count(), 2);
}

// Registration identity and finite admission survive duplicate suggestions.
// The maximum unbind batch releases only the caller's current-generation owners.
#[test]
fn finite_registrations() {
    let e = Editor::keymaps("non-modal", &[("field.notes", "")], "", true);
    let mut rpc = Rpc::from_editor(e);
    let mut ids = Vec::new();
    for _ in 0..128 {
        ids.push(bind(&mut rpc, "F8", "editor", json!({"kind":"global"}))["registration"].clone());
    }
    assert_eq!(
        ids.iter()
            .map(|id| id.as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        128
    );
    rpc.error("keymaps.bind",json!({"command":"field.notes.collect","keys":"F8","context":"editor","scope":{"kind":"global"}}),"limit_exceeded");
    rpc.ok("keymaps.unbind", json!({"registrations":ids}));
    assert_eq!(
        bind(&mut rpc, "F8", "editor", json!({"kind":"global"}))["status"],
        "effective"
    );
}

// Passive admission checks the merged native user map and selector intersections
// before publishing any package catalog; invalid candidates cannot start peers.
#[test]
fn atomic_configuration() {
    for (label, key, context, extra, valid) in [
        ("leaf", "C-f x", "editor", "".to_owned(), false),
        ("recovery", "M-x", "editor", "".to_owned(), false),
        ("transition", "i", "vim_normal", "".to_owned(), false),
        ("merged-prefix", "C-c n x", "editor", "[keys.editor]\n'C-c n'='beginning-of-buffer'".to_owned(), false),
        ("user-alias", "F8", "editor", "[[plugins.keybindings]]\ncommand='field.notes.collect'\nkeys='M-n'\ncontext='editor'\nscope='global'\n[[plugins.keybindings]]\ncommand='field.notes.collect'\nkeys='ESC n'\ncontext='editor'\nscope='global'".to_owned(), false),
        ("disjoint-selectors", "F8", "editor", "[[plugins.keybindings]]\ncommand='field.notes.collect'\nkeys='F9'\ncontext='editor'\nscope='buffer'\nfiletypes=['rust']\n[[plugins.keybindings]]\ncommand='field.notes.collect'\nkeys='F9'\ncontext='editor'\nscope='buffer'\nfiletypes=['toml']".to_owned(), true),
    ] {
        let fixture=super::fixture::Fixture::new();
        fixture.write("package.toml", &format!("[plugin]\nid='field.notes'\nversion='0.1.0'\napi='rev.plugins/0.1'\n[process]\nprogram='missing-peer'\n[[commands]]\nname='collect'\ndescription='Collect notes.'\nprefix='none'\nrecording='reject'\n{}", binding("field.notes",key,context,"global","")));
        let path=fixture.write("config.toml", &format!("[plugins]\nenabled=true\nmanifests=['package.toml']\n{extra}"));
        let prepared=rev::config::prepare(&path);
        assert_eq!(prepared.diagnostic.is_none(), valid, "{label}: {:?}",prepared.diagnostic);
        if !valid { assert_eq!(prepared.snapshot.plugins().packages().count(),0,"{label}"); }
    }
}
