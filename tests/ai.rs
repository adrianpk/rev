#![forbid(unsafe_code)]
#[path = "ai/definitions.rs"]
mod definitions;
#[path = "ai/discovery.rs"]
mod discovery;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "ai/results.rs"]
mod results;
#[path = "ai/sessions.rs"]
mod sessions;
#[path = "ai/support.rs"]
mod support;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fixture::Fixture;
use rev::buffer::{Buffers, edit::History};
use rev::command::{Command, ModalCommand, Motion};
use rev::coordinates::Scalar;
use rev::session::Session;
use rev::view::{Size, View};
use std::fs;
use std::time::Instant;

fn editor(f: &Fixture, mode: &str, profile: &str, selection: bool) -> Session {
    let command = support::peer(f, mode);
    let source = format!(
        "[editor]\ninteraction_profile = '{profile}'\n[ai.pi]\npersist_sessions=false\nidle_timeout_ms={}\ncommand = '{}'\nrequest_timeout_ms = 1000\n",
        if mode == "idle" { 5000 } else { 0 },
        command.display()
    );
    let config = f.write("config.toml", &source);
    let startup = rev::config::prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let path = f.write("source.txt", "é original selection\n");
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    if mode == "readonly" {
        buffer.set_readonly(true);
    }
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
fn text(s: &Session) -> String {
    let b = s.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn tick(s: &mut Session) {
    s.expire_input(Instant::now());
}
fn ready(s: &mut Session) {
    support::until(|| {
        tick(s);
        assert!(
            s.remaining_input(Instant::now()).is_some() || s.echo().contains("explanation ready"),
            "explanation outcome: {}",
            s.echo()
        );
        s.echo().contains("explanation ready")
    });
}
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn named(s: &mut Session, profile: &str, name: &str) {
    if profile == "modal" {
        s.handle(key(KeyCode::Char(':')));
    } else {
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::ALT,
        )));
    }
    s.handle(Event::Paste(name.into()));
    s.handle(key(KeyCode::Enter));
}

// Both native command selectors capture the same Unicode selection and present
// an owned read-only result; history, disk and the source stamp remain intact.
#[test]
fn named_explanation() {
    for profile in ["non-modal", "modal"] {
        let f = Fixture::new();
        let mut s = editor(&f, "normal", profile, true);
        let source = s.buffer().unwrap().stamp();
        assert!(!f.0.join("rpc-started").exists());
        assert!(s.remaining_input(Instant::now()).is_none());
        named(&mut s, profile, "ai-explain");
        ready(&mut s);
        assert!(s.buffer().unwrap().is_readonly());
        assert!(s.buffer().unwrap().is_transient());
        assert!(s.buffer().unwrap().file().is_none());
        assert_eq!(s.buffer().unwrap().history().len(), 0);
        assert!(text(&s).contains("Context: guarded directory"));
        assert!(text(&s).contains("A selection explanation."));
        let result = text(&s);
        s.insert('x');
        assert_eq!(text(&s), result);
        let prompt: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(f.0.join("prompt")).unwrap()).unwrap();
        assert_eq!(prompt["observations"]["text"], "é original");
        assert_eq!(prompt["observations"]["revision"], source.revision);
        assert_eq!(prompt["observations"]["dirty"], false);
        let args: Vec<String> =
            serde_json::from_str(&fs::read_to_string(f.0.join("args")).unwrap()).unwrap();
        for flag in [
            "--no-session",
            "--tools",
            "--extension",
            "--no-extensions",
            "--no-mcp",
            "--no-skills",
            "--no-prompt-templates",
            "--no-context-files",
            "--no-approve",
        ] {
            assert!(args.iter().any(|s| s == flag), "{flag}");
        }
        assert_eq!(
            fs::read_to_string(f.0.join("cwd")).unwrap(),
            f.0.canonicalize().unwrap().to_str().unwrap()
        );
        assert_eq!(
            fs::read_to_string(f.0.join("source.txt")).unwrap(),
            "é original selection\n"
        );
        s.execute(Command::Buffer(rev::command::BufferCommand::Switch));
        s.handle(Event::Paste("source.txt".into()));
        s.handle(key(KeyCode::Enter));
        assert_eq!(s.buffer().unwrap().stamp(), source);
        assert_eq!(s.buffer().unwrap().history().len(), 0);
        assert!(!s.buffer().unwrap().is_modified());
        assert!(s.remaining_input(Instant::now()).is_none());
    }
}

// A captured snapshot stays distinct from ongoing local edits. Typing and undo
// proceed before settlement, and a result cannot steal a changed source view.
#[test]
fn editing_during_generation() {
    let f = Fixture::new();
    let mut s = editor(&f, "settle_gate", "non-modal", true);
    s.execute(Command::AiExplain);
    support::until(|| f.0.join("partial").exists());
    let source = s.buffer().unwrap().id();
    s.handle(Event::Paste("local".into()));
    let changed = text(&s);
    assert!(changed.contains("local"));
    fs::write(f.0.join("release"), "").unwrap();
    ready(&mut s);
    assert_eq!(s.buffer().unwrap().id(), source);
    assert_eq!(text(&s), changed);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "é original selection\n");
    let prompt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(f.0.join("prompt")).unwrap()).unwrap();
    assert_eq!(prompt["observations"]["text"], "é original");
    assert_eq!(
        fs::read_to_string(f.0.join("source.txt")).unwrap(),
        "é original selection\n"
    );
}

