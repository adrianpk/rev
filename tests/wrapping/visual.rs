use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{
    Command, History, KillCommand, LinearCommand, Motion, Scroll, Split, WindowCommand,
};
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

// Pages move the viewport and point by visual rows; recenter cycles through
// center/top/bottom without inventing source lines. Goto still takes source rows.
#[test]
fn pages_and_goto() {
    let source = "aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt\nlast";
    let mut session = editor(source, 1, 6, true, false);
    let stamp = session.buffer().unwrap().stamp();
    control(&mut session, 'v');
    assert_eq!(session.view().point.position, Scalar(19));
    assert_eq!(session.frame().unwrap().cursor().unwrap().row, 0);
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::ALT,
    )));
    assert_eq!(session.view().point.position, Scalar(1));
    for _ in 0..6 {
        control(&mut session, 'n');
    }
    assert_eq!(session.view().point.position, Scalar(37));
    for row in [2, 0, 4] {
        control(&mut session, 'l');
        assert_eq!(session.frame().unwrap().cursor().unwrap().row, row);
    }
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("2".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(
        session.view().point.position,
        Scalar(source.find('\n').unwrap() + 1)
    );
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        1
    );
    assert!(session.frame().unwrap().cursor().is_some());
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Width changes invalidate visual goal columns before the next command even
// without an intervening frame. Split siblings retain independent source points.
#[test]
fn resize_and_split_goals() {
    let source = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut session = editor(source, 3, 8, true, false);
    control(&mut session, 'n');
    assert_eq!(session.view().point.position, Scalar(11));
    session.handle(Event::Resize(10, 7));
    control(&mut session, 'n');
    assert_eq!(session.view().point.position, Scalar(21));
    assert_eq!(session.view().column, Some(1));
    session.execute(Command::Split(Split::Right));
    let selected = session.windows().selected_id();
    let sibling = session
        .windows()
        .iter()
        .find(|(id, _)| *id != selected)
        .unwrap()
        .0;
    control(&mut session, 'n');
    assert_eq!(session.view().point.position, Scalar(25));
    assert_eq!(
        session.windows().get(sibling).unwrap().point.position,
        Scalar(21)
    );
    session.frame().unwrap();
    session.execute(Command::Window(WindowCommand::Keep));
    session.frame().unwrap();
    assert_eq!(session.windows().selected_id(), selected);
    assert_eq!(session.view().point.position, Scalar(25));
    assert_eq!(text(&session), source);
    assert!(!session.buffer().unwrap().is_modified());
}

// Visual goals count cells, while every destination is a grapheme boundary.
// Clamping on a short row retains intent for the return to a tab/wide source row.
#[test]
fn unicode_visual_goals() {
    let source = "a\t界e\u{301} long words more";
    let mut session = editor(source, 3, 8, true, false);
    for (command, position) in [
        (Scroll::NextLine, 10),
        (Scroll::NextLine, 16),
        (Scroll::PreviousLine, 10),
        (Scroll::PreviousLine, 3),
    ] {
        session.execute(Command::Scroll(command));
        assert_eq!(session.view().point.position, Scalar(position));
        assert_eq!(session.view().column, Some(6));
        assert!(
            rev::coordinates::Coordinates::new(session.buffer().unwrap())
                .unwrap()
                .is_boundary(session.view().point.position)
                .unwrap()
        );
        assert!(session.frame().unwrap().cursor().is_some());
    }
}

// Shared edits map each viewport's source anchor before publication. Combining
// insertion at an anchor reprojects that derived boundary and cannot reject an
// otherwise valid edit. Undo restores both source points and exact text.
#[test]
fn shared_viewport_anchors() {
    let source = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut session = editor(source, 55, 18, true, false);
    session.execute(Command::Split(Split::Right));
    session.frame().unwrap();
    let selected = session.windows().selected_id();
    let sibling = session
        .windows()
        .iter()
        .find(|(id, _)| *id != selected)
        .unwrap()
        .0;
    let peer = session.windows().get(sibling).unwrap();
    let anchor = peer.wrapped_rows(session.buffer().unwrap(), 9, 1).unwrap()[0]
        .source
        .start
        .0;
    assert!(anchor > 0);
    session.replace(0..0, "X").unwrap();
    let peer = session.windows().get(sibling).unwrap();
    assert_eq!(peer.point.position, Scalar(56));
    let first = peer.wrapped_rows(session.buffer().unwrap(), 9, 1).unwrap();
    assert!(first[0].source.contains(&Scalar(anchor + 1)));
    session.replace(anchor + 1..anchor + 1, "\u{301}").unwrap();
    session.frame().unwrap();
    for _ in 0..2 {
        session.execute(Command::History(History::Undo));
    }
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(sibling).unwrap().point.position,
        Scalar(55)
    );
    session.execute(Command::Motion(Motion::BufferEnd));
    assert!(session.frame().unwrap().cursor().is_some());
}
