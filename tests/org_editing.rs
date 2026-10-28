#![forbid(unsafe_code)]

#[path = "org_editing/colors.rs"]
mod colors;
#[path = "org_editing/contexts.rs"]
mod contexts;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "org_editing/fold_edges.rs"]
mod fold_edges;
#[path = "org_editing/folds.rs"]
mod folds;
#[path = "org_editing/projection.rs"]
mod projection;
#[path = "org_editing/visibility.rs"]
mod visibility;

use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{Buffers, Mode};
use rev::command::{Command, Split, WindowCommand};
use rev::config::prepare;
use rev::session::Session;
use rev::view::{Size, View};

use fixture::Fixture;

fn editor(config: &str, name: &str, source: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&fixture.write(name, source)).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    (fixture, session)
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn meta(character: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::ALT))
}

fn control(character: char) -> Event {
    Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    ))
}

fn named(session: &mut Session, name: &str) {
    session.handle(meta('x'));
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

const KEYS: &str = "[keys.editor]\n'M-j' = 'forward-char'\n'M-o' = 'org-mode'\n'M-t' = 'text-mode'\n[keys.org]\n'M-j' = 'newline'";

// Visiting, M-x and a validated alias activate the same working Org input path.
// Text mode restores normal editing without changing source merely to switch.
#[test]
fn activation_paths() {
    let cases = [
        ("visit", "notes.org"),
        ("named", "notes.txt"),
        ("binding", "notes.txt"),
    ];
    for (name, filename) in cases {
        let (_fixture, mut session) = editor(KEYS, filename, "ab");
        let before = session.buffer().unwrap().stamp();
        match name {
            "named" => named(&mut session, "org-mode"),
            "binding" => {
                session.handle(meta('o'));
            }
            _ => (),
        }
        assert_eq!(session.buffer().unwrap().mode(), "Org", "{name}");
        assert_eq!(session.buffer().unwrap().stamp(), before, "{name}");
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().contains("Org")),
            "{name}"
        );
        session.handle(meta('j'));
        assert_eq!(text(&session), "\nab", "{name}");
        let edited = session.buffer().unwrap().stamp();
        named(&mut session, "text-mode");
        assert_eq!(session.buffer().unwrap().stamp(), edited, "{name}");
        session.handle(meta('j'));
        assert_eq!(text(&session), "\nab", "{name}");
        session.handle(key(KeyCode::Char('!')));
        assert_eq!(text(&session), "\na!b", "{name}");
        settle(&mut session);
        assert_eq!(session.remaining_input(Instant::now()), None);
        assert!(session.buffer().unwrap().org_index().is_none());
    }
}

// Shared views follow their buffer's mode, while another selected buffer retains
// plain editing. Switching panes cannot reuse a stale Org keymap selection.
#[test]
fn shared_mode() {
    let (fixture, mut session) = editor(KEYS, "notes.org", "ab");
    let org = session.buffer().unwrap().id();
    session.execute(Command::Split(Split::Right));
    assert_eq!(session.windows().iter().len(), 2);
    session.load(&fixture.write("plain.txt", "ab")).unwrap();
    assert_ne!(session.buffer().unwrap().id(), org);
    assert!(!session.buffer().unwrap().is_org());
    session.handle(meta('j'));
    assert_eq!(text(&session), "ab");
    assert_eq!(session.view().point.position.0, 1);
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.buffer().unwrap().id(), org);
    session.handle(meta('j'));
    assert_eq!(text(&session), "\nab");
    session.execute(Command::Mode(Mode::Text));
    assert!(!session.buffer().unwrap().is_org());
    assert_eq!(session.remaining_input(Instant::now()), None);
}

// A mode alias cannot escape an active path prompt or create its draft target.
// Cancel retires the prompt; the next fresh alias changes only buffer metadata.
#[test]
fn prompt_ownership() {
    let (fixture, mut session) = editor(KEYS, "notes.org", "* H\n");
    let before = (session.buffer().unwrap().stamp(), text(&session));
    let absent = fixture.0.join("unwritten.org");
    session.execute(Command::FindFile);
    session.handle(Event::Paste(absent.to_string_lossy().into_owned()));
    session.handle(meta('t'));
    session.execute(Command::Mode(Mode::Text));
    assert!(session.buffer().unwrap().is_org());
    assert_eq!((session.buffer().unwrap().stamp(), text(&session)), before);
    assert!(!absent.exists());
    session.handle(control('g'));
    session.handle(meta('t'));
    assert!(!session.buffer().unwrap().is_org());
    assert_eq!((session.buffer().unwrap().stamp(), text(&session)), before);
    assert!(!absent.exists());
}

