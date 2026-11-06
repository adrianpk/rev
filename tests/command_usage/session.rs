use crate::fixture::Fixture;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::usage::{History, Store};
use rev::command::{Command, History as Undo};
use rev::config::prepare;
use rev::display::Band;
use rev::session::Session;
use rev::view::{Size, View};
use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, Instant};

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn cancel(editor: &mut Session) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
}

fn editor(fixture: &Fixture, profile: &str, path: &Path) -> Session {
    let startup = prepare(&fixture.write(
        "config.toml",
        &format!(
            "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false"
        ),
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let view = View::with_settings(buffers.get(buffers.initial()).unwrap(), &startup.snapshot);
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
    .with_command_usage(History::open(path))
}

fn named(editor: &mut Session, name: &str, modal: bool) {
    if modal {
        editor.handle(key(KeyCode::Char(':')));
    } else {
        editor.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::ALT,
        )));
    }
    editor.handle(Event::Paste(name.into()));
    editor.handle(key(KeyCode::Enter));
}

fn candidates(editor: &mut Session) -> Vec<String> {
    editor
        .frame()
        .unwrap()
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| row.text().trim().to_owned())
        .collect()
}

// Both profiles persist canonical names once, including a later canceled
// argument. Restart restores ranking, while distinct-name navigation stays local.
#[test]
fn session_restart() {
    for profile in ["non-modal", "modal"] {
        let fixture = Fixture::new();
        let path = fixture.0.join("state/history.toml");
        let mut first = editor(&fixture, profile, &path);
        named(&mut first, "goto-line", profile == "modal");
        cancel(&mut first);
        first.execute(Command::History(Undo::Undo));
        first.flush_command_usage().unwrap();
        assert_eq!(super::store::disk(&path), ["goto-line"]);
        first.flush_command_usage().unwrap();
        assert_eq!(super::store::disk(&path), ["goto-line"]);
        let mut restarted = editor(&fixture, profile, &path);
        restarted.execute(Command::ExecuteExtended);
        assert_eq!(candidates(&mut restarted)[0], "goto-line");
        restarted.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::ALT,
        )));
        assert_eq!(restarted.echo(), "Command history is empty");
        cancel(&mut restarted);
        assert_eq!(super::store::disk(&path), ["goto-line"]);
    }
}

// A real publisher changes disk while a selector is open. Publication rebases
// the owner's memory, but paint, resize, Tab and Return retain the chosen identity.
#[test]
fn retained_snapshot() {
    let fixture = Fixture::new();
    let path = fixture.0.join("history.toml");
    let mut editor = editor(&fixture, "non-modal", &path);
    named(&mut editor, "undo", false);
    named(&mut editor, "undo-redo", false);
    editor.execute(Command::ExecuteExtended);
    editor.handle(key(KeyCode::Down));
    let before = candidates(&mut editor);
    let (mut other, _) = Store::open(&path).unwrap();
    other.publish(&super::store::names(&["goto-line"])).unwrap();
    editor.publish_command_usage();
    editor.flush_command_usage().unwrap();
    editor.handle(Event::Resize(70, 20));
    assert_eq!(candidates(&mut editor), before);
    let selected = editor
        .frame()
        .unwrap()
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.selection().is_some())
        .unwrap()
        .text()
        .trim()
        .to_owned();
    assert_eq!(selected, "undo");
    editor.handle(key(KeyCode::Tab));
    editor.handle(key(KeyCode::Enter));
    editor.flush_command_usage().unwrap();
    assert_eq!(
        super::store::disk(&path),
        ["goto-line", "undo", "undo-redo", "undo"]
    );
}

