#[path = "context_keymaps/flows.rs"]
mod flows;
#[path = "context_keymaps/guards.rs"]
mod guards;

use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, TreeCommand};
use rev::config::{Snapshot, prepare};
use rev::input::{Focus, Input, Outcome, Prefix};
use rev::session::Session;
use rev::view::{Size, View};

use super::{Event, KeyCode, character, control, key};

fn settings(config: &str) -> Snapshot {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    startup.snapshot
}

fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}

fn phase(event: &Event, kind: KeyEventKind) -> Event {
    let Event::Key(event) = event else {
        unreachable!()
    };
    Event::Key(KeyEvent { kind, ..*event })
}

fn editor(config: &str, source: &str) -> Session {
    let mut buffers = Buffers::with_settings(settings(config)).unwrap();
    let id = buffers.create(source).unwrap();
    session(buffers, id)
}

fn session(buffers: Buffers, id: rev::buffer::BufferId) -> Session {
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap()
}

fn tree_editor(config: &str) -> (super::config_fixture::Fixture, Session) {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("root/a.txt", "retained");
    fixture.write("root/b.txt", "other");
    let mut buffers = Buffers::with_settings(settings(config)).unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut session = session(buffers, id);
    session.execute(Command::Tree(TreeCommand::Focus));
    (fixture, session)
}

// Select a real loaded entry through native navigation, including the parent
// row, so consent tests never accidentally target navigation or a neighbor.
fn select_tree(session: &mut Session, path: &std::path::Path) {
    let entries = session.tree().entries();
    let target = entries
        .iter()
        .position(|entry| entry.path() == path)
        .unwrap();
    let current = entries
        .iter()
        .position(|entry| entry.path() == session.tree().selected().unwrap().path())
        .unwrap();
    let direction = if target < current {
        KeyCode::Up
    } else {
        KeyCode::Down
    };
    for _ in 0..target.abs_diff(current) {
        session.handle(key(direction));
    }
    assert_eq!(session.tree().selected().unwrap().path(), path);
}

fn command_tokens<'a>(name: &'a str, hint: Option<&'a str>) -> Vec<&'a str> {
    std::iter::once(name)
        .chain(hint.into_iter().flat_map(str::split_whitespace))
        .chain(
            Command::named(name)
                .unwrap()
                .description()
                .into_iter()
                .flat_map(str::split_whitespace),
        )
        .collect()
}