// Write-file reassociation uses the existing frozen suffix policy. An explicit
// Text choice survives the same destination change; neither path converts bytes.
#[test]
fn reassociated_mode() {
    for explicit in [false, true] {
        let (fixture, mut session) = editor(KEYS, "old.txt", "\u{feff}* 界\r\nlast");
        if explicit {
            named(&mut session, "text-mode");
        }
        let target = fixture.0.join("new.org");
        named(&mut session, "write-file");
        session.handle(Event::Paste(target.to_string_lossy().into_owned()));
        session.handle(key(KeyCode::Enter));
        assert_eq!(session.buffer().unwrap().is_org(), !explicit);
        assert_eq!(
            std::fs::read(&target).unwrap(),
            "\u{feff}* 界\r\nlast".as_bytes()
        );
        assert_eq!(text(&session), "* 界\r\nlast");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Disabled Org rejects both named and aliased entry while keeping plain editing.
// Failed UTF-8 ingress cannot retire an already active Org buffer's identity.
#[test]
fn safe_fallbacks() {
    let config = format!("org.enabled = false\n{KEYS}");
    let (_fixture, mut session) = editor(&config, "notes.org", "ab");
    for invocation in ["named", "binding"] {
        if invocation == "named" {
            named(&mut session, "org-mode");
        } else {
            session.handle(meta('o'));
        }
        assert!(session.echo().contains("org.enabled"));
        assert_eq!(session.buffer().unwrap().mode(), "Text");
        assert_eq!(text(&session), "ab");
    }
    session.handle(meta('j'));
    assert_eq!(session.view().point.position.0, 1);
    let (fixture, mut session) = editor(KEYS, "active.org", "ab");
    let invalid = fixture.0.join("invalid.org");
    std::fs::write(&invalid, b"\xff").unwrap();
    let before = session.buffer().unwrap().stamp();
    assert!(session.load(&invalid).is_err());
    assert_eq!(session.buffer().unwrap().stamp(), before);
    assert!(session.buffer().unwrap().is_org());
    session.handle(meta('j'));
    assert_eq!(text(&session), "\nab");
}

// Tree focus and its prompt own input above Org. Canceling M-x restores the
// tree, and leaving the tree restores the original buffer's Org keymap.
#[test]
fn workspace_precedence() {
    let config = format!("{KEYS}\n[keys.tree]\n'M-j' = 'tree-next-line'");
    let (_fixture, mut session) = editor(&config, "notes.org", "ab");
    let before = session.buffer().unwrap().stamp();
    session.execute(Command::Tree(rev::command::TreeCommand::Focus));
    session.execute(Command::Tree(rev::command::TreeCommand::Expand));
    let selected = session.tree().selected().unwrap().path().to_path_buf();
    session.handle(meta('j'));
    assert_ne!(session.tree().selected().unwrap().path(), selected);
    assert_eq!(session.buffer().unwrap().stamp(), before);
    session.handle(meta('x'));
    session.handle(Event::Paste("text-mode".into()));
    session.handle(control('g'));
    assert!(session.tree().focused());
    assert!(session.buffer().unwrap().is_org());
    session.handle(key(KeyCode::Tab));
    assert!(!session.tree().focused());
    session.handle(meta('j'));
    assert_eq!(text(&session), "\nab");
}

// Wait for only the session-owned observation, preserving the idle assertion
// after mode cancellation rather than assuming a background job ends inline.
fn settle(session: &mut Session) {
    let until = Instant::now() + std::time::Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(Instant::now() < until, "owned Org work did not finish");
        session.expire_input(Instant::now() + std::time::Duration::from_millis(10));
        std::thread::yield_now();
    }
}
