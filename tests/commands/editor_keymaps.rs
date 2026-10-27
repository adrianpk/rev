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
