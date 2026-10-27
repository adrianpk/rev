#[path = "startup_errors.rs"]
mod errors;
#[path = "startup_lifetime.rs"]
mod lifetime;

use std::path::Path;
use std::process::Command as Process;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, TreeCommand};
use rev::config::{Roots, Startup};
use rev::session::Session;
use rev::view::Size;

use crate::fixture::Fixture;

fn process(test: &str, fixture: &Fixture) -> Process {
    let mut process = Process::new(std::env::current_exe().unwrap());
    process
        .args(["--exact", test, "--nocapture"])
        .current_dir(fixture.0.join("project"))
        .env("REV_STARTUP_CHILD", "1")
        .env_remove("NO_COLOR")
        .env("TERM", "xterm-256color")
        .env("HOME", fixture.0.join("home"))
        .env("XDG_CONFIG_HOME", fixture.0.join("user"))
        .env("XDG_DATA_HOME", fixture.0.join("data"))
        .env("XDG_STATE_HOME", fixture.0.join("state"))
        .env("XDG_CACHE_HOME", fixture.0.join("cache"));
    process
}

fn success(output: std::process::Output) {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn child(test: &str, fixture: &Fixture) {
    success(process(test, fixture).output().unwrap());
}

fn isolated() -> bool {
    std::env::var_os("REV_STARTUP_CHILD").is_some()
}

fn launch(path: Option<&Path>) -> Session {
    Session::startup(
        path,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Real environment discovery freezes explicit user includes before the initial
// file is visited. Project-local files cannot replace editing or key policies.
#[test]
fn discovered_consumers() {
    if isolated() {
        consumers();
        return;
    }
    let fixture = Fixture::new();
    fixture.write("user/rev/config.toml", "include = ['editing.toml']\n[palette]\nforeground = '#010203'\n[keys.editor]\n'M-j' = 'newline'\n[keys.minibuffer]\n'M-a' = 'minibuffer-accept'\n[keys.tree]\n'M-l' = 'tree-expand'");
    fixture.write("user/rev/editing.toml", "editor.tab_width = 8\neditor.indent_width = 2\nfiles.newline = 'crlf'\nfiles.bom = true\n[language.go]\nindent_width = 3");
    fixture.write("project/config.toml", "invalid local configuration");
    fixture.write(
        "project/.rev/config.toml",
        "invalid hidden local configuration",
    );
    fixture.write("project/example.go", "\tkept");
    child("startup::discovered_consumers", &fixture);
    for name in ["home", "data", "state", "cache"] {
        assert!(!fixture.0.join(name).exists(), "{name}");
    }
}

fn consumers() {
    let startup = Startup::from_env();
    assert!(startup.diagnostic.is_none());
    assert_eq!(startup.snapshot.settings().editor.tab_width, 8);
    assert_eq!(startup.snapshot.editor("go").indent_width, 3);
    let mut session = launch(Some(Path::new("example.go")));
    assert_eq!(text(&session), "\tkept");
    assert!(!session.buffer().unwrap().has_bom());
    assert_eq!(
        session.tree().root(),
        Some(std::env::current_dir().unwrap().as_path())
    );
    session.handle(meta('j'));
    assert_eq!(text(&session), "\n\tkept");
    session.handle(key(KeyCode::Tab));
    assert_eq!(text(&session), "\n   \tkept");
    let mut wire = Vec::new();
    session.frame().unwrap().write(&mut wire).unwrap();
    assert!(
        wire.windows(b"\x1b[38;2;1;2;3m".len())
            .any(|part| part == b"\x1b[38;2;1;2;3m")
    );
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("find-file".into()));
    session.handle(meta('a'));
    let frame = session.frame().unwrap();
    let query = frame
        .rows()
        .iter()
        .find(|row| row.band() == rev::display::Band::Echo)
        .unwrap();
    assert!(query.text().starts_with("Find file: "));
    assert_eq!(frame.cursor().unwrap().row, query.row());
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Tree(TreeCommand::Focus));
    session.handle(meta('l'));
    assert!(session.tree().entries().len() > 1);
    let mut fresh = launch(None);
    assert!(fresh.buffer().unwrap().has_bom());
    fresh.handle(meta('j'));
    fresh.handle(key(KeyCode::Tab));
    assert_eq!(text(&fresh), "\r\n  ");
    assert_eq!(
        Roots::from_env()
            .unwrap()
            .config_file()
            .file_name()
            .unwrap(),
        "config.toml"
    );
}