// Busy publication leaves no recurring idle deadline. A later explicit owned
// attempt publishes the bounded pending queue once and ordinary editing continues.
#[test]
fn deferred_publication() {
    let fixture = Fixture::new();
    let path = fixture.write("history.toml", "version=1\ncommands=[]");
    let holder = File::create(path.with_extension("toml.lock")).unwrap();
    holder.try_lock().unwrap();
    let mut editor = editor(&fixture, "non-modal", &path);
    named(&mut editor, "undo", false);
    let start = Instant::now();
    assert!(editor.flush_command_usage().unwrap_err().contains("busy"));
    assert!(start.elapsed() < Duration::from_secs(1));
    for _ in 0..4 {
        editor.expire_input(Instant::now());
    }
    assert!(editor.remaining_input(Instant::now()).is_none());
    assert!(super::store::disk(&path).is_empty());
    holder.unlock().unwrap();
    editor.publish_command_usage();
    editor.flush_command_usage().unwrap();
    assert_eq!(super::store::disk(&path), ["undo"]);
    editor.flush_command_usage().unwrap();
    assert_eq!(super::store::disk(&path), ["undo"]);
}

// More than 256 pending events coalesce without growing a job queue. The newest
// retained events displace older saved state when the busy publisher can resume.
#[test]
fn pending_capacity() {
    let fixture = Fixture::new();
    let path = fixture.write("history.toml", "version=1\ncommands=['goto-line']");
    let holder = File::create(path.with_extension("toml.lock")).unwrap();
    holder.try_lock().unwrap();
    let mut editor = editor(&fixture, "non-modal", &path);
    let events: Vec<_> = (0..300)
        .map(|index| if index % 2 == 0 { "undo" } else { "undo-redo" })
        .collect();
    for (index, name) in events.iter().enumerate() {
        named(&mut editor, name, false);
        if index % 16 == 0 {
            editor.publish_command_usage();
        }
    }
    assert!(editor.flush_command_usage().is_err());
    holder.unlock().unwrap();
    editor.flush_command_usage().unwrap();
    assert_eq!(super::store::disk(&path), events[44..]);
}

// Invalid or newly replaced history never disables dispatch or overwrites the
// rejected bytes. Diagnostics do not echo malformed file contents or drafts.
#[test]
fn invalid_fallback() {
    for when in ["startup", "publication"] {
        let fixture = Fixture::new();
        let invalid = "private-invalid-content";
        let path = fixture.write(
            "history.toml",
            if when == "startup" {
                invalid
            } else {
                "version=1\ncommands=[]"
            },
        );
        let mut editor = editor(&fixture, "non-modal", &path);
        if when == "startup" {
            assert!(editor.echo().contains("Command usage:"));
            assert!(!editor.echo().contains(invalid));
        } else {
            fs::write(&path, invalid).unwrap();
        }
        named(&mut editor, "undo", false);
        let error = editor.flush_command_usage().unwrap_err();
        assert!(error.contains("Command usage:") && !error.contains(invalid));
        editor.expire_input(Instant::now());
        if when == "publication" {
            assert!(editor.echo().contains("Command usage:"));
            assert!(!editor.echo().contains(invalid));
        }
        editor.execute(Command::ExecuteExtended);
        assert_eq!(candidates(&mut editor)[0], "undo");
        cancel(&mut editor);
        editor.handle(key(KeyCode::Char('x')));
        assert_eq!(editor.buffer().unwrap().len_chars(), 1);
        assert_eq!(fs::read_to_string(&path).unwrap(), invalid);
    }
}

// Unknown historical names never become admitted commands. All compiled names
// and maximum-size registered AI identities fit the same strict storage syntax.
#[test]
fn admitted_identities() {
    let fixture = Fixture::new();
    let path = fixture.write("history.toml", "version=1\ncommands=['ai-removed','undo']");
    let mut editor = editor(&fixture, "non-modal", &path);
    editor.execute(Command::ExecuteExtended);
    let rows = candidates(&mut editor);
    assert_eq!(rows[0], "undo");
    assert!(!rows.iter().any(|name| name == "ai-removed"));
    let ai = Command::AiDefined(rev::command::AiName::new(&"a".repeat(64)).unwrap());
    for command in Command::ALL.into_iter().chain([ai]) {
        let name = command.name();
        assert!(name.len() <= 128 && name.as_bytes()[0].is_ascii_lowercase());
        assert!(
            name.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        );
    }
}
