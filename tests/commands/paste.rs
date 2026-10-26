use crossterm::event::{Event, KeyCode};
use rev::buffer::Buffers;
use rev::command::{Command, History, RegionCommand};
use rev::coordinates::Scalar;
use rev::input::{Focus, Input, Outcome};
use rev::session::{Action, Session};
use rev::view::{Size, View};

fn editor(source: &str, position: usize, readonly: bool) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_read_only(readonly);
    let mut view = View::new(buffer);
    view.point.position = Scalar(position);
    Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
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

fn paste(session: &mut Session, text: &str) {
    assert_eq!(session.handle(Event::Paste(text.into())), Action::Redraw);
}

// Pasted control/escape-looking text is a literal owned payload, never a stream
// of keys; it clears pending nested/Meta prefixes before another chord arrives.
#[test]
fn literal_payload() {
    let payload = "é\r\n界\x18\x03\x1b>\x07";
    let mut input = Input::default();
    input.handle(super::control('x'));
    input.handle(super::character('r'));
    input.handle(super::key(KeyCode::Esc));
    assert_eq!(
        input.handle(Event::Paste(payload.into())),
        Outcome::Paste(payload.into())
    );
    assert_eq!(input.prefix(), None);
    assert_eq!(input.focus(), Focus::Editing);
    assert!(matches!(
        input.handle(super::character('a')),
        Outcome::Editing(_)
    ));
}

// One mixed-newline paste publishes one replacement/revision/history record,
// maps point/mark and renders controls visibly instead of emitting payload ANSI.
#[test]
fn atomic_payload() {
    let payload = "\tαe\u{301}\r\n界\ny\x18\x03\x1b[31m";
    let mut session = editor("<>", 1, false);
    session.execute(Command::Region(RegionCommand::Set));
    let before = session.buffer().unwrap().stamp();
    paste(&mut session, payload);
    let buffer = session.buffer().unwrap();
    assert_eq!(text(&session), format!("<{payload}>"));
    assert_eq!(buffer.stamp().revision, before.revision + 1);
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(buffer.history().back().unwrap().changes().len(), 1);
    assert_eq!(
        buffer.history().back().unwrap().changes()[0]
            .inserted()
            .to_string(),
        payload
    );
    assert_eq!(
        session.view().point.position,
        Scalar(1 + payload.chars().count())
    );
    assert_eq!(buffer.mark(), Some(Scalar(1)));
    assert!(!buffer.mark_active());
    let frame = session.frame().unwrap();
    assert!(frame.rows().iter().all(|row| !row.text().contains('\x1b')));
    let mut wire = Vec::new();
    frame.write(&mut wire).unwrap();
    assert!(!wire.windows(5).any(|part| part == b"\x1b[31m"));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "<>");
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), format!("<{payload}>"));
}

// Ordinary typing on either side forms separate groups; one undo removes the
// entire paste, and redo recovers exact CRLF bytes without merging the groups.
#[test]
fn paste_group() {
    let mut session = editor("", 0, false);
    session.insert('A');
    paste(&mut session, "é\r\n界");
    session.insert('Z');
    assert_eq!(session.buffer().unwrap().history().len(), 3);
    for expected in ["Aé\r\n界", "A", ""] {
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), expected);
    }
    assert!(!session.buffer().unwrap().is_modified());
    for expected in ["A", "Aé\r\n界", "Aé\r\n界Z"] {
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected);
    }
}

// Prompt paste inserts a bounded single-line literal at its own cursor. A
// multiline/control payload rejects wholly; decision and inventory never accept it.
#[test]
fn focused_payload() {
    let mut input = Input::default();
    input.argument("Path: ", "az").unwrap();
    input.handle(super::key(KeyCode::Left));
    assert_eq!(input.handle(Event::Paste("é界".into())), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text(), "aé界z");
    assert_eq!(
        input.handle(Event::Paste("ok\nno".into())),
        Outcome::Message("Control character rejected")
    );
    assert_eq!(input.prompt().unwrap().text(), "aé界z");
    input.cancel();
    for focus in [Focus::Decision, Focus::Inventory] {
        if focus == Focus::Decision {
            input.decision("Overwrite?").unwrap();
        } else {
            input.inventory().unwrap();
        }
        assert!(matches!(
            input.handle(Event::Paste("y".into())),
            Outcome::Message(_)
        ));
        assert_eq!(input.focus(), focus);
        input.cancel();
    }
}
