use crossterm::event::{Event, KeyEvent};
use rev::buffer::{Buffers, Mode};
use rev::command::{Command, ModalCommand};
use rev::config::{Snapshot, prepare};
use rev::coordinates::Scalar;
use rev::input::keymap::Sequence;
use rev::session::Session;
use rev::view::{Size, View};

use super::config_fixture::Fixture;

#[path = "factory_shortcuts/agenda.rs"]
mod agenda;
#[path = "factory_shortcuts/editing.rs"]
mod editing;
#[path = "factory_shortcuts/org.rs"]
mod org;
#[path = "factory_shortcuts/projects.rs"]
mod projects;

const SIZE: Size = Size {
    width: 100,
    height: 32,
};
const STATES: [Option<ModalCommand>; 7] = [
    None,
    Some(ModalCommand::Normal),
    Some(ModalCommand::Insert),
    Some(ModalCommand::Replace),
    Some(ModalCommand::VisualCharacter),
    Some(ModalCommand::VisualLine),
    Some(ModalCommand::VisualBlock),
];

fn settings(config: &str) -> Snapshot {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    startup.snapshot
}

fn editor(config: &str, source: &str, mode: Mode) -> Session {
    source_editor(&settings(config), source, mode, false)
}

fn source_editor(snapshot: &Snapshot, source: &str, mode: Mode, readonly: bool) -> Session {
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.create(source).unwrap();
    buffers.get_mut(id).unwrap().select_mode(mode).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(readonly);
    let mut view = View::with_settings(buffers.get(id).unwrap(), snapshot);
    view.point.position = Scalar(source.chars().count());
    Session::new(buffers, view, SIZE).unwrap()
}

fn profile(state: Option<ModalCommand>) -> &'static str {
    if state.is_some() {
        "[editor]\ninteraction_profile = 'modal'\n"
    } else {
        ""
    }
}

fn enter(session: &mut Session, state: Option<ModalCommand>) {
    if let Some(state) = state {
        session.execute(Command::Modal(state));
    }
}

fn keys(session: &mut Session, sequence: &str) {
    // Normalize test input spelling, without consulting a factory binding table.
    for key in Sequence::parse(sequence).unwrap().chords() {
        session.handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
    }
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
