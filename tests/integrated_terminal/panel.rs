#[path = "interaction.rs"]
mod interaction;

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

impl Drop for Editor {
    fn drop(&mut self) {
        // The owned child test process must not exit before cleanup workers.
        self.session.shutdown_terminals().unwrap();
    }
}

impl Editor {
    fn new(profile: &str, height: u16) -> Self {
        Self::with_keys(profile, height, "")
    }
    fn with_keys(profile: &str, height: u16, keys: &str) -> Self {
        Self::with_script(
            profile,
            height,
            keys,
            "printf '\\033[2J\\033[HREADY:%s\\n' \"$1\"\npwd\nwhile IFS= read -r line; do\n case \"$line\" in\n ping) printf '\\033[31mPONG\\033[0m\\n';;\n change) mkdir -p changed; cd changed; printf CHANGED;;\n where) pwd;;\n scroll) i=0; while [ $i -lt 60 ]; do printf \"LINE%s\\n\" \"$i\"; i=$((i+1)); done;;\n alt) printf '\\033[?1049h\\033[2J\\033[HALTERNATE';;\n exit) exit 7;;\n *) printf 'INPUT:%s\\n' \"$line\";;\n esac\ndone\n",
        )
    }
    fn with_script(profile: &str, height: u16, keys: &str, script: &str) -> Self {
        let fixture = Fixture::new();
        let script = fixture.write("child.sh", script);
        let config = fixture.write("config.toml", &format!(
            "[editor]\ninteraction_profile='{profile}'\n[terminal]\nprogram='/bin/sh'\nargs=[{},'literal;not-evaluated']\n{keys}",
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
                .terminal_buffer()
                .and_then(|b| b.terminal())
                .or_else(|| self.session.terminals().ids().next())
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
    if isolated("panel::native_panel", true) {
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
        editor.named("terminal-toggle", colon);
        editor
            .session
            .handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
        let frame = editor.session.frame().unwrap();
        assert_eq!(frame.rows()[11].band(), rev::display::Band::Divider);
        assert!(
            frame.rows()[12]
                .text()
                .contains("READY:literal;not-evaluated")
        );
        editor
            .session
            .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        assert!(editor.session.terminal_focused());
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
    if isolated("panel::panel_admission", false) {
        return;
    }
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
    editor.session.handle(Event::Resize(u16::MAX, 8));
    assert_eq!(
        editor
            .session
            .terminals()
            .get(id)
            .unwrap()
            .geometry()
            .columns,
        80
    );
    let mut native = Vec::new();
    editor.session.frame().unwrap().write(&mut native).unwrap();
    assert!(native.len() < 2_000_000);
    editor.session.handle(Event::Resize(80, 24));
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
    if isolated("panel::source_guards", false) {
        return;
    }
    let mut editor = Editor::new("non-modal", 24);
    editor.session.insert('X');
    let source = editor.session.buffer().unwrap().stamp();
    editor.open();
    let buffer = editor.session.terminal_buffer().unwrap();
    let edit = rev::buffer::edit::Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "source edit".into(),
    };
    assert!(matches!(
        buffer.prepare(&[edit], &[], None),
        Err(rev::buffer::edit::Error::ReadOnly)
    ));
    assert!(editor.session.replace(0..0, "source edit").is_err());
    editor
        .session
        .handle(key(KeyCode::Char('x'), KeyModifiers::CONTROL));
    editor.session.handle(Event::Paste("prefix-paste\n".into()));
    editor.wait(|s| text(s).contains("INPUT:prefix-paste"));
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
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
    assert!(frame.rows().iter().any(|r| r.text().contains("terminal")));
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
    let closing = Instant::now();
    editor
        .session
        .handle(key(KeyCode::Char('y'), KeyModifiers::NONE));
    assert!(
        closing.elapsed() < Duration::from_millis(100),
        "native close blocked input"
    );
    assert!(matches!(
        editor
            .session
            .terminals()
            .get(editor.session.terminals().ids().next().unwrap())
            .unwrap()
            .state(),
        rev::term::State::Closing
    ));
    assert!(editor.session.terminal_buffer().is_some());
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(!editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    let deadline = Instant::now() + Duration::from_secs(3);
    while editor.session.terminals().ids().next().is_some() {
        editor.session.expire_input(Instant::now());
        assert!(
            Instant::now() < deadline,
            "confirmed close: {}",
            editor.session.echo()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(editor.session.terminals().ids().count(), 0);
    assert!(!editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    editor
        .session
        .execute(Command::History(rev::command::History::Undo));
    assert_ne!(editor.session.buffer().unwrap().stamp(), source);
}

// Effective command bindings share the same live host handler in both profiles;
// they remain host keys while ordinary characters retain child ownership.
#[test]
fn terminal_keys() {
    if isolated("panel::terminal_keys", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let mut editor = Editor::with_keys(
            profile,
            24,
            "[keys.editor]\n'C-t'='terminal-toggle'\n[keys.vim_normal]\n'C-t'='terminal-toggle'",
        );
        editor
            .session
            .handle(key(KeyCode::Char('t'), KeyModifiers::CONTROL));
        assert!(
            editor.session.terminal_focused(),
            "{}",
            editor.session.echo()
        );
        editor.wait(|s| text(s).contains("READY:literal;not-evaluated"));
        editor.send("ping");
        editor.wait(|s| text(s).contains("PONG"));
        editor
            .session
            .handle(key(KeyCode::Char('t'), KeyModifiers::CONTROL));
        assert!(!editor.session.terminal_focused());
        assert_eq!(editor.session.terminals().ids().count(), 1);
    }
}

// Relocation changes only attachment/geometry. Rejected geometry leaves the
// same typed buffer, child, alternate screen and valid original placement.
#[test]
fn live_relocation() {
    if isolated("panel::live_relocation", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let mut editor = Editor::new(profile, 24);
        let source = editor.session.buffer().unwrap().stamp();
        editor.open();
        let buffer = editor.session.terminal_buffer().unwrap().id();
        let id = editor.session.terminals().ids().next().unwrap();
        let pid = editor.session.terminals().get(id).unwrap().process_id();
        editor.send("change");
        editor.wait(|s| text(s).contains("CHANGED"));
        editor.send("alt");
        editor.wait(|s| text(s).contains("ALTERNATE"));
        editor.session.handle(Event::Resize(u16::MAX, 24));
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::MoveToBuffer));
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
        assert_eq!(
            editor.session.terminals().get(id).unwrap().process_id(),
            pid
        );
        editor.session.handle(Event::Resize(80, 24));
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::MoveToBuffer));
        assert_eq!(editor.session.buffer().unwrap().id(), buffer);
        assert!(editor.session.terminal_focused());
        assert!(text(editor.session.terminals().get(id).unwrap()).contains("ALTERNATE"));
        assert!(
            editor
                .session
                .terminals()
                .get(id)
                .unwrap()
                .screen()
                .mode()
                .contains(alacritty_terminal::term::TermMode::ALT_SCREEN)
        );
        assert!(
            editor
                .session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("ALTERNATE"))
        );
        editor.session.handle(Event::Resize(80, 7));
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::MoveToPanel));
        assert_eq!(editor.session.buffer().unwrap().id(), buffer);
        editor.session.handle(Event::Resize(80, 24));
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::MoveToPanel));
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
        assert_eq!(editor.session.terminal_buffer().unwrap().id(), buffer);
        assert_eq!(editor.session.terminals().ids().count(), 1);
        assert_eq!(
            editor.session.terminals().get(id).unwrap().process_id(),
            pid
        );
        editor.send("where");
        editor.wait(|s| text(s).contains("source/changed"));
        editor.send("ping");
        editor.wait(|s| text(s).contains("PONG"));
    }
}

