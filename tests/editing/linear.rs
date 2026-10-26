use rev::command::{Command, History, LinearCommand, Motion, RegionCommand};
use rev::coordinates::Scalar;
use rev::session::{Action, Session};

fn linear(session: &mut Session, command: LinearCommand) {
    assert_eq!(session.execute(Command::Linear(command)), Action::Redraw);
}

fn whole_region(session: &mut Session, backward: bool) {
    session.execute(Command::Motion(if backward {
        Motion::BufferEnd
    } else {
        Motion::BufferStart
    }));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(if backward {
        Motion::BufferStart
    } else {
        Motion::BufferEnd
    }));
}

fn entries(session: &Session) -> Vec<String> {
    session.kill_ring().map(ToString::to_string).collect()
}

// Forward/reversed copies retain exact bytes without changing text, selection,
// point, revision, dirty state or history, including on a read-only source.
#[test]
fn copy_region() {
    let source = "αe\u{301}\r\n界👩\u{200d}💻";
    for (backward, readonly) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut session = super::editor(source, 0, readonly);
        whole_region(&mut session, backward);
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        linear(&mut session, LinearCommand::Copy);
        assert_eq!(session.echo(), "Region copied");
        assert_eq!(
            super::snapshot(session.buffer().unwrap(), &[session.view().point]),
            before
        );
        assert!(session.buffer().unwrap().mark_active());
        assert_eq!(entries(&session), [source]);
    }
}

// Killing either endpoint order removes exactly the region as one reversible
// edit, retains owned original bytes and deactivates only after publication.
#[test]
fn kill_region() {
    let source = "αe\u{301}\r\n界👩\u{200d}💻";
    for backward in [false, true] {
        let mut session = super::editor(source, 0, false);
        whole_region(&mut session, backward);
        let before = session.view().point.position;
        linear(&mut session, LinearCommand::Kill);
        assert_eq!(super::text(session.buffer().unwrap()), "");
        assert_eq!(session.view().point.position, Scalar(0));
        assert!(!session.buffer().unwrap().mark_active());
        assert_eq!(entries(&session), [source]);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(super::text(session.buffer().unwrap()), source);
        assert_eq!(session.view().point.position, before);
        assert_eq!(entries(&session), [source]);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// A multiline yank takes exactly one revision and one native replacement,
// separate from adjacent typing. Undo/redo removes/restores the whole fragment.
#[test]
fn atomic_yank() {
    let source = "ab\r\n界e\u{301}";
    let mut session = super::editor(source, 0, false);
    whole_region(&mut session, false);
    linear(&mut session, LinearCommand::Copy);
    session.execute(Command::Motion(Motion::BufferStart));
    session.insert('>');
    let before = session.buffer().unwrap().stamp();
    linear(&mut session, LinearCommand::Yank);
    let buffer = session.buffer().unwrap();
    assert_eq!(super::text(buffer), format!(">{source}{source}"));
    assert_eq!(buffer.stamp().revision, before.revision + 1);
    assert_eq!(buffer.history().len(), 2);
    let changes = buffer.history().back().unwrap().changes();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].before(), 1..1);
    assert_eq!(changes[0].inserted().to_string(), source);
    assert_eq!(
        session.view().point.position,
        Scalar(1 + source.chars().count())
    );
    assert!(!buffer.mark_active());
    session.execute(Command::History(History::Undo));
    assert_eq!(super::text(session.buffer().unwrap()), format!(">{source}"));
    session.execute(Command::History(History::Redo));
    assert_eq!(
        super::text(session.buffer().unwrap()),
        format!(">{source}{source}")
    );
    assert_eq!(entries(&session), [source]);
}

// Empty-ring yank is a visible refusal, preserving a live selection and all
// document state rather than silently inserting or recording an empty edit.
#[test]
fn empty_ring() {
    let mut session = super::editor("keep", 2, false);
    session.execute(Command::Region(RegionCommand::Set));
    let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
    linear(&mut session, LinearCommand::Yank);
    assert_eq!(session.echo(), "Kill ring is empty");
    assert_eq!(
        super::snapshot(session.buffer().unwrap(), &[session.view().point]),
        before
    );
    assert!(session.buffer().unwrap().mark_active());
    assert_eq!(session.kill_ring().len(), 0);
}
