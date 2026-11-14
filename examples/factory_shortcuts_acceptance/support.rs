use crossterm::event::{Event, KeyEvent};
use rev::{
    buffer::Buffers,
    config::prepare,
    input::keymap::Sequence,
    session::Session,
    view::{Size, View},
};

pub(crate) use crate::metrics::{cpu_ticks, peak_rss};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) const SIZE: Size = Size {
    width: 100,
    height: 24,
};

pub(crate) fn editor(profile: &str, overrides: &str) -> Session {
    let fixture = crate::fixture::Fixture::new();
    let path = fixture.write(
        "config.toml",
        &format!("editor.interaction_profile='{profile}'\nwindows.split_ratio=0.3\n{overrides}"),
    );
    let startup = prepare(&path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.create("  source retained\n").unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    Session::new(buffers, view, SIZE).unwrap()
}

pub(crate) fn send(session: &mut Session, sequence: &str) -> Result<()> {
    for chord in Sequence::parse(sequence)?.chords() {
        session.handle(Event::Key(KeyEvent::new(chord.code, chord.modifiers)));
    }
    Ok(())
}

pub(crate) fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
