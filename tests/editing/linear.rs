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

// Partial regions in both endpoint orders retain only the selected Unicode
// graphemes; kill preserves adjacent text and copy preserves the complete source.
#[test]
fn partial_regions() {
    for exchange in [false, true] {
        for command in [LinearCommand::Copy, LinearCommand::Kill] {
            let mut session = super::editor("ab界e\u{301}z", 2, false);
            session.execute(Command::Region(RegionCommand::Set));
            session.execute(Command::Motion(Motion::ForwardChar));
            session.execute(Command::Motion(Motion::ForwardChar));
            if exchange {
                session.execute(Command::Region(RegionCommand::Exchange));
            }
            linear(&mut session, command);
            assert_eq!(entries(&session), ["界e\u{301}"]);
            let killed = command == LinearCommand::Kill;
            assert_eq!(
                super::text(session.buffer().unwrap()),
                if killed { "abz" } else { "ab界e\u{301}z" }
            );
            assert_eq!(
                session.buffer().unwrap().history().len(),
                usize::from(killed)
            );
        }
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

// New copies evict only the oldest entry at the fixed bound. Later source
// replacement cannot change any retained fragment or its newest-first order.
#[test]
fn ring_eviction() {
    let mut session = super::editor("seed", 0, false);
    for index in 0..35 {
        let len = session.buffer().unwrap().len_chars();
        session.replace(0..len, &index.to_string()).unwrap();
        whole_region(&mut session, false);
        linear(&mut session, LinearCommand::Copy);
        assert_eq!(session.kill_ring().len(), (index + 1).min(32));
    }
    let expected: Vec<_> = (3..35).rev().map(|index| index.to_string()).collect();
    assert_eq!(entries(&session), expected);
    session.replace(0..2, "changed source").unwrap();
    assert_eq!(entries(&session), expected);
}

// Coalescing replaces the newest entry without spending another slot. Undo
// leaves ring contents intact and breaks the chain before a subsequent kill.
#[test]
fn bounded_coalescing() {
    use rev::command::KillCommand;
    let mut session = super::editor("aa bb cc", 0, false);
    whole_region(&mut session, false);
    for _ in 0..32 {
        linear(&mut session, LinearCommand::Copy);
    }
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Kill(KillCommand::Word));
    session.execute(Command::Kill(KillCommand::Word));
    assert_eq!(session.kill_ring().len(), 32);
    assert_eq!(&entries(&session)[..3], ["aa bb", "aa bb cc", "aa bb cc"]);
    session.execute(Command::History(History::Undo));
    session.execute(Command::Kill(KillCommand::Word));
    assert_eq!(session.kill_ring().len(), 32);
    assert_eq!(&entries(&session)[..3], [" bb", "aa bb", "aa bb cc"]);
}

// Missing/inactive and empty active regions cannot consume text, add an empty
// entry, evict prior text or deactivate the current selection on refusal.
#[test]
fn absent_regions() {
    for state in ["missing", "inactive", "empty"] {
        let mut session = super::editor("abc", 0, false);
        if state != "missing" {
            whole_region(&mut session, false);
            linear(&mut session, LinearCommand::Copy);
            session.execute(Command::KeyboardQuit);
        }
        if state == "empty" {
            session.execute(Command::Region(RegionCommand::Set));
        }
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        let saved = entries(&session);
        for command in [LinearCommand::Copy, LinearCommand::Kill] {
            linear(&mut session, command);
            assert!(!session.echo().is_empty());
            assert_eq!(
                super::snapshot(session.buffer().unwrap(), &[session.view().point]),
                before
            );
            assert_eq!(session.buffer().unwrap().mark_active(), state == "empty");
            assert_eq!(entries(&session), saved);
        }
    }
}

// Copy is permitted on read-only text; subsequent kill/yank refusals preserve
// that existing entry and the active source selection without recording edits.
#[test]
fn readonly_ring() {
    let mut session = super::editor("α\r\n界", 0, true);
    whole_region(&mut session, false);
    linear(&mut session, LinearCommand::Copy);
    let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
    for command in [LinearCommand::Kill, LinearCommand::Yank] {
        linear(&mut session, command);
        assert!(session.echo().contains("read-only"));
        assert_eq!(
            super::snapshot(session.buffer().unwrap(), &[session.view().point]),
            before
        );
        assert!(session.buffer().unwrap().mark_active());
        assert_eq!(entries(&session), ["α\r\n界"]);
    }
}

// A non-mutating copy after undo preserves the conventional redo future and
// does not disguise the old logical save state as a newly edited document.
#[test]
fn copy_redo() {
    let mut session = super::editor("abc", 0, false);
    session.insert('X');
    session.execute(Command::History(History::Undo));
    whole_region(&mut session, false);
    linear(&mut session, LinearCommand::Copy);
    assert_eq!(session.buffer().unwrap().redo_history().len(), 1);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(super::text(session.buffer().unwrap()), "Xabc");
    assert_eq!(entries(&session), ["abc"]);
}

// Shared views see one insertion and one position map: the initiator ends at
// the inserted fragment, its sibling shifts independently, and undo restores it.
#[test]
fn shared_yank() {
    use rev::command::Split;
    let mut session = super::editor("ab", 0, false);
    whole_region(&mut session, false);
    linear(&mut session, LinearCommand::Copy);
    session.execute(Command::Split(Split::Right));
    let sibling = session.windows().order()[1];
    session.execute(Command::Motion(Motion::BufferStart));
    linear(&mut session, LinearCommand::Yank);
    assert_eq!(super::text(session.buffer().unwrap()), "abab");
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(
        session.windows().get(sibling).unwrap().point.position,
        Scalar(4)
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(0)));
    assert!(!session.buffer().unwrap().mark_active());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(super::text(session.buffer().unwrap()), "ab");
    assert_eq!(session.view().point.position, Scalar(0));
    assert_eq!(
        session.windows().get(sibling).unwrap().point.position,
        Scalar(2)
    );
    assert_eq!(entries(&session), ["ab"]);
}