// Direct creation needs no lower-panel space, and its live buffer cannot gain
// source authority. Recovery returns the exact original unsaved source.
#[test]
fn direct_buffer() {
    if isolated("panel::direct_buffer", false) {
        return;
    }
    let mut editor = Editor::new("non-modal", 7);
    editor.session.insert('X');
    let source = editor.session.buffer().unwrap().stamp();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::NewBuffer));
    editor.wait(|s| text(s).contains("READY"));
    let id = editor.session.terminals().ids().next().unwrap();
    assert_eq!(editor.session.buffer().unwrap().terminal(), Some(id));
    editor.session.execute(Command::SaveBuffer);
    assert!(editor.session.echo().contains("requires a source buffer"));
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert_eq!(editor.session.terminals().ids().count(), 1);
}

// Listing and renaming retain independent processes. Selecting a hidden peer
// and reopening the instance default never creates another child.
#[test]
fn independent_sessions() {
    if isolated("panel::independent_sessions", false) {
        return;
    }
    let mut editor = Editor::new("non-modal", 24);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::List));
    assert_eq!(editor.session.terminals().ids().count(), 0);
    editor.open();
    let first = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    let first_buffer = editor.session.terminal_buffer().unwrap().id();
    let first_pid = editor.session.terminals().get(first).unwrap().process_id();
    assert_eq!(editor.session.terminal_buffer().unwrap().name(), "terminal");
    editor.send("one");
    editor.wait(|s| text(s).contains("INPUT:one"));
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::New));
    editor.wait(|s| text(s).contains("READY"));
    let second = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    assert_ne!(first, second);
    assert_eq!(
        editor.session.terminal_buffer().unwrap().name(),
        "terminal-2"
    );
    let second_pid = editor.session.terminals().get(second).unwrap().process_id();
    editor.send("two");
    editor.wait(|s| text(s).contains("INPUT:two"));
    assert!(!text(editor.session.terminals().get(first).unwrap()).contains("INPUT:two"));
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::List));
    let frame = editor.session.frame().unwrap();
    assert!(frame.rows().iter().any(|r| r.text().contains("terminal-2")));
    assert!(
        !frame
            .rows()
            .iter()
            .any(|r| r.text().contains("document.org"))
    );
    editor.session.handle(key(KeyCode::Up, KeyModifiers::NONE));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(editor.session.terminal_buffer().unwrap().id(), first_buffer);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Rename));
    editor.session.handle(Event::Paste("work".into()));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(editor.session.terminal_buffer().unwrap().name(), "work");
    assert_eq!(editor.session.terminal_buffer().unwrap().id(), first_buffer);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Rename));
    editor.session.handle(Event::Paste("terminal-2".into()));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(editor.session.terminal_buffer().unwrap().name(), "work");
    assert!(editor.session.echo().contains("already has that name"));
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    assert_eq!(editor.session.terminal_buffer().unwrap().id(), first_buffer);
    assert_eq!(editor.session.terminals().ids().count(), 2);
    assert_eq!(
        editor.session.terminals().get(first).unwrap().process_id(),
        first_pid
    );
    assert_eq!(
        editor.session.terminals().get(second).unwrap().process_id(),
        second_pid
    );
    assert!(text(editor.session.terminals().get(second).unwrap()).contains("INPUT:two"));
}

