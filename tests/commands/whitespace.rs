#[path = "whitespace/history.rs"]
mod history;

#[path = "whitespace/policy.rs"]
mod policy;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, Motion, RegionCommand, Whitespace};
use rev::coordinates::Scalar;
use rev::session::Session;
use rev::view::{Size, View};

fn editor(source: &str) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
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

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn select(session: &mut Session, start: usize, end: usize) {
    session.execute(Command::Motion(Motion::BufferStart));
    while session.view().point.position.0 < start {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    assert_eq!(session.view().point.position, Scalar(start));
    session.execute(Command::Region(RegionCommand::Set));
    while session.view().point.position.0 < end {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    assert_eq!(session.view().point.position, Scalar(end));
}

fn named(session: &mut Session, name: &str) {
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste(name.into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

// Both catalog identities reach real M-x editing handlers. A missing mark
// reports the ordinary region error and cannot silently convert the buffer.
#[test]
fn named_conversion() {
    for (name, operation, source, expected) in [
        ("tabify", Whitespace::Tabify, "    a  b", "\ta  b"),
        ("untabify", Whitespace::Untabify, "\ta\tb", "    a   b"),
    ] {
        let mut session = editor(source);
        assert_eq!(Command::named(name), Some(Command::Whitespace(operation)));
        named(&mut session, name);
        assert_eq!(session.echo(), "Region is not active", "{name}");
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        select(&mut session, 0, source.chars().count());
        named(&mut session, name);
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
    }
}
