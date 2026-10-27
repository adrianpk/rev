use rev::buffer::{Buffers, Stamp, State};
use rev::command::{Command, Motion, RegionCommand, Scroll, Split};
use rev::coordinates::Scalar;
use rev::session::Session;
use rev::view::{Size, View};

use super::{Event, KeyCode, KeyModifiers, editor, key, submit};

#[derive(Debug, PartialEq, Eq)]
struct Document {
    bytes: Vec<u8>,
    stamp: Stamp,
    state: State,
    dirty: bool,
    readonly: bool,
    history: usize,
    redo: usize,
    mark: Option<Scalar>,
    active: bool,
}

fn document(session: &Session) -> Document {
    let buffer = session.buffer().unwrap();
    let mut bytes = Vec::new();
    rev::file::write(buffer, &mut bytes).unwrap();
    Document {
        bytes,
        stamp: buffer.stamp(),
        state: buffer.state(),
        dirty: buffer.is_modified(),
        readonly: buffer.is_readonly(),
        history: buffer.history().len(),
        redo: buffer.redo_history().len(),
        mark: buffer.mark(),
        active: buffer.mark_active(),
    }
}

// Destinations use logical lines and exact scalar endpoints. CRLF, Unicode,
// final empty lines and both signed extremes retain source bytes and history.
#[test]
fn logical_bounds() {
    for (name, source, starts) in [
        ("empty", "", vec![0]),
        ("single Unicode", "界e\u{301}", vec![0]),
        ("LF", "first\nlast", vec![0, 6]),
        ("CRLF Unicode", "界e\u{301}\r\nx\r\nlast", vec![0, 5, 8]),
        ("final empty", "a\r\n界\n", vec![0, 3, 5]),
    ] {
        let mut session = editor(source);
        let before = document(&session);
        for line in [i64::MIN, -1, 0, 1, 2, 3, 4, i64::MAX] {
            session.execute(Command::GotoLine);
            submit(&mut session, &line.to_string());
            let expected = if line <= 0 {
                0
            } else {
                usize::try_from(line - 1)
                    .ok()
                    .and_then(|index| starts.get(index).copied())
                    .unwrap_or(source.chars().count())
            };
            assert_eq!(session.view().point.position.0, expected, "{name}: {line}");
            assert!(
                session.frame().unwrap().cursor().is_some(),
                "{name}: {line}"
            );
            assert_eq!(document(&session), before, "{name}: {line}");
        }
    }
}

// A successful goto reveals only its selected view and extends the ordinary
// shared mark. Read-only admission changes no document, save or history state.
#[test]
fn shared_readonly() {
    for readonly in [false, true] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&"界\trow\n".repeat(80)).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(readonly);
        let view = View::new(buffer);
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 10,
            },
        )
        .unwrap();
        session.execute(Command::Region(RegionCommand::Set));
        let selected = session.windows().selected_id();
        session.execute(Command::Split(Split::Right));
        let other = session.windows().order()[1];
        session.select_view(other).unwrap();
        session.execute(Command::Motion(Motion::BufferEnd));
        session.select_view(selected).unwrap();
        session.frame().unwrap();
        let peer = session.windows().get(other).unwrap().clone();
        let before = document(&session);
        session.execute(Command::GotoLine);
        session.handle(Event::Paste("37".into()));
        assert_eq!(document(&session), before);
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
        let frame = session.frame().unwrap();
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap().row,
            36
        );
        assert!(session.view().top > 0);
        assert!(frame.cursor().is_some());
        assert_eq!(*session.windows().get(other).unwrap(), peer);
        assert_eq!(session.windows().selected_id(), selected);
        assert_eq!(document(&session), before);
        let region = session
            .buffer()
            .unwrap()
            .region(session.view().point)
            .unwrap()
            .unwrap();
        assert_eq!(region.range, 0..216);
    }
}

// Entering and canceling either alias or M-x retains unsaved text, complete
// view state (including navigation goals), viewport, saved identity and history.
#[test]
fn canceled_view() {
    for name in ["plain", "meta", "named"] {
        let mut session = editor(&"0123456789abcdef\n".repeat(60));
        session.execute(Command::Motion(Motion::BufferStart));
        for _ in 0..8 {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        for _ in 0..25 {
            session.execute(Command::Scroll(Scroll::NextLine));
        }
        session.execute(Command::Scroll(Scroll::Recenter));
        session.frame().unwrap();
        let before = document(&session);
        let view = session.view().clone();
        if name == "named" {
            session.execute(Command::ExecuteExtended);
            submit(&mut session, "goto-line");
        } else {
            session.handle(key(KeyCode::Char('g'), KeyModifiers::ALT));
            session.handle(key(
                KeyCode::Char('g'),
                if name == "plain" {
                    KeyModifiers::NONE
                } else {
                    KeyModifiers::ALT
                },
            ));
        }
        session.handle(Event::Paste("37".into()));
        session.frame().unwrap();
        assert_eq!(*session.view(), view, "{name}");
        assert_eq!(document(&session), before, "{name}");
        session.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        session.frame().unwrap();
        assert_eq!(*session.view(), view, "{name}");
        assert_eq!(document(&session), before, "{name}");
    }
}

// A host-side edit advances the source while input is pending. Acceptance must
// reject that stale capture and preserve the newer revision and its mapped view.
#[test]
fn stale_source() {
    let mut session = editor("first\nsecond\nlast");
    session.execute(Command::GotoLine);
    session.replace(0..0, "new ").unwrap();
    let before = document(&session);
    let view = session.view().clone();
    submit(&mut session, "2");
    assert_eq!(session.echo(), "Goto line source changed");
    assert_eq!(*session.view(), view);
    assert_eq!(document(&session), before);
    session.execute(Command::GotoLine);
    submit(&mut session, "2");
    assert_eq!(session.view().point.position, Scalar(10));
}
