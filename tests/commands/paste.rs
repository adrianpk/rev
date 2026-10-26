use crossterm::event::{Event, KeyCode};
use rev::buffer::Buffers;
use rev::command::{Command, History, KillCommand, LinearCommand, Motion, RegionCommand, Split};
use rev::coordinates::{Coordinates, Scalar};
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

// A payload spanning many rope chunks remains a single checked change, including
// exact Unicode/control bytes and the absence of an invented final newline.
#[test]
fn large_payload() {
    let source = format!("{}<tail>", "prefix\n".repeat(300));
    let point = source.chars().count() - 6;
    let payload = format!(
        "{}end",
        "αe\u{301}界👩\u{200d}💻\t\r\nline\n\x18\x03".repeat(512)
    );
    let mut session = editor(&source, point, false);
    paste(&mut session, &payload);
    let expected = format!("{}{payload}<tail>", "prefix\n".repeat(300));
    let buffer = session.buffer().unwrap();
    assert_eq!(text(&session), expected);
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(buffer.history().back().unwrap().changes().len(), 1);
    assert_eq!(
        session.view().point.position,
        Scalar(point + payload.chars().count())
    );
    assert!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .chunks()
            .count()
            > 8
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point.position, Scalar(point));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}

// Paste preserves its own separators regardless of the prevailing buffer
// newline policy; undo restores both the source bytes and original metadata.
#[test]
fn newline_bytes() {
    for (name, source, newline) in [("LF", "a\nb", "\n"), ("CRLF", "a\r\nb", "\r\n")] {
        let mut session = editor(source, 1, false);
        paste(&mut session, "x\r\ny\nz\r\n");
        assert_eq!(
            text(&session),
            format!("ax\r\ny\nz\r\n{}", &source[1..]),
            "{name}"
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.buffer().unwrap().newline(), newline, "{name}");
        assert!(!session.buffer().unwrap().is_modified(), "{name}");
    }
}

// Empty, binary and read-only admission failures preserve the complete live
// view, mark activation, text identity and history without publishing a prefix.
#[test]
fn refused_payload() {
    let cases = [
        ("empty", "", false),
        ("binary", "ok\0bad", false),
        ("readonly", "界\r\n", true),
    ];
    for (name, payload, readonly) in cases {
        let mut session = editor("abc\nlast", 1, readonly);
        session.execute(Command::Region(RegionCommand::Set));
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        paste(&mut session, payload);
        let buffer = session.buffer().unwrap();
        assert_eq!(text(&session), "abc\nlast", "{name}");
        assert_eq!(*session.view(), view, "{name}");
        assert_eq!(buffer.stamp(), stamp, "{name}");
        assert_eq!(buffer.state(), state, "{name}");
        assert_eq!(buffer.mark(), Some(Scalar(1)), "{name}");
        assert!(buffer.mark_active(), "{name}");
        assert!(buffer.history().is_empty(), "{name}");
        assert!(buffer.redo_history().is_empty(), "{name}");
    }
}