// Both profiles share keyboard/named/remapped upward navigation and effective
// hints. Source-focused refusal preserves the source region; source caret input
// retains its profile's ordinary behavior rather than navigating the tree.
#[test]
fn tree_up_directory() {
    let command = Command::Tree(TreeCommand::UpDirectory);
    assert_eq!(Command::named("tree-up-directory"), Some(command));
    assert_eq!(
        command.description(),
        Some("Show the parent directory of an unanchored file tree.")
    );
    for profile in ["non-modal", "modal"] {
        for route in ["key", "named", "remapped"] {
            let mapping = if route == "remapped" {
                "[keys.tree]\n'^'='tree-next-line'\n'M-u'='tree-up-directory'"
            } else {
                ""
            };
            let (fixture, mut session) = tree_editor(&format!(
                "editor.interaction_profile='{profile}'\n{mapping}"
            ));
            session.handle(Event::Resize(140, 18));
            let source = session.buffer().unwrap().stamp();
            let point = session.view().point;
            let previous = session.tree().root().unwrap().to_owned();
            if route == "key" {
                session.handle(character('^'));
            } else {
                if route == "remapped" {
                    session.handle(character('^'));
                    assert_eq!(session.tree().root(), Some(previous.as_path()));
                }
                session.execute(Command::ExecuteExtended);
                session.handle(Event::Paste("tree-up-directory".into()));
                let frame = session.frame().unwrap();
                let row = frame
                    .rows()
                    .iter()
                    .find(|r| r.text().starts_with("tree-up-directory"))
                    .unwrap();
                assert!(
                    row.text()
                        .contains(if route == "remapped" { "[M-u]" } else { "[^]" })
                );
                assert!(
                    row.text()
                        .trim_end()
                        .ends_with(command.description().unwrap())
                );
                if route == "named" {
                    session.handle(key(KeyCode::Enter));
                } else {
                    session.execute(Command::KeyboardQuit);
                    session.handle(meta('u'));
                }
            }
            assert_eq!(
                session.tree().root(),
                Some(fixture.0.as_path()),
                "{profile}/{route}: {}",
                session.echo()
            );
            assert_eq!(session.tree().selected().unwrap().path(), previous);
            assert!(session.tree().focused());
            assert_eq!(session.buffer().unwrap().stamp(), source);
            assert_eq!(session.view().point, point);
            assert_eq!(text(&session), "retained");
            assert!(session.buffer().unwrap().history().is_empty());
            session.handle(key(KeyCode::Tab));
            session.execute(Command::Region(rev::command::RegionCommand::Set));
            session.execute(Command::Motion(rev::command::Motion::BufferEnd));
            let source_point = session.view().point;
            let mark = session.buffer().unwrap().mark();
            session.execute(Command::ExecuteExtended);
            session.handle(Event::Paste("tree-up-directory".into()));
            session.handle(key(KeyCode::Enter));
            assert!(session.echo().contains("not focused"));
            assert_eq!(session.tree().root(), Some(fixture.0.as_path()));
            assert_eq!(session.buffer().unwrap().stamp(), source);
            assert_eq!(session.buffer().unwrap().mark(), mark);
            assert_eq!(session.view().point, source_point);
            session.execute(Command::KeyboardQuit);
            session.handle(character('^'));
            assert_eq!(session.tree().root(), Some(fixture.0.as_path()));
            if profile == "non-modal" {
                assert_eq!(text(&session), "retained^");
            } else {
                assert_eq!(text(&session), "retained");
                assert_eq!(session.view().point.position, rev::coordinates::Scalar(0));
            }
        }
    }
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Reassigned Return edits instead of accepting. Custom aliases still use the
// prompt's grapheme cursor/deletion and consume its argument exactly once.
#[test]
fn prompt_actions() {
    let mut input = Input::with_settings(&settings(
        "[keys.minibuffer]\n'RET' = 'backward-char'\n'M-h' = 'backward-delete-char'\n'M-a' = 'minibuffer-accept'",
    ));
    input.argument("Text: ", "e\u{301}界").unwrap();
    input.handle(key(KeyCode::Enter));
    assert_eq!(input.focus(), Focus::Argument);
    assert_eq!(input.prompt().unwrap().cursor(), "e\u{301}".len());
    input.handle(meta('h'));
    assert_eq!(input.prompt().unwrap().text(), "界");
    assert_eq!(input.prompt().unwrap().cursor(), 0);
    input.handle(character('é'));
    assert_eq!(input.handle(meta('a')), Outcome::Argument("é界".into()));
    assert_eq!(
        input.handle(phase(&meta('a'), KeyEventKind::Repeat)),
        Outcome::Ignored
    );
    assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
}

// Completion and history aliases go through Palette, preserving its query,
// accepted command identity, history promotion and navigation draft.
#[test]
fn palette_actions() {
    let mut input = Input::with_settings(&settings(
        "[keys.minibuffer]\n'M-a' = 'minibuffer-accept'\n'M-c' = 'minibuffer-complete'\n'M-h' = 'minibuffer-history-previous'\n'M-l' = 'minibuffer-history-next'",
    ));
    input.commands().unwrap();
    input.handle(Event::Paste("newl".into()));
    assert_eq!(input.handle(meta('c')), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text(), "newline");
    assert_eq!(input.handle(meta('a')), Outcome::Command(Command::Newline));
    input.commands().unwrap();
    input.handle(Event::Paste("draft".into()));
    input.handle(meta('h'));
    assert_eq!(input.prompt().unwrap().text(), "newline");
    input.handle(meta('l'));
    assert_eq!(input.prompt().unwrap().text(), "draft");
    input.handle(control('g'));
    assert_eq!(input.focus(), Focus::Editing);
}

// Tree actions use their existing selection/focus owners, with action-based
// navigation repeats. An editor alias for the same chord cannot edit the file.
#[test]
fn tree_actions() {
    let (_fixture, mut session) = tree_editor(
        "[keys.editor]\n'M-j' = 'newline'\n[keys.tree]\n'M-j' = 'tree-next-line'\n'M-l' = 'tree-expand'\n'M-e' = 'insert-tab'",
    );
    let stamp = session.buffer().unwrap().stamp();
    session.handle(meta('l'));
    assert_eq!(session.tree().entries().len(), 4);
    session.handle(meta('j'));
    assert!(session.tree().selected().unwrap().is_parent());
    session.handle(meta('j'));
    assert!(session.tree().selected().unwrap().path().ends_with("a.txt"));
    session.handle(phase(&meta('j'), KeyEventKind::Repeat));
    assert!(session.tree().selected().unwrap().path().ends_with("b.txt"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.handle(meta('e'));
    assert!(!session.tree().focused());
    session.handle(meta('j'));
    assert_eq!(text(&session), "\nretained");
}

// A minibuffer sequence is consumed by that prompt. Cancel retires its suffix;
// the editor's independent binding remains live after the prompt disappears.
#[test]
fn context_isolation() {
    let config =
        "[keys.editor]\n'M-j' = 'newline'\n[keys.minibuffer]\n'M-j a' = 'minibuffer-accept'";
    let mut input = Input::with_settings(&settings(config));
    input.argument("Text: ", "draft").unwrap();
    assert_eq!(input.handle(meta('j')), Outcome::Prefix(Prefix::Sequence));
    assert_eq!(input.prefix_label().as_deref(), Some("M-j"));
    input.handle(control('g'));
    assert_eq!(input.prefix(), None);
    assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
    let mut session = editor(config, "retained");
    session.execute(Command::FindFile);
    session.handle(meta('j'));
    session.handle(control('g'));
    assert_eq!(text(&session), "retained");
    session.handle(meta('j'));
    assert_eq!(text(&session), "\nretained");
}

// Origin snapshots precede minibuffer activation. Local leaves and prefixes
// shadow inherited editor sequences; changing the owner retires stale hints.
#[test]
fn context_hints() {
    use rev::input::OrgContext;
    for (case, config, expected) in [
        (
            "inherit",
            "[keys.editor]\n'M-j a'='mouse-mode'",
            Some("M-j a"),
        ),
        (
            "leaf shadows prefix",
            "[keys.editor]\n'M-j a'='mouse-mode'\n[keys.org]\n'M-j'='newline'",
            None,
        ),
        (
            "prefix shadows leaf",
            "[keys.editor]\n'M-j'='mouse-mode'\n[keys.org]\n'M-j a'='newline'",
            None,
        ),
        (
            "local replacement",
            "[keys.editor]\n'M-j'='mouse-mode'\n[keys.org]\n'M-j'='newline'\n'M-q'='mouse-mode'",
            Some("M-q"),
        ),
    ] {
        let mut input = Input::with_settings(&settings(&config.replace(
            "[keys.editor]",
            "editor.interaction_profile='modal'\n[keys.vim_normal]",
        )));
        input.set_org_context(Some(OrgContext::Editing));
        input.commands().unwrap();
        assert_eq!(
            input.palette().unwrap().hint(Command::MouseMode),
            expected,
            "{case}"
        );
        assert_eq!(
            input.palette().unwrap().hint(Command::SaveBuffer),
            Some("C-x C-s"),
            "{case}"
        );
        input.set_org_context(None);
        assert_eq!(
            input.palette().unwrap().hint(Command::SaveBuffer),
            None,
            "{case}"
        );
        input.handle(control('g'));
        input.commands().unwrap();
        assert_eq!(
            input.palette().unwrap().hint(Command::MouseMode),
            Some(if case == "inherit" || case == "leaf shadows prefix" {
                "M-j a"
            } else {
                "M-j"
            }),
            "{case}"
        );
    }
    let (_fixture, mut session) = tree_editor(
        "[keys.editor]\n'M-j'='newline'\n[keys.tree]\n'M-j'='tree-refresh'\n'M-a'='newline'\n'M-b'='forward-char'",
    );
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("newline".into()));
    assert!(
        session.frame().unwrap().rows().iter().any(|row| row
            .text()
            .split_whitespace()
            .collect::<Vec<_>>()
            == command_tokens("newline", None))
    );
    for (name, hint) in [
        ("forward-char", None),
        ("insert-tab", None),
        ("tree-open", Some("M-a")),
        ("tree-expand", Some("C-f")),
    ] {
        session.handle(control('g'));
        session.execute(Command::ExecuteExtended);
        session.handle(Event::Paste(name.into()));
        let frame = session.frame().unwrap();
        let row = frame
            .rows()
            .iter()
            .find(|row| row.text().starts_with(name))
            .unwrap();
        let hint = hint.map(|hint| format!("[{hint}]"));
        let expected = command_tokens(name, hint.as_deref());
        assert_eq!(
            row.text().split_whitespace().collect::<Vec<_>>(),
            expected,
            "{name}"
        );
    }
    session.execute(Command::KeyboardQuit);
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("tree-refresh".into()));
    assert!(
        session.frame().unwrap().rows().iter().any(|row| row
            .text()
            .split_whitespace()
            .collect::<Vec<_>>()
            == command_tokens("tree-refresh", Some("[M-j]")))
    );
}

// Live terminal input consumes ordinary editing keys and C-c prefixes. Host
// C-x sequences remain available; the selector records that owner before focus.
#[test]
fn terminal_hints() {
    use rev::command::TerminalCommand;
    for profile in ["non-modal", "modal"] {
        let fixture = super::config_fixture::Fixture::new();
        let script = fixture.write(
            "child.sh",
            "while IFS= read -r line; do printf '%s\\n' \"$line\"; done\n",
        );
        let config = format!(
            "editor.interaction_profile='{profile}'\nterminal.program='/bin/sh'\nterminal.args=[{}]\n[keys.editor]\n'M-j'='mouse-mode'\n'C-c s'='mouse-mode'\n'C-x z'='mouse-mode'\n[keys.vim_normal]\n'M-j'='mouse-mode'\n'C-c s'='mouse-mode'\n'C-x z'='mouse-mode'",
            serde_json::to_string(&script.to_string_lossy()).unwrap()
        );
        let mut session = editor(&config, "source stays native");
        session.handle(Event::Resize(80, 24));
        session.execute(Command::Terminal(TerminalCommand::Open));
        assert!(session.terminal_focused(), "{profile}: {}", session.echo());
        for (name, hint) in [
            ("newline", None),
            ("mouse-mode", Some("C-x z")),
            ("save-buffer", Some("C-x C-s")),
        ] {
            session.handle(meta('x'));
            session.handle(Event::Paste(name.into()));
            let frame = session.frame().unwrap();
            let row = frame
                .rows()
                .iter()
                .find(|row| row.text().starts_with(name))
                .unwrap_or_else(|| {
                    panic!(
                        "{profile}: {name}: echo={}, rows={:?}",
                        session.echo(),
                        frame
                            .rows()
                            .iter()
                            .map(|row| (row.band(), row.text()))
                            .collect::<Vec<_>>()
                    )
                });
            let hint = hint.map(|hint| format!("[{hint}]"));
            let prefix = std::iter::once(name)
                .chain(hint.as_deref())
                .collect::<Vec<_>>()
                .join(" ");
            let actual = row.text().split_whitespace().collect::<Vec<_>>().join(" ");
            let expected = command_tokens(name, hint.as_deref()).join(" ");
            // Names and complete hints survive the 80-column viewport; the
            // description occupies the remaining cells and may end mid-word.
            assert!(
                actual.starts_with(&format!("{prefix} ")),
                "{profile}: {actual}"
            );
            assert!(expected.starts_with(&actual), "{profile}: {actual}");
            session.handle(control('g'));
        }
        session.shutdown_terminals().unwrap();
        assert_eq!(text(&session), "source stays native");
    }
}