// Mirrors share one child while focus owns geometry. Navigation offsets stay
// attachment-local across focus, resize, suspension and moving another view.
#[test]
fn mirrored_panes() {
    if isolated("panel::mirrored_panes", false) {
        return;
    }
    let mut editor = Editor::new("non-modal", 24);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::NewBuffer));
    editor.wait(|s| text(s).contains("READY"));
    let terminal = editor.session.buffer().unwrap().terminal().unwrap();
    let pid = editor
        .session
        .terminals()
        .get(terminal)
        .unwrap()
        .process_id();
    let buffer = editor.session.buffer().unwrap().id();
    editor
        .session
        .execute(Command::Split(rev::window::Split::Right));
    let panes = editor.session.windows().order();
    assert_eq!(panes.len(), 2);
    editor.session.select_view(panes[1]).unwrap();
    assert_eq!(editor.session.buffer().unwrap().id(), buffer);
    let right_geometry = editor.session.terminals().get(terminal).unwrap().geometry();
    editor.send("scroll");
    editor.wait(|s| text(s).contains("LINE59"));
    editor.session.set_terminal_viewport(panes[0], 3).unwrap();
    assert!(
        editor
            .session
            .set_terminal_viewport(panes[0], usize::MAX)
            .is_err()
    );
    assert_eq!(
        editor
            .session
            .windows()
            .get(panes[0])
            .unwrap()
            .terminal
            .offset,
        3
    );
    assert_eq!(
        editor
            .session
            .windows()
            .get(panes[1])
            .unwrap()
            .terminal
            .offset,
        0
    );
    let frame = editor.session.frame().unwrap();
    let row = frame.rows()[0].text();
    let (left, right) = row.split_once('│').unwrap();
    assert_ne!(left.trim(), right.trim());
    let mut native = Vec::new();
    frame.write(&mut native).unwrap();
    assert!(native.windows(4).any(|s| s == b"LINE"));
    editor.session.select_view(panes[0]).unwrap();
    assert_ne!(
        editor
            .session
            .terminals()
            .get(terminal)
            .unwrap()
            .geometry()
            .columns,
        right_geometry.columns
    );
    assert_eq!(
        editor
            .session
            .windows()
            .get(panes[0])
            .unwrap()
            .terminal
            .offset,
        3
    );
    editor.session.handle(Event::Resize(80, 3));
    let admitted = editor.session.terminals().get(terminal).unwrap().geometry();
    assert!(editor.session.select_view(panes[1]).is_err());
    assert_eq!(editor.session.windows().selected_id(), panes[0]);
    editor.session.frame().unwrap();
    assert_eq!(
        editor.session.terminals().get(terminal).unwrap().geometry(),
        admitted
    );
    editor.session.handle(Event::Resize(100, 30));
    editor.session.select_view(panes[1]).unwrap();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::MoveToPanel));
    assert_eq!(
        editor
            .session
            .windows()
            .get(panes[0])
            .unwrap()
            .point
            .stamp
            .buffer,
        buffer
    );
    assert_eq!(
        editor
            .session
            .windows()
            .get(panes[0])
            .unwrap()
            .terminal
            .offset,
        3
    );
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::MoveToBuffer));
    assert_eq!(editor.session.buffer().unwrap().id(), buffer);
    assert_eq!(
        editor
            .session
            .terminals()
            .get(terminal)
            .unwrap()
            .process_id(),
        pid
    );
    assert_eq!(editor.session.terminals().ids().count(), 1);
}

