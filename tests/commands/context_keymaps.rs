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

fn phase(event: Event, kind: KeyEventKind) -> Event {
    let Event::Key(event) = event else {
        unreachable!()
    };
    Event::Key(KeyEvent { kind, ..event })
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
        input.handle(phase(meta('a'), KeyEventKind::Repeat)),
        Outcome::Ignored
    );
    assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
}

// Completion and history aliases go through Palette, preserving its query,
// accepted command identity, history promotion and navigation draft.
#[test]
fn palette_actions() {
    let mut input = Input::with_settings(&settings(
        "[keys.minibuffer]\n'M-a' = 'minibuffer-accept'\n'M-c' = 'minibuffer-complete'\n'M-p' = 'minibuffer-previous'\n'M-n' = 'minibuffer-next'",
    ));
    input.commands().unwrap();
    input.handle(Event::Paste("newl".into()));
    assert_eq!(input.handle(meta('c')), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text(), "newline");
    assert_eq!(input.handle(meta('a')), Outcome::Command(Command::Newline));
    input.commands().unwrap();
    input.handle(Event::Paste("draft".into()));
    input.handle(meta('p'));
    assert_eq!(input.prompt().unwrap().text(), "newline");
    input.handle(meta('n'));
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
    assert_eq!(session.tree().entries().len(), 3);
    session.handle(meta('j'));
    assert!(session.tree().selected().unwrap().path().ends_with("a.txt"));
    session.handle(phase(meta('j'), KeyEventKind::Repeat));
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
