use super::*;
use rev::command::{BufferCommand, History, Motion};

fn begin(session: &mut Session, needle: &str, replacement: &str) {
    session.execute(Command::QueryReplace);
    query(session, needle);
    key(session, KeyCode::Enter, KeyModifiers::NONE);
    query(session, replacement);
    key(session, KeyCode::Enter, KeyModifiers::NONE);
}

fn answer(session: &mut Session, value: char) {
    key(session, KeyCode::Char(value), KeyModifiers::NONE);
}

// The real query/argument/decision flow replaces only a confirmed occurrence;
// skip and quit retain source bytes, and undo reverses that one accepted change.
#[test]
fn confirmed_flow() {
    let mut session = editor("cat cat cat", 0);
    let stamp = session.buffer().unwrap().stamp();
    begin(&mut session, "cat", "界");
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.frame().unwrap().rows()[0].search(), Some(6..9));
    answer(&mut session, 'y');
    assert_eq!(text(&session), "界 cat cat");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    answer(&mut session, 'n');
    assert_eq!(text(&session), "界 cat cat");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    answer(&mut session, 'q');
    assert_eq!(text(&session), "界 cat cat");
    assert_eq!(session.echo(), "Quit");
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.search().is_none())
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "cat cat cat");
    assert!(!session.buffer().unwrap().is_modified());
}

// Literal replacement text is never interpreted or searched again. Each answer
// publishes its own transaction, including when inserted text contains the query.
#[test]
fn literal_progress() {
    let mut session = editor("a a", 0);
    begin(&mut session, "a", "a$a");
    answer(&mut session, 'y');
    assert_eq!(text(&session), "a$a a");
    answer(&mut session, 'y');
    assert_eq!(text(&session), "a$a a$a");
    assert_eq!(session.echo(), "Query replace complete: 2 replaced");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "a$a a");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "a a");
}

// Empty queries reject before replacement input. An absent literal query retires
// both argument stages without moving point or publishing a history transaction.
#[test]
fn empty_absent() {
    let mut session = editor("source", 2);
    let origin = session.view().clone();
    session.execute(Command::QueryReplace);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(session.echo(), "Query must not be empty");
    assert_eq!(session.view(), &origin);
    begin(&mut session, "absent", "replacement");
    assert_eq!(session.echo(), "Query replace complete: 0 replaced");
    assert_eq!(session.view(), &origin);
    assert_eq!(text(&session), "source");
    assert!(session.buffer().unwrap().history().is_empty());
}

// Different replacement lengths and literal controls consume only original
// occurrences. Each undo/redo traverses exactly one accepted transaction.
#[test]
fn lengths() {
    for (name, source, needle, replacement, expected, count) in [
        ("delete", "x-x", "x", "", "-", 2),
        ("shorten", "word word", "word", "X", "X X", 2),
        ("grow", "x x", "x", "long", "long long", 2),
        ("equal", "x x", "x", "x", "x x", 2),
        (
            "joining",
            "e\u{301} e\u{301}",
            "e",
            "界",
            "界\u{301} 界\u{301}",
            2,
        ),
        (
            "controls",
            "x x",
            "x",
            "$1\x18\x03",
            "$1\x18\x03 $1\x18\x03",
            2,
        ),
        ("fold", "Straße STRASSE", "strasse", "é", "é é", 2),
    ] {
        let mut session = editor(source, 0);
        begin(&mut session, needle, replacement);
        for _ in 0..count {
            answer(&mut session, 'y');
        }
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), count, "{name}");
        for _ in 0..count {
            session.execute(Command::History(History::Undo));
        }
        assert_eq!(text(&session), source, "{name}");
        assert!(!session.buffer().unwrap().is_modified(), "{name}");
        for _ in 0..count {
            session.execute(Command::History(History::Redo));
        }
        assert_eq!(text(&session), expected, "{name}");
    }
}

// Multiline query/replacement paste preserves supplied LF/CRLF, emoji, tabs and
// combining marks; skipping the second occurrence cannot normalize its bytes.
#[test]
fn unicode_lines() {
    let source = "e\u{301}\t界\r\nend e\u{301}\t界\r\nend";
    let mut session = editor(source, 0);
    begin(&mut session, "界\r\nend", "👩\u{200d}💻\n\té\r\n");
    assert!(session.frame().unwrap().rows()[0].search().is_some());
    answer(&mut session, 'y');
    answer(&mut session, 'n');
    assert_eq!(
        text(&session),
        "e\u{301}\t👩\u{200d}💻\n\té\r\n e\u{301}\t界\r\nend"
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session).as_bytes(), source.as_bytes());
}

// Canonical publication maps other views of the same buffer and leaves a pane
// showing a different buffer untouched. Marks and per-occurrence undo stay valid.
#[test]
fn view_scope() {
    for shared in [false, true] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("cat cat\nlast").unwrap();
        let other_id = buffers.create("other buffer").unwrap();
        let label = buffers.get(other_id).unwrap().name().to_string();
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 10,
            },
        )
        .unwrap();
        session.execute(Command::Split(Split::Right));
        let left = session.windows().selected_id();
        let right = session.windows().order()[1];
        session.select_view(right).unwrap();
        if shared {
            session.execute(Command::Motion(Motion::BufferEnd));
        } else {
            session.execute(Command::Buffer(BufferCommand::Switch));
            query(&mut session, &label);
            key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        }
        let before = session.windows().get(right).unwrap().clone();
        session.select_view(left).unwrap();
        session.execute(Command::Region(RegionCommand::Set));
        begin(&mut session, "cat", "界");
        answer(&mut session, 'y');
        answer(&mut session, 'n');
        assert_eq!(text(&session), "界 cat\nlast");
        assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(0)));
        let other = session.windows().get(right).unwrap();
        if shared {
            assert_eq!(other.point.position.0, before.point.position.0 - 2);
        } else {
            assert_eq!(other, &before);
        }
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "cat cat\nlast");
        assert_eq!(
            session.windows().get(right).unwrap().point.position,
            before.point.position
        );
    }
}