// Directory entry uses admitted resource context. Other-pane entry prepares
// the ordinary split/selection before publishing and reuses the live process.
#[test]
fn directory_and_other_pane() {
    if isolated("panel::directory_and_other_pane", false) {
        return;
    }
    let mut editor = Editor::new("modal", 24);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::OpenHere));
    editor.wait(|s| text(s).contains("READY"));
    let id = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
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
    let original = editor.session.windows().selected_id();
    editor.session.handle(Event::Resize(u16::MAX, 24));
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::OpenOtherPane));
    assert_eq!(editor.session.windows().iter().len(), 1);
    assert_eq!(editor.session.windows().selected_id(), original);
    assert_eq!(
        editor.session.terminals().get(id).unwrap().process_id(),
        pid
    );
    editor.session.handle(Event::Resize(80, 24));
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::OpenOtherPane));
    assert_eq!(editor.session.windows().iter().len(), 2);
    assert_ne!(editor.session.windows().selected_id(), original);
    assert_eq!(editor.session.buffer().unwrap().terminal(), Some(id));
    assert_eq!(
        editor.session.terminals().get(id).unwrap().process_id(),
        pid
    );
    assert_eq!(editor.session.terminals().ids().count(), 1);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::OpenOtherPane));
    assert_eq!(editor.session.windows().selected_id(), original);
    assert_eq!(editor.session.terminals().ids().count(), 1);
    let mut tiny = Editor::new("non-modal", 4);
    let view = tiny.session.windows().selected_id();
    tiny.session
        .execute(Command::Terminal(TerminalCommand::OpenOtherPane));
    assert_eq!(tiny.session.terminals().ids().count(), 0);
    assert_eq!(tiny.session.windows().iter().len(), 1);
    assert_eq!(tiny.session.windows().selected_id(), view);
    let mut fresh = Editor::new("non-modal", 24);
    fresh
        .session
        .execute(Command::Terminal(TerminalCommand::OpenOtherPane));
    fresh.wait(|s| text(s).contains("READY"));
    assert_eq!(fresh.session.windows().iter().len(), 2);
    assert_eq!(fresh.session.terminals().ids().count(), 1);
    assert!(fresh.session.buffer().unwrap().terminal().is_some());
}

// Native file observers retire asynchronously. Isolate their descriptors and
// color cache from the engine's exact descriptor-baseline regression tests.
fn isolated(name: &str, colored: bool) -> bool {
    if std::env::var("REV_TERMINAL_PANEL_CHILD").as_deref() == Ok(name) {
        return false;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", name, "--nocapture"])
        .env("REV_TERMINAL_PANEL_CHILD", name);
    if colored {
        command.env_remove("NO_COLOR");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}
