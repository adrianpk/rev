use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, History, KillCommand, LinearCommand};
use rev::coordinates::Scalar;
use rev::gutter::Change;
use rev::session::Session;
use rev::view::{Size, View};

fn editor(source: &str, position: usize, width: u16, wrap: bool, readonly: bool) -> Session {
    let fixture = super::fixture::Fixture::new();
    let startup =
        rev::config::prepare(&fixture.write("config.toml", &format!("editor.soft_wrap = {wrap}")));
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(readonly);
    let mut view = View::new(buffer);
    view.gutter.apply(Change::Visible(false));
    view.point.position = Scalar(position);
    Session::new(buffers, view, Size { width, height: 7 }).unwrap()
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn control(session: &mut Session, value: char) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char(value),
        KeyModifiers::CONTROL,
    )));
}

// Native aliases resolve the same source-qualified visual rows and retain the
// intended column across short rows. Explicit no-wrap keeps logical commands.
#[test]
fn native_visual_motion() {
    let source = "one two three four five\nlast";
    let mut session = editor(source, 1, 12, true, false);
    let stamp = session.buffer().unwrap().stamp();
    for (key, point) in [('e', 7), ('a', 0), ('n', 8), ('n', 19), ('p', 8)] {
        control(&mut session, key);
        assert_eq!(session.view().point.position, Scalar(point), "C-{key}");
        assert!(session.frame().unwrap().cursor().is_some());
    }
    for (key, point) in [(KeyCode::Down, 19), (KeyCode::Up, 8), (KeyCode::Right, 9)] {
        session.handle(Event::Key(KeyEvent::new(key, KeyModifiers::NONE)));
        assert_eq!(session.view().point.position, Scalar(point));
    }
    control(&mut session, 'n');
    assert_eq!(session.view().point.position, Scalar(20));
    control(&mut session, 'p');
    assert_eq!(session.view().point.position, Scalar(9));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());

    let mut session = editor(source, 1, 12, false, false);
    for (key, point) in [('e', 23), ('a', 0), ('n', 24), ('p', 0)] {
        control(&mut session, key);
        assert_eq!(
            session.view().point.position,
            Scalar(point),
            "unwrapped C-{key}"
        );
    }
}

// Emacs 31.1 terminal observations at twelve text cells pin wrap whitespace,
// hard-token boundaries, real LF/CRLF and EOF. Kills use the ordinary one-edit
// transaction; read-only rejection leaves both text and the existing ring alone.
#[test]
fn visual_kill_boundaries() {
    let cases = [
        (
            "word interior",
            "one two three four five\nlast",
            1,
            "ne two ",
        ),
        ("word edge", "one two three four five\nlast", 7, " "),
        (
            "continuation",
            "one two three four five\nlast",
            8,
            "three four ",
        ),
        (
            "hard token",
            "abcdefghijklmnopqrstuvwx\nlast",
            0,
            "abcdefghijk",
        ),
        ("token edge", "abcdefghijklmnopqrstuvwx\nlast", 11, "l"),
        ("logical LF", "one two three four five\nlast", 23, "\n"),
        (
            "logical CRLF",
            "one two three four five\r\nlast",
            23,
            "\r\n",
        ),
        ("EOF", "abcdef", 6, ""),
    ];
    for (name, source, position, removed) in cases {
        for readonly in [false, true] {
            let mut session = editor(source, position, 12, true, readonly);
            let stamp = session.buffer().unwrap().stamp();
            control(&mut session, 'k');
            if readonly || removed.is_empty() {
                assert_eq!(text(&session), source, "{name}");
                assert_eq!(session.buffer().unwrap().stamp(), stamp, "{name}");
                assert_eq!(session.kill_ring().len(), 0, "{name}");
                assert!(session.buffer().unwrap().history().is_empty(), "{name}");
            } else {
                assert_eq!(
                    session.killed_text().unwrap().to_string(),
                    removed,
                    "{name}"
                );
                assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
                assert_eq!(session.view().point.position, Scalar(position), "{name}");
                session.execute(Command::History(History::Undo));
                assert_eq!(text(&session), source, "{name}");
                assert!(!session.buffer().unwrap().is_modified(), "{name}");
            }
        }
    }
}

// Reflow after each C-k keeps the normal directional kill chain. Yank restores
// its exact source bytes, and each kill remains independently undoable.
#[test]
fn visual_kill_chain() {
    let source = "one two three four five\nlast";
    let mut session = editor(source, 1, 12, true, false);
    for _ in 0..2 {
        session.execute(Command::Kill(KillCommand::Line));
    }
    assert_eq!(session.kill_ring().len(), 1);
    assert_eq!(
        session.killed_text().unwrap().to_string(),
        "ne two three four "
    );
    assert_eq!(text(&session), "ofive\nlast");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::Linear(LinearCommand::Yank));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "ofive\nlast");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "othree four five\nlast");
}