fn stage(session: &mut Session, name: &str) {
    session.execute(Command::QueryReplace);
    query(session, "cat");
    if name == "query" {
        return;
    }
    key(session, KeyCode::Enter, KeyModifiers::NONE);
    query(session, "X");
    if name == "replacement" {
        return;
    }
    key(session, KeyCode::Enter, KeyModifiers::NONE);
    if name == "accepted" {
        answer(session, 'y');
    }
}

// Cancellation at either argument and before/after an accepted occurrence
// retires pending authority while keeping accepted changes and current point.
#[test]
fn cancel_stages() {
    for (name, expected, edited, count) in [
        ("query", "cat cat", "!cat cat", 0),
        ("replacement", "cat cat", "!cat cat", 0),
        ("decision", "cat cat", "cat! cat", 0),
        ("accepted", "X cat", "X cat!", 1),
    ] {
        let mut session = editor("cat cat", 0);
        stage(&mut session, name);
        let point = session.view().point;
        control(&mut session, 'g');
        assert_eq!(session.view().point, point, "{name}");
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), count, "{name}");
        answer(&mut session, '!');
        assert_eq!(text(&session), edited, "{name}");
    }
}

fn phased(session: &mut Session, value: char, kind: crossterm::event::KeyEventKind) -> Action {
    session.handle(Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char(value),
        KeyModifiers::NONE,
        kind,
    )))
}

// Neither paste, default Return nor held/released answers can grant consent or
// reach the next occurrence. A fresh answer is required for each transaction.
#[test]
fn fresh_consent() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let mut session = editor("cat cat", 0);
    begin(&mut session, "cat", "X");
    query(&mut session, "y");
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert!(session.echo().contains("Replace occurrence? [y/n/q]"));
    for phase in [Repeat, Release] {
        assert_eq!(phased(&mut session, 'y', phase), Action::Wait);
    }
    assert_eq!(text(&session), "cat cat");
    answer(&mut session, 'y');
    for phase in [Repeat, Release] {
        assert_eq!(phased(&mut session, 'y', phase), Action::Wait);
    }
    assert_eq!(text(&session), "X cat");
    answer(&mut session, 'n');
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(phased(&mut session, 'n', Repeat), Action::Wait);
    assert_eq!(text(&session), "X cat");
    assert_eq!(session.echo(), "Query replace complete: 1 replaced");
}

// A native publication between argument/decision and answer invalidates its
// captured revision. The stale answer cannot create a replacement transaction.
#[test]
fn source_change() {
    for name in ["query", "decision"] {
        let mut session = editor("cat cat", 0);
        stage(&mut session, name);
        session.replace(0..0, "Z").unwrap();
        let point = session.view().point;
        if name == "query" {
            key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        } else {
            answer(&mut session, 'y');
        }
        assert_eq!(session.echo(), "Query replace source changed", "{name}");
        assert_eq!(text(&session), "Zcat cat", "{name}");
        assert_eq!(session.view().point, point, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .all(|row| row.search().is_none())
        );
        answer(&mut session, 'y');
        assert_eq!(text(&session), "Zycat cat", "{name}");
    }
}

fn open(session: &mut Session, named: bool) {
    if named {
        session.execute(Command::ExecuteExtended);
        query(session, "query-replace");
        key(session, KeyCode::Enter, KeyModifiers::NONE);
    } else {
        key(session, KeyCode::Char('%'), KeyModifiers::ALT);
    }
}

// Bound and named entry both refuse a read-only source before taking arguments,
// preserving its point, state, stored mark and empty history.
#[test]
fn read_only() {
    for named in [false, true] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("cat cat").unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_mark(Some(Scalar(4))).unwrap();
        buffer.set_readonly(true);
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
        let origin = session.view().point;
        let state = session.buffer().unwrap().state();
        open(&mut session, named);
        assert!(session.echo().contains("read-only"));
        assert_eq!(session.view().point, origin);
        let buffer = session.buffer().unwrap();
        assert_eq!(buffer.state(), state);
        assert_eq!(buffer.mark(), Some(Scalar(4)));
        assert!(buffer.history().is_empty());
        assert_eq!(text(&session), "cat cat");
    }
}

// Oversized query paste rejects wholly and remains editable. NUL replacement
// rejection closes the operation without inserting binary data or losing source.
#[test]
fn argument_bounds() {
    let mut session = editor("cat", 0);
    open(&mut session, false);
    query(&mut session, &"x".repeat(4096));
    query(&mut session, "界");
    assert_eq!(session.echo(), "Prompt input limit exceeded");
    key(&mut session, KeyCode::Backspace, KeyModifiers::NONE);
    query(&mut session, "z");
    assert_eq!(session.echo(), "");
    control(&mut session, 'g');
    begin(&mut session, "cat", "bad\0text");
    assert_eq!(
        session.echo(),
        "Replacement contains NUL; binary input rejected"
    );
    assert_eq!(text(&session), "cat");
    assert!(session.buffer().unwrap().history().is_empty());
}
