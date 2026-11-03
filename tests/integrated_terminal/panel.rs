use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::{Buffers, Resource},
    command::{BufferCommand, Command, TerminalCommand},
    session::Session,
    view::{Size, View},
};
use std::time::{Duration, Instant};

use super::{Fixture, text};

struct Editor {
    session: Session,
    fixture: Fixture,
}

impl Editor {
    fn new(profile: &str, height: u16) -> Self {
        let fixture = Fixture::new();
        let script = fixture.write("child.sh", "printf '\\033[2J\\033[HREADY:%s\\n' \"$1\"\npwd\nwhile IFS= read -r line; do\n case \"$line\" in\n ping) printf '\\033[31mPONG\\033[0m\\n';;\n exit) exit 7;;\n *) printf 'INPUT:%s\\n' \"$line\";;\n esac\ndone\n");
        let config = fixture.write("config.toml", &format!(
            "[editor]\ninteraction_profile='{profile}'\n[terminal]\nprogram='/bin/sh'\nargs=[{},'literal;not-evaluated']\n",
            serde_json::to_string(&script.to_string_lossy()).unwrap(),
        ));
        let startup = rev::config::prepare(&config);
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let source = fixture.write("source/document.org", "* source stays native\n");
        let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
        let id = buffers.visit(&source).unwrap();
        let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
        let session = Session::new(buffers, view, Size { width: 80, height }).unwrap();
        Self { session, fixture }
    }
    fn open(&mut self) {
        self.session
            .execute(Command::Terminal(TerminalCommand::Open));
        assert!(self.session.terminal_focused(), "{}", self.session.echo());
        self.wait(|s| text(s).contains("READY:literal;not-evaluated"));
    }
    fn wait(&mut self, ready: impl Fn(&rev::term::Session) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.session.expire_input(Instant::now());
            if self
                .session
                .terminals()
                .ids()
                .next()
                .and_then(|id| self.session.terminals().get(id))
                .is_some_and(&ready)
            {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "native terminal deadline: {}",
                self.session.echo()
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn send(&mut self, line: &str) {
        for c in line.chars() {
            self.session
                .handle(key(KeyCode::Char(c), KeyModifiers::NONE));
        }
        self.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    }
    fn named(&mut self, command: &str, colon: bool) {
        self.session.handle(if colon {
            key(KeyCode::Char(':'), KeyModifiers::NONE)
        } else {
            key(KeyCode::Char('x'), KeyModifiers::ALT)
        });
        self.session.handle(Event::Paste(command.into()));
        self.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    }
}
fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

// Both native command-entry profiles launch only on demand. Toggle/navigation
// retain the same PTY and ordinary source while live keys reach the real child.
#[test]
fn native_panel() {
    if std::env::var_os("REV_TERMINAL_COLOR_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "panel::native_panel", "--nocapture"])
            .env("REV_TERMINAL_COLOR_CHILD", "1")
            .env_remove("NO_COLOR")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    for (profile, colon) in [("non-modal", false), ("modal", true)] {
        let mut editor = Editor::new(profile, 24);
        assert_eq!(editor.session.terminals().ids().count(), 0);
        let source = editor.session.buffer().unwrap().stamp();
        let view = editor.session.windows().selected_id();
        editor.named("terminal-open", colon);
        assert!(
            editor.session.terminal_focused(),
            "{}",
            editor.session.echo()
        );
        editor.wait(|s| text(s).contains("READY:literal;not-evaluated"));
        let id = editor.session.terminals().ids().next().unwrap();
        let pid = editor.session.terminals().get(id).unwrap().process_id();
        assert!(
            text(editor.session.terminals().get(id).unwrap()).contains(
                &editor
                    .fixture
                    .0
                    .join("source")
                    .to_string_lossy()
                    .to_string()
            )
        );
        editor.send("ping");
        editor.wait(|s| text(s).contains("PONG"));
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
        assert_eq!(editor.session.windows().selected_id(), view);
        let resource = editor.session.terminal_buffer().unwrap();
        assert_eq!(resource.resource(), Resource::Terminal(id));
        assert_eq!(resource.mode(), "Terminal");
        assert!(resource.is_readonly() && resource.is_transient());
        assert_eq!(resource.len_bytes(), 0);
        let buffer_id = resource.id();
        let frame = editor.session.frame().unwrap();
        assert_eq!(frame.rows().len(), 24);
        let mut native = Vec::new();
        frame.write(&mut native).unwrap();
        assert!(
            native
                .windows(b"\x1b[38;2;205;0;0m".len())
                .any(|s| s == b"\x1b[38;2;205;0;0m")
        );
        editor.named("terminal-toggle", false);
        assert!(!editor.session.terminal_focused());
        assert!(
            !editor
                .session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("PONG"))
        );
        editor.named("terminal-toggle", colon);
        assert!(editor.session.terminal_focused());
        assert_eq!(editor.session.terminals().ids().count(), 1);
        assert_eq!(
            editor.session.terminals().get(id).unwrap().process_id(),
            pid
        );
        assert_eq!(editor.session.terminal_buffer().unwrap().id(), buffer_id);
        editor
            .session
            .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        assert!(!editor.session.terminal_focused());
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    }
}

// Rejected presentation cannot allocate a child or steal source focus. A small
// admitted panel clamps its rows and restores the remembered height after resize.
#[test]
fn panel_admission() {
    let mut editor = Editor::new("non-modal", 7);
    let source = editor.session.buffer().unwrap().stamp();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    assert_eq!(editor.session.terminals().ids().count(), 0);
    assert!(!editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    editor.session.handle(Event::Resize(80, 8));
    editor.open();
    let id = editor.session.terminals().ids().next().unwrap();
    assert_eq!(
        editor.session.terminals().get(id).unwrap().geometry().rows,
        3
    );
    editor.session.handle(Event::Resize(80, 24));
    assert_eq!(
        editor.session.terminals().get(id).unwrap().geometry().rows,
        10
    );
    editor.session.handle(Event::Resize(80, 4));
    assert_eq!(editor.session.frame().unwrap().rows().len(), 4);
    editor.session.handle(Event::Resize(80, 24));
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Toggle));
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Toggle));
    assert_eq!(
        editor.session.terminals().get(id).unwrap().geometry().rows,
        10
    );
}