fn buffer_argument(session: &mut Session, command: rev::command::BufferCommand, name: &str) {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    session.execute(Command::Buffer(command));
    for character in name.chars() {
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char(character),
            KeyModifiers::NONE,
        )));
    }
    session.handle(super::press(KeyCode::Enter));
}

// Entries belong to the session, not the source buffer: they survive closing
// it and can be yanked into another canonical buffer, but never another session.
#[test]
fn closed_source() {
    use rev::buffer::Buffers;
    use rev::command::BufferCommand;
    use rev::view::{Size, View};
    let mut buffers = Buffers::new().unwrap();
    let source = buffers.create("α\r\n界").unwrap();
    let target = buffers.create("tail").unwrap();
    let source_name = buffers.get(source).unwrap().name().to_owned();
    let target_name = buffers.get(target).unwrap().name().to_owned();
    let view = View::new(buffers.get(source).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap();
    whole_region(&mut session, true);
    linear(&mut session, LinearCommand::Copy);
    buffer_argument(&mut session, BufferCommand::Switch, &target_name);
    buffer_argument(&mut session, BufferCommand::Kill, &source_name);
    assert_eq!(session.echo(), "Buffer closed");
    linear(&mut session, LinearCommand::Yank);
    assert_eq!(session.buffer().unwrap().id(), target);
    assert_eq!(super::text(session.buffer().unwrap()), "α\r\n界tail");
    assert_eq!(entries(&session), ["α\r\n界"]);
    let fresh = super::editor("fresh", 0, false);
    assert_eq!(fresh.kill_ring().len(), 0);
}

// Existing ring data and a document selection survive native linear commands
// while argument, inventory or save/discard confirmation owns input focus.
#[test]
fn focused_ring() {
    use crossterm::event::KeyCode;
    use rev::command::BufferCommand;
    for command in [
        Command::FindFile,
        Command::Buffer(BufferCommand::List),
        Command::Buffer(BufferCommand::Kill),
    ] {
        let mut session = super::editor("abc", 0, false);
        whole_region(&mut session, false);
        linear(&mut session, LinearCommand::Copy);
        session.insert('X');
        whole_region(&mut session, false);
        session.execute(command);
        if command == Command::Buffer(BufferCommand::Kill) {
            session.handle(super::press(KeyCode::Enter));
        }
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        for command in [
            LinearCommand::Copy,
            LinearCommand::Kill,
            LinearCommand::Yank,
        ] {
            linear(&mut session, command);
            assert!(session.echo().contains("owns input focus"));
            assert_eq!(
                super::snapshot(session.buffer().unwrap(), &[session.view().point]),
                before
            );
            assert!(session.buffer().unwrap().mark_active());
            assert_eq!(entries(&session), ["abc"]);
        }
    }
}

// Owned fragments spanning many Rope chunks preserve multiline Unicode bytes
// through copy, kill, yank and each standalone undo without partial insertion.
#[test]
fn chunk_yank() {
    let source = "e\u{301}界👩\u{200d}💻\r\n".repeat(500) + "tail";
    let mut session = super::editor(&source, 0, false);
    whole_region(&mut session, false);
    linear(&mut session, LinearCommand::Copy);
    linear(&mut session, LinearCommand::Kill);
    linear(&mut session, LinearCommand::Yank);
    assert_eq!(super::text(session.buffer().unwrap()), source);
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()
            .len(),
        1
    );
    assert_eq!(session.kill_ring().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(super::text(session.buffer().unwrap()), "");
    session.execute(Command::History(History::Undo));
    assert_eq!(super::text(session.buffer().unwrap()), source);
    assert!(!session.buffer().unwrap().is_modified());
}