// C-g and owning modal Escape fence late completions. A second command cannot
// replace a stopping child; normal navigation remains available throughout.
#[test]
fn native_cancel() {
    for profile in ["non-modal", "modal"] {
        let f = Fixture::new();
        let mut s = editor(&f, "late_cancel", profile, true);
        s.execute(Command::AiExplain);
        support::until(|| f.0.join("prompt").exists());
        if profile == "modal" {
            s.handle(key(KeyCode::Esc));
        } else {
            s.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('g'),
                KeyModifiers::CONTROL,
            )));
        }
        assert!(s.echo().contains("canceled"), "{}", s.echo());
        s.execute(Command::AiExplain);
        assert!(s.echo().contains("active or stopping"));
        s.execute(Command::Motion(Motion::ForwardChar));
        support::until(|| {
            tick(&mut s);
            s.remaining_input(Instant::now()).is_none()
        });
        assert!(!s.buffer().unwrap().is_transient());
        assert_eq!(text(&s), "é original selection\n");
        assert!(f.0.join("closed").exists());
    }
}

// Selection admission fails before starting Pi; backend/authentication failure
// leaves the editor source available, and later ordinary editing still works.
#[test]
fn unavailable_and_admission() {
    let f = Fixture::new();
    let mut s = editor(&f, "normal", "non-modal", false);
    s.execute(Command::AiExplain);
    assert!(s.echo().contains("active selection"));
    assert!(!f.0.join("rpc-started").exists());
    for mode in ["auth", "version", "invalid", "missing"] {
        let f = Fixture::new();
        let mut s = editor(&f, mode, "non-modal", true);
        let stamp = s.buffer().unwrap().stamp();
        if mode == "missing" {
            fs::remove_file(f.0.join("pi-peer")).unwrap();
        }
        s.execute(Command::AiExplain);
        support::until(|| {
            tick(&mut s);
            s.remaining_input(Instant::now()).is_none()
        });
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert!(!s.echo().contains("SECRET-FIXTURE"));
        assert!(!s.buffer().unwrap().is_transient());
        s.insert('x');
        assert!(text(&s).contains('x'));
    }
}

// Canceling an M-x/colon selector belongs to that selector, leaving a running
// explanation intact. Dropping the editor closes and reaps only its Pi child.
#[test]
fn prompt_and_exit_ownership() {
    let f = Fixture::new();
    let mut s = editor(&f, "hold", "modal", true);
    s.execute(Command::AiExplain);
    support::until(|| f.0.join("prompt").exists());
    s.execute(Command::ExecuteExtended);
    s.handle(key(KeyCode::Esc));
    assert!(
        !f.0.join("requests").exists()
            || !fs::read_to_string(f.0.join("requests"))
                .unwrap()
                .contains("clear_queue")
    );
    s.execute(Command::Modal(ModalCommand::Normal));
    drop(s);
    assert!(f.0.join("closed").exists());
    let pid = fs::read_to_string(f.0.join("pid")).unwrap();
    assert!(!std::path::Path::new("/proc").join(pid).exists());
}

// Unsaved data is labeled as a native snapshot without an implicit disk save;
// explaining an already read-only source does not require source write access.
#[test]
fn dirty_and_readonly_sources() {
    for mode in ["normal", "readonly"] {
        let f = Fixture::new();
        let mut s = editor(&f, mode, "non-modal", true);
        if mode == "normal" {
            s.replace(0..0, "unsaved ").unwrap();
            // Editing clears the old region; capture a new native selection.
            s.execute(Command::Motion(Motion::BufferStart));
            s.execute(Command::Region(rev::command::RegionCommand::Set));
            s.execute(Command::Motion(Motion::ForwardWord));
        }
        let stamp = s.buffer().unwrap().stamp();
        let expected = text(&s);
        s.execute(Command::AiExplain);
        ready(&mut s);
        let prompt: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(f.0.join("prompt")).unwrap()).unwrap();
        assert_eq!(prompt["observations"]["dirty"], mode == "normal");
        if mode == "normal" {
            assert_eq!(prompt["observations"]["text"], "unsaved");
        }
        assert_eq!(
            fs::read_to_string(f.0.join("source.txt")).unwrap(),
            "é original selection\n"
        );
        s.execute(Command::Buffer(rev::command::BufferCommand::Switch));
        s.handle(Event::Paste("source.txt".into()));
        s.handle(key(KeyCode::Enter));
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(text(&s), expected);
        assert_eq!(s.buffer().unwrap().is_readonly(), mode == "readonly");
    }
}

// A result arriving while the command selector owns input must preserve that
// selector and the source view; its pending command can still execute normally.
#[test]
fn result_preserves_prompt() {
    let f = Fixture::new();
    let mut s = editor(&f, "settle_gate", "non-modal", true);
    s.execute(Command::AiExplain);
    support::until(|| f.0.join("partial").exists());
    let source = s.buffer().unwrap().stamp();
    let point = s.view().point.position;
    s.execute(Command::ExecuteExtended);
    fs::write(f.0.join("release"), "").unwrap();
    ready(&mut s);
    assert_eq!(s.buffer().unwrap().stamp(), source);
    s.handle(Event::Paste("forward-char".into()));
    s.handle(key(KeyCode::Enter));
    assert!(s.view().point.position > point);
}