// Terminal focus cannot borrow a source buffer's editing, file or provider
// authority. Host prompts remain usable and cancel before child focus changes.
#[test]
fn source_guards() {
    let mut editor = Editor::new("non-modal", 24);
    editor.session.insert('X');
    let source = editor.session.buffer().unwrap().stamp();
    editor.open();
    for name in [
        "save-buffer",
        "write-file",
        "revert-buffer",
        "undo",
        "undo-redo",
        "org-mode",
        "completion-at-point",
        "bookmark-set",
        "lsp-connect",
        "org-insert-heading",
        "ai-prompt",
        "shell-command-on-region",
    ] {
        let command = Command::named(name).unwrap();
        editor.session.execute(command);
        assert!(
            editor.session.echo().contains("requires a source buffer"),
            "{name}: {}",
            editor.session.echo()
        );
        assert_eq!(editor.session.buffer().unwrap().stamp(), source, "{name}");
        assert!(editor.session.terminal_focused());
    }
    editor
        .session
        .handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(editor.session.terminal_focused());
    editor.session.execute(Command::Buffer(BufferCommand::List));
    let frame = editor.session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|r| r.text().contains("*terminal-1*"))
    );
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert!(editor.session.terminal_focused());
    editor.session.execute(Command::Buffer(BufferCommand::Kill));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
    editor
        .session
        .handle(key(KeyCode::Char('n'), KeyModifiers::NONE));
    assert_eq!(editor.session.terminals().ids().count(), 1);
    editor.session.execute(Command::Buffer(BufferCommand::Kill));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
    editor
        .session
        .handle(key(KeyCode::Char('y'), KeyModifiers::NONE));
    assert_eq!(editor.session.terminals().ids().count(), 0);
    assert!(!editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    editor
        .session
        .execute(Command::History(rev::command::History::Undo));
    assert_ne!(editor.session.buffer().unwrap().stamp(), source);
}