// One paste maps every independent point in the shared buffer. Undo from the
// other pane removes the whole payload and restores the initiating point too.
#[test]
fn shared_payload() {
    let mut session = editor("abc\nlast", 1, false);
    session.execute(Command::Region(RegionCommand::Set));
    let first = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let second = session.windows().order()[1];
    session.select_view(second).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.select_view(first).unwrap();
    paste(&mut session, "x\r\n界");
    assert_eq!(text(&session), "ax\r\n界bc\nlast");
    assert_eq!(session.view().point.position, Scalar(5));
    assert_eq!(
        session.windows().get(second).unwrap().point.position,
        Scalar(12)
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(1)));
    assert!(!session.buffer().unwrap().mark_active());
    session.select_view(second).unwrap();
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "abc\nlast");
    assert_eq!(session.view().point.position, Scalar(8));
    assert_eq!(
        session.windows().get(first).unwrap().point.position,
        Scalar(1)
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// Pending modifiers are canceled by paste, held suffixes and releases cannot
// replay it, and a following C-g cancels input without undoing delivered text.
#[test]
fn input_boundaries() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let prefixes = [
        ("C-x", vec![super::control('x')]),
        ("C-x r", vec![super::control('x'), super::character('r')]),
        ("Meta", vec![super::key(KeyCode::Esc)]),
    ];
    for (name, events) in prefixes {
        let mut session = editor("", 0, false);
        for event in events {
            session.handle(event);
        }
        paste(&mut session, "\x18\x03u");
        for kind in [Release, Repeat] {
            assert_eq!(
                session.handle(super::phase(super::character('u'), kind)),
                Action::Wait,
                "{name}"
            );
        }
        session.handle(super::control('g'));
        assert_eq!(text(&session), "\x18\x03u", "{name}");
        session.handle(super::character('!'));
        assert_eq!(text(&session), "\x18\x03u!", "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), 2, "{name}");
    }
}

// Prompt admission counts Unicode scalars rather than bytes and rejects the
// whole over-limit payload. Canceling a pasted path never edits the document.
#[test]
fn prompt_bounds() {
    let initial = "é".repeat(4095);
    let mut input = Input::default();
    input.argument("Path: ", &initial).unwrap();
    assert!(matches!(
        input.handle(Event::Paste("界z".into())),
        Outcome::Message(_)
    ));
    assert_eq!(input.prompt().unwrap().text(), initial);
    assert_eq!(input.handle(Event::Paste("界".into())), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text().chars().count(), 4096);
    assert_eq!(input.prompt().unwrap().cursor(), initial.len() + 3);
    let mut session = editor("source", 2, false);
    let view = session.view().clone();
    session.execute(Command::FindFile);
    paste(&mut session, "unused-é.txt");
    session.handle(super::control('g'));
    assert_eq!(text(&session), "source");
    assert_eq!(*session.view(), view);
    paste(&mut session, "界");
    assert_eq!(text(&session), "so界urce");
}

// Every paste attempt ends yank-pop continuity, including an empty or rejected
// payload; a stale capture must never replace earlier text or rotate the ring.
#[test]
fn yank_boundary() {
    for (name, payload) in [("empty", ""), ("NUL", "\0"), ("literal", "\x18\x03")] {
        let mut session = editor("aa bb", 0, false);
        session.execute(Command::Kill(KillCommand::Word));
        session.execute(Command::KeyboardQuit);
        session.execute(Command::Kill(KillCommand::Word));
        session.execute(Command::Linear(LinearCommand::Yank));
        paste(&mut session, payload);
        let before = text(&session);
        let stamp = session.buffer().unwrap().stamp();
        let ring: Vec<_> = session.kill_ring().map(ToString::to_string).collect();
        session.execute(Command::Linear(LinearCommand::Pop));
        assert_eq!(session.echo(), "No current yank to replace", "{name}");
        assert_eq!(text(&session), before, "{name}");
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{name}");
        assert_eq!(
            session
                .kill_ring()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ring,
            "{name}"
        );
    }
}

// Pasted text can join a neighbor into one grapheme. The cursor lands after the
// cluster while undo still removes exactly the raw inserted span, not its neighbor.
#[test]
fn joined_graphemes() {
    let cases = [
        ("accent", "\u{301}z", 0, "e", 2),
        ("CRLF", "\nz", 0, "\r", 2),
        ("ZWJ", "💻z", 0, "👩\u{200d}", 3),
        ("suffix", "ez", 1, "\u{301}", 2),
    ];
    for (name, source, point, payload, target) in cases {
        let mut session = editor(source, point, false);
        paste(&mut session, payload);
        assert_eq!(session.view().point.position, Scalar(target), "{name}");
        assert!(
            Coordinates::new(session.buffer().unwrap())
                .unwrap()
                .is_boundary(Scalar(target))
                .unwrap(),
            "{name}"
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.view().point.position, Scalar(point), "{name}");
    }
}
