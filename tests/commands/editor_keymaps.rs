use crossterm::event::{KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, Scroll};
use rev::config::{Snapshot, prepare};
use rev::coordinates::Scalar;
use rev::input::{Input, Outcome, Prefix};
use rev::session::{Action, Session};
use rev::view::{Size, View};

use super::{Event, KeyCode, character, control, key};

fn settings(config: &str) -> Snapshot {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(
        startup.diagnostic.is_none(),
        "{config}: {:?}",
        startup.diagnostic
    );
    startup.snapshot
}

fn meta(character: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::ALT))
}

fn editor(config: &str, source: &str) -> Session {
    let mut buffers = Buffers::with_settings(settings(config)).unwrap();
    let id = buffers.create(source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

// A user alias reaches the native handler rather than inserting its suffix.
// The inherited chord remains live and uses the same buffer-qualified motion.
#[test]
fn configured_alias() {
    let mut session = editor("[keys.editor]\n'M-j' = 'next-line'", "ab\ncd\nef");
    let stamp = session.buffer().unwrap().stamp();
    assert_eq!(session.handle(meta('j')), Action::Redraw);
    assert_eq!(session.view().point.position, Scalar(3));
    session.handle(control('n'));
    assert_eq!(session.view().point.position, Scalar(6));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// A custom prefix consumes every chord, reports its full normalized label and
// emits one existing command only after its complete bounded sequence arrives.
#[test]
fn custom_sequence() {
    let snapshot = settings("[keys.editor]\n'M-z a b' = 'find-file'");
    let mut input = Input::with_settings(&snapshot);
    assert_eq!(input.handle(meta('z')), Outcome::Prefix(Prefix::Sequence));
    assert_eq!(input.prefix_label().as_deref(), Some("M-z"));
    assert_eq!(
        input.handle(character('a')),
        Outcome::Prefix(Prefix::Sequence)
    );
    assert_eq!(input.prefix_label().as_deref(), Some("M-z a"));
    assert_eq!(
        input.handle(character('b')),
        Outcome::Command(Command::FindFile)
    );
    assert_eq!(input.prefix(), None);
    assert!(matches!(input.handle(character('b')), Outcome::Editing(_)));
}

// Native report aliases and compatible Escape delivery resolve the same custom
// binding, using the existing Meta deadline and repeat owner.
#[test]
fn meta_delivery() {
    let snapshot = settings("[keys.editor]\n'M-j' = 'next-line'\n'C-i' = 'forward-char'");
    let mut input = Input::with_settings(&snapshot);
    input.handle(key(KeyCode::Esc));
    assert_eq!(
        input.handle(character('j')),
        Outcome::Command(Command::Scroll(Scroll::NextLine))
    );
    assert_eq!(
        input.handle(key(KeyCode::Tab)),
        Outcome::Command(Command::Motion(rev::command::Motion::ForwardChar))
    );
    assert_eq!(
        input.handle(control('i')),
        Outcome::Command(Command::Motion(rev::command::Motion::ForwardChar))
    );
}

// Exact replacement does not fall through to the hardcoded original handler.
// Independent inherited aliases remain, and omitting the override restores it.
#[test]
fn exact_reassignment() {
    let overridden = settings("[keys.editor]\n'C-f' = 'backward-char'");
    let restored = settings("# No custom binding.\n");
    for (snapshot, expected) in [
        (&overridden, rev::command::Motion::BackwardChar),
        (&restored, rev::command::Motion::ForwardChar),
    ] {
        let mut input = Input::with_settings(snapshot);
        assert_eq!(
            input.handle(control('f')),
            Outcome::Command(Command::Motion(expected))
        );
        assert_eq!(
            input.handle(key(KeyCode::Right)),
            Outcome::Command(Command::Motion(rev::command::Motion::ForwardChar))
        );
    }
}

// Prompt acquisition and cancellation consume the full custom prefix state.
// Its old suffix becomes prompt text, then ordinary editing after acceptance.
#[test]
fn prompt_reset() {
    let snapshot = settings("[keys.editor]\n'M-z a' = 'find-file'");
    for prompt in ["argument", "commands", "search", "decision", "inventory"] {
        let mut input = Input::with_settings(&snapshot);
        input.handle(meta('z'));
        open_prompt(&mut input, prompt);
        assert_eq!(input.prefix(), None, "{prompt}");
        input.handle(control('g'));
        assert_eq!(input.prefix(), None, "{prompt}");
        assert!(
            matches!(input.handle(character('a')), Outcome::Editing(_)),
            "{prompt}"
        );
    }
}

fn open_prompt(input: &mut Input, prompt: &str) {
    match prompt {
        "argument" => input.argument("Text: ", ""),
        "commands" => input.commands(),
        "search" => input.search(rev::search::Direction::Forward),
        "decision" => input.decision("Continue? y/n"),
        "inventory" => input.inventory(),
        _ => unreachable!(),
    }
    .unwrap();
}

// Editor reassignment cannot change a focused prompt's native cursor binding.
// A mismatch/cancel consumes all prefix chords before editing resumes.
#[test]
fn transient_isolation() {
    let snapshot = settings("[keys.editor]\n'C-f' = 'find-file'\n'M-z a' = 'next-line'");
    let mut input = Input::with_settings(&snapshot);
    input.argument("Text: ", "ab").unwrap();
    input.handle(key(KeyCode::Home));
    assert_eq!(input.handle(control('f')), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().cursor(), 1);
    input.handle(control('g'));
    input.handle(meta('z'));
    assert_eq!(
        input.handle(character('x')),
        Outcome::Message("Undefined key sequence")
    );
    assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
    input.handle(meta('z'));
    input.handle(control('g'));
    assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
}

// Workspace ownership changes clear pending editor prefixes immediately, even
// when a native host changes focus directly instead of delivering a bound key.
#[test]
fn workspace_reset() {
    use rev::command::TreeCommand;
    let mut session = editor(
        "[keys.editor]\n'C-n' = 'newline'\n'M-z a' = 'newline'",
        "ab\ncd",
    );
    let stamp = session.buffer().unwrap().stamp();
    session.handle(meta('z'));
    session.execute(Command::Tree(TreeCommand::Focus));
    assert!(session.tree().focused());
    session.handle(control('n'));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.execute(Command::Tree(TreeCommand::Toggle));
    assert!(!session.tree().focused());
    session.handle(character('a'));
    assert_eq!(
        session.buffer().unwrap().stamp().revision,
        stamp.revision + 1
    );
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()[0]
            .inserted()
            .to_string(),
        "a"
    );
}
