use super::*;
use std::{path::Path, process::Command as Process};

fn configured(
    f: &Fixture,
    mode: &str,
    profile: &str,
    definitions: &str,
    selection: bool,
) -> Session {
    drop(editor(f, mode, profile, false));
    let config = f.0.join("config.toml");
    let mut text = fs::read_to_string(&config).unwrap();
    text.push_str(definitions);
    fs::write(&config, text).unwrap();
    reopen(f, selection)
}
fn reopen(f: &Fixture, selection: bool) -> Session {
    let startup = rev::config::prepare(&f.0.join("config.toml"));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.0.join("source.txt")).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut view = View::new(buffer);
    if selection {
        buffer.activate_mark(view.point).unwrap();
        view.point.position = Scalar(10);
    }
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 28,
        },
    )
    .unwrap()
}
fn definition(name: &str, input: &str, context: &str, output: &str) -> String {
    format!(
        "\n[ai.commands.{name}]\nprompt='Keep literal $(not-a-command) and @source.'\ninput='{input}'\ncontext='{context}'\noutput='{output}'\nmodel='another/model'\n"
    )
}
fn argument(s: &mut Session, value: &str) {
    s.handle(Event::Paste(value.into()));
    s.handle(key(KeyCode::Enter));
}
fn complete(s: &mut Session) {
    support::until(|| {
        tick(s);
        assert!(
            s.remaining_input(Instant::now()).is_some() || s.echo().contains("ready"),
            "{}",
            s.echo()
        );
        s.echo().contains("ready")
    });
}
fn stopped(s: &mut Session) {
    support::until(|| {
        tick(s);
        s.remaining_input(Instant::now()).is_none()
    });
}
fn envelope(f: &Fixture) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(f.0.join("prompt")).unwrap()).unwrap()
}

// Saved selection/point/whole-buffer policies capture their own native target
// under both profiles. Their literal prompt/model reaches only fresh input RPC.
#[test]
fn saved_targets() {
    for profile in ["non-modal", "modal"] {
        for (input, output, range) in [
            ("selection", "replace", [0, 10]),
            ("point", "insert", [0, 0]),
            ("buffer", "replace", [0, 21]),
        ] {
            let f = Fixture::new();
            f.write("result", "proposed");
            let mut s = configured(
                &f,
                "normal",
                profile,
                &definition("literal", input, "input", output),
                input == "selection",
            );
            let original = text(&s);
            let source = s.buffer().unwrap().stamp();
            named(&mut s, profile, "ai-literal");
            complete(&mut s);
            let request = envelope(&f);
            assert_eq!(
                request["instruction"],
                "Keep literal $(not-a-command) and @source."
            );
            assert_eq!(request["observations"]["input"], input);
            assert_eq!(
                request["observations"]["range_scalars"],
                serde_json::json!(range)
            );
            assert_eq!(request["dirty_sources"], serde_json::json!([]));
            assert_eq!(request["context"], "input");
            let args: Vec<String> =
                serde_json::from_str(&fs::read_to_string(f.0.join("args")).unwrap()).unwrap();
            assert!(args.iter().any(|x| x == "--no-session"));
            assert!(args.iter().any(|x| x == "--no-tools"));
            assert!(!args.iter().any(|x| x == "--extension"));
            assert!(
                args.windows(2)
                    .any(|pair| pair == ["--model", "another/model"])
            );
            s.execute(Command::AiApplyResult);
            assert_ne!(s.buffer().unwrap().stamp(), source);
            s.execute(Command::History(History::Undo));
            assert_eq!(text(&s), original);
            // Reuse the definition after a new native revision, never its old span.
            let revision = s.buffer().unwrap().stamp().revision;
            if input == "selection" {
                s.execute(Command::AiDefined(
                    rev::command::AiName::new("literal").unwrap(),
                ));
                assert!(s.echo().contains("active selection"));
            } else {
                named(&mut s, profile, "ai-literal");
                complete(&mut s);
                assert_eq!(envelope(&f)["observations"]["revision"], revision);
                s.execute(Command::AiDiscardResult);
            }
        }
    }
}

// A narrow saved request cannot inherit a broader directory child/history.
// Stopping owns the slot; only the next explicit retry starts fresh no-tools RPC.
#[test]
fn narrow_context() {
    let f = Fixture::new();
    let definitions = definition("broad", "buffer", "directory", "explanation")
        + &definition("narrow", "point", "input", "insert");
    let mut s = configured(&f, "idle", "non-modal", &definitions, false);
    named(&mut s, "non-modal", "ai-broad");
    complete(&mut s);
    let old = fs::read_to_string(f.0.join("pid")).unwrap();
    s.execute(Command::AiDiscardResult);
    named(&mut s, "non-modal", "ai-narrow");
    assert!(s.echo().contains("boundary changed"), "{}", s.echo());
    stopped(&mut s);
    assert!(!Path::new("/proc").join(&old).exists());
    named(&mut s, "non-modal", "ai-narrow");
    complete(&mut s);
    assert_ne!(fs::read_to_string(f.0.join("pid")).unwrap(), old);
    let request = envelope(&f);
    assert_eq!(request["context"], "input");
    assert_eq!(request["history"], "fresh input-only");
    assert_eq!(request["dirty_sources"], serde_json::json!([]));
    assert_eq!(request["observations"]["text"], "");
    let args: Vec<String> =
        serde_json::from_str(&fs::read_to_string(f.0.join("args")).unwrap()).unwrap();
    assert!(args.contains(&"--no-session".into()) && args.contains(&"--no-tools".into()));
    assert!(!args.contains(&"--extension".into()));
    s.execute(Command::AiDiscardResult);
    assert_eq!(text(&s), "é original selection\n");
}

// Use an owned subprocess environment for the real settings lock. Saving keeps
// the successful instruction, rejects source races, activates one definition,
// and leaves disk-only keymaps/other definitions for ordinary restart.
#[test]
fn native_save() {
    if std::env::var_os("REV_AI_SAVE_CHILD").is_none() {
        let f = Fixture::new();
        let status = Process::new(std::env::current_exe().unwrap())
            .args(["--exact", "definitions::native_save", "--test-threads=1"])
            .env("REV_AI_SAVE_CHILD", "1")
            .env("XDG_STATE_HOME", f.0.join("state"))
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    for profile in ["non-modal", "modal"] {
        let f = Fixture::new();
        let mut s = configured(
            &f,
            "normal",
            profile,
            "\n# Preserve this user comment.\n",
            false,
        );
        s.execute(Command::AiSaveCommand);
        assert!(s.echo().contains("No successful"));
        named(&mut s, profile, "ai-prompt");
        let instruction = "Keep the original literal instruction.";
        argument(&mut s, instruction);
        complete(&mut s);
        let root = f.0.join("config.toml");
        let original = fs::read(&root).unwrap();
        named(&mut s, profile, "ai-save-command");
        s.execute(Command::KeyboardQuit);
        assert_eq!(fs::read(&root).unwrap(), original);
        named(&mut s, profile, "ai-save-command");
        let mut concurrent = String::from_utf8(original).unwrap();
        concurrent.push_str("# Concurrent change.\n");
        fs::write(&root, &concurrent).unwrap();
        argument(&mut s, "racing");
        assert!(s.echo().contains("conflict"), "{}", s.echo());
        assert_eq!(fs::read_to_string(&root).unwrap(), concurrent);
        named(&mut s, profile, "ai-save-command");
        argument(&mut s, "saved");
        assert!(s.echo().contains("saved and callable"), "{}", s.echo());
        let startup = rev::config::prepare(&root);
        let definition = &startup.snapshot.settings().ai.commands["saved"];
        assert_eq!(definition.prompt, instruction);
        assert_eq!(definition.input, rev::config::AiInput::Point);
        assert!(!definition.prompt.contains("selection explanation"));
        assert!(
            fs::read_to_string(&root)
                .unwrap()
                .contains("# Preserve this user comment.")
        );
        s.execute(Command::AiDiscardResult);
        named(&mut s, profile, "ai-saved");
        complete(&mut s);
        assert_eq!(envelope(&f)["instruction"], instruction);
        s.execute(Command::AiDiscardResult);
        let mut external = fs::read_to_string(&root).unwrap();
        external.push_str(&definition_text_external());
        external
            .push_str("\n[keys.editor]\n'M-j'='ai-saved'\n[keys.vim_normal]\n'g t'='ai-saved'\n");
        fs::write(&root, &external).unwrap();
        named(&mut s, profile, "ai-save-command");
        assert!(
            s.echo().contains("successful literal"),
            "{profile}: before second save: {}, graph: {:?}",
            s.echo(),
            rev::config::prepare(&root).diagnostic
        );
        argument(&mut s, "second");
        assert!(s.echo().contains("saved and callable"), "{}", s.echo());
        // Only the chosen definition activates: another valid disk definition
        // and newly written keymaps cannot silently enter this live session.
        named(&mut s, profile, "ai-external");
        assert!(s.echo().contains("Unknown command"), "{}", s.echo());
        let pid = fs::read_to_string(f.0.join("pid")).unwrap();
        if profile == "modal" {
            s.handle(key(KeyCode::Char('g')));
            s.handle(key(KeyCode::Char('t')));
        } else {
            s.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('j'),
                KeyModifiers::ALT,
            )));
        }
        assert_eq!(fs::read_to_string(f.0.join("pid")).unwrap(), pid);
        assert!(s.remaining_input(Instant::now()).is_none());
        drop(s);
        let mut restarted = reopen(&f, false);
        if profile == "modal" {
            restarted.handle(key(KeyCode::Char('g')));
            restarted.handle(key(KeyCode::Char('t')));
        } else {
            restarted.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('j'),
                KeyModifiers::ALT,
            )));
        }
        complete(&mut restarted);
        assert_eq!(envelope(&f)["instruction"], instruction);
        restarted.execute(Command::AiDiscardResult);
    }
}
fn definition_text_external() -> String {
    definition("external", "point", "input", "insert")
}
