use rev::command::{Command, History, LinearCommand, Motion, RegionCommand};
use rev::coordinates::Scalar;
use rev::session::{Action, Session};

fn linear(session: &mut Session, command: LinearCommand) {
    assert_eq!(session.execute(Command::Linear(command)), Action::Redraw);
}

fn stocked(entries: &[&str], target: &str) -> Session {
    let mut session = super::editor("", 0, false);
    for entry in entries {
        let len = session.buffer().unwrap().len_chars();
        session.replace(0..len, entry).unwrap();
        session.execute(Command::Motion(Motion::BufferStart));
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::Motion(Motion::BufferEnd));
        linear(&mut session, LinearCommand::Copy);
        assert_eq!(session.echo(), "Region copied");
    }
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, target).unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session
}

fn rejected(session: &mut Session) {
    let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
    let entries: Vec<_> = session.kill_ring().map(ToString::to_string).collect();
    linear(session, LinearCommand::Pop);
    assert_eq!(session.echo(), "No current yank to replace");
    assert_eq!(
        super::snapshot(session.buffer().unwrap(), &[session.view().point]),
        before
    );
    assert_eq!(
        session
            .kill_ring()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        entries
    );
}

// Unequal-length rotations always replace the last inserted span, refresh its
// revision/range and wrap without reordering the ring or touching adjacent text.
#[test]
fn rotating_ranges() {
    let mut session = stocked(&["é\r\n界", "x", "longer"], "<>");
    session.execute(Command::Motion(Motion::ForwardChar));
    linear(&mut session, LinearCommand::Yank);
    let mut previous = "longer";
    for expected in ["x", "é\r\n界", "longer", "x"] {
        let stamp = session.buffer().unwrap().stamp();
        let history = session.buffer().unwrap().history().len();
        linear(&mut session, LinearCommand::Pop);
        let buffer = session.buffer().unwrap();
        assert_eq!(super::text(buffer), format!("<{expected}>"));
        assert_eq!(buffer.stamp().revision, stamp.revision + 1);
        assert_eq!(buffer.history().len(), history + 1);
        let changes = buffer.history().back().unwrap().changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].before(), 1..1 + previous.chars().count());
        assert_eq!(changes[0].removed().to_string(), previous);
        assert_eq!(changes[0].inserted().to_string(), expected);
        assert_eq!(
            session.view().point.position,
            Scalar(1 + expected.chars().count())
        );
        previous = expected;
    }
    assert_eq!(
        session
            .kill_ring()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["longer", "x", "é\r\n界"]
    );
}

// Each rotation is one coherent undo step. Undo/redo restores complete prior
// payloads and logical states but cannot revive the special replacement capture.
#[test]
fn sequence_history() {
    let mut session = stocked(&["small", "LONGER"], "ab");
    session.execute(Command::Motion(Motion::ForwardChar));
    let state = session.buffer().unwrap().state();
    linear(&mut session, LinearCommand::Yank);
    linear(&mut session, LinearCommand::Pop);
    linear(&mut session, LinearCommand::Pop);
    for expected in ["asmallb", "aLONGERb", "ab"] {
        session.execute(Command::History(History::Undo));
        assert_eq!(super::text(session.buffer().unwrap()), expected);
        rejected(&mut session);
    }
    assert_eq!(session.buffer().unwrap().state(), state);
    for expected in ["aLONGERb", "asmallb", "aLONGERb"] {
        session.execute(Command::History(History::Redo));
        assert_eq!(super::text(session.buffer().unwrap()), expected);
    }
    rejected(&mut session);
}

// A new yank restarts at the newest entry and owns only its new insertion;
// rotating it must retain text produced by an earlier yank sequence nearby.
#[test]
fn fresh_yank() {
    let mut session = stocked(&["old", "LONG"], "<>");
    session.execute(Command::Motion(Motion::ForwardChar));
    linear(&mut session, LinearCommand::Yank);
    linear(&mut session, LinearCommand::Pop);
    linear(&mut session, LinearCommand::Yank);
    assert_eq!(super::text(session.buffer().unwrap()), "<oldLONG>");
    linear(&mut session, LinearCommand::Pop);
    assert_eq!(super::text(session.buffer().unwrap()), "<oldold>");
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()[0]
            .before(),
        4..8
    );
}

// Joined clusters can move the cursor past the inserted bytes. Rotation must
// replace only those bytes, retaining a neighboring accent, LF or ZWJ target.
#[test]
fn joined_ranges() {
    for (newest, suffix) in [("e", "\u{301}z"), ("\r", "\nz"), ("👩\u{200d}", "💻z")] {
        let mut session = stocked(&["X", newest], suffix);
        let state = session.buffer().unwrap().state();
        linear(&mut session, LinearCommand::Yank);
        assert!(session.view().point.position.0 > newest.chars().count());
        assert_eq!(
            super::text(session.buffer().unwrap()),
            format!("{newest}{suffix}")
        );
        linear(&mut session, LinearCommand::Pop);
        assert_eq!(super::text(session.buffer().unwrap()), format!("X{suffix}"));
        assert_eq!(
            session
                .buffer()
                .unwrap()
                .history()
                .back()
                .unwrap()
                .changes()[0]
                .before(),
            0..newest.chars().count()
        );
        linear(&mut session, LinearCommand::Pop);
        assert_eq!(
            super::text(session.buffer().unwrap()),
            format!("{newest}{suffix}")
        );
        for _ in 0..3 {
            session.execute(Command::History(History::Undo));
        }
        assert_eq!(super::text(session.buffer().unwrap()), suffix);
        assert_eq!(session.buffer().unwrap().state(), state);
    }
}

// Ring contents alone grant no replacement authority, including an empty ring
// or a stored mark/active region without an immediately preceding successful yank.
#[test]
fn no_yank() {
    for entries in [&[][..], &["old", "new"][..]] {
        let mut session = stocked(entries, "keep");
        session.execute(Command::Region(RegionCommand::Set));
        rejected(&mut session);
        assert!(session.buffer().unwrap().mark_active());
    }
}

// Any unrelated command consumes replacement authority, even if it leaves text
// unchanged or returns point and document content to their previous positions.
#[test]
fn command_breaks() {
    use rev::command::WindowCommand;
    for commands in [
        [
            Command::Motion(Motion::ForwardChar),
            Command::Motion(Motion::BackwardChar),
        ],
        [Command::Region(RegionCommand::Set), Command::KeyboardQuit],
        [Command::FindFile, Command::KeyboardQuit],
        [Command::SaveBuffer, Command::KeyboardQuit],
        [Command::Window(WindowCommand::Other), Command::KeyboardQuit],
        [
            Command::History(History::Undo),
            Command::History(History::Redo),
        ],
    ] {
        let mut session = stocked(&["old", "new"], "<>");
        session.execute(Command::Motion(Motion::ForwardChar));
        linear(&mut session, LinearCommand::Yank);
        for command in commands {
            session.execute(command);
        }
        rejected(&mut session);
    }
}

// A prefix or unbound-key notice ends the immediate sequence before any later
// command completes; a failed pop must preserve both text and existing entries.
#[test]
fn input_breaks() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    for event in [
        super::chord('x'),
        Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT)),
    ] {
        let mut session = stocked(&["old", "new"], "<>");
        linear(&mut session, LinearCommand::Yank);
        session.handle(event);
        rejected(&mut session);
    }
}

// Native replacement requests invalidate even on no-op/failure. Typing and
// deleting it again cannot make an old inserted range current a second time.
#[test]
fn edit_breaks() {
    for kind in ["noop", "failed", "typing"] {
        let mut session = stocked(&["old", "new"], "<>");
        linear(&mut session, LinearCommand::Yank);
        match kind {
            "noop" => assert!(!session.replace(0..0, "").unwrap()),
            "failed" => assert!(session.replace(usize::MAX..usize::MAX, "X").is_err()),
            _ => {
                session.insert('X');
                session.execute(Command::BackwardDeleteChar);
            }
        }
        rejected(&mut session);
    }
}

// Native view selection invalidates across a round trip, with or without a
// shared-buffer edit in the sibling. Refusal cannot move either live point.
#[test]
fn view_breaks() {
    use rev::command::Split;
    for edited in [false, true] {
        let mut session = stocked(&["old", "new"], "<>");
        session.execute(Command::Split(Split::Right));
        let first = session.windows().selected_id();
        let second = session.windows().order()[1];
        linear(&mut session, LinearCommand::Yank);
        session.select_view(second).unwrap();
        if edited {
            session.replace(0..0, "X").unwrap();
        }
        session.select_view(first).unwrap();
        let points: Vec<_> = session
            .windows()
            .iter()
            .map(|(_, view)| view.point)
            .collect();
        rejected(&mut session);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(_, view)| view.point)
                .collect::<Vec<_>>(),
            points
        );
    }
}

// A successful pop publishes one mapped replacement for every shared view;
// the noninitiating point follows unequal lengths and undo restores that map.
#[test]
fn shared_rotation() {
    use rev::command::Split;
    let mut session = stocked(&["x", "longer"], "<>");
    session.execute(Command::Split(Split::Right));
    let first = session.windows().selected_id();
    let second = session.windows().order()[1];
    session.select_view(second).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.select_view(first).unwrap();
    session.execute(Command::Motion(Motion::ForwardChar));
    linear(&mut session, LinearCommand::Yank);
    assert_eq!(
        session.windows().get(second).unwrap().point.position,
        Scalar(8)
    );
    linear(&mut session, LinearCommand::Pop);
    assert_eq!(super::text(session.buffer().unwrap()), "<x>");
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(
        session.windows().get(second).unwrap().point.position,
        Scalar(3)
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(super::text(session.buffer().unwrap()), "<longer>");
    assert_eq!(
        session.windows().get(second).unwrap().point.position,
        Scalar(8)
    );
    rejected(&mut session);
}

// Loading another target invalidates before ingress, including ordinary failure.
// Closing the current buffer cannot transfer replacement authority to its survivor.
#[test]
fn target_breaks() {
    use rev::command::BufferCommand;
    use std::path::Path;
    let mut session = stocked(&["old", "new"], "<>");
    linear(&mut session, LinearCommand::Yank);
    assert!(session.load(Path::new("")).is_err());
    rejected(&mut session);
    linear(&mut session, LinearCommand::Yank);
    let previous = session.buffer().unwrap().id();
    session.execute(Command::Buffer(BufferCommand::Kill));
    session.handle(super::press(crossterm::event::KeyCode::Enter));
    session.handle(super::press(crossterm::event::KeyCode::Char('n')));
    assert_ne!(session.buffer().unwrap().id(), previous);
    rejected(&mut session);
}

// Release reports do not end a sequence, while actual repeated M-y commands
// rotate again and retain a separate coherent edit for each dispatch.
#[test]
fn rotation_phases() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    let mut session = stocked(&["old", "newest"], "<>");
    let count = session.buffer().unwrap().history().len();
    linear(&mut session, LinearCommand::Yank);
    for kind in [
        KeyEventKind::Press,
        KeyEventKind::Release,
        KeyEventKind::Repeat,
    ] {
        session.handle(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('y'),
            KeyModifiers::ALT,
            kind,
        )));
    }
    assert_eq!(super::text(session.buffer().unwrap()), "newest<>");
    assert_eq!(session.buffer().unwrap().history().len(), count + 3);
}

// Presentation suspension/restoration changes geometry without editing or
// changing targets, so it preserves an otherwise immediate rotation sequence.
#[test]
fn resize_sequence() {
    use crossterm::event::Event;
    let mut session = stocked(&["old", "newest"], "<>");
    linear(&mut session, LinearCommand::Yank);
    session.handle(Event::Resize(0, 0));
    assert!(session.frame().unwrap().cursor().is_none());
    linear(&mut session, LinearCommand::Pop);
    assert_eq!(super::text(session.buffer().unwrap()), "old<>");
    session.handle(Event::Resize(48, 8));
    assert!(session.frame().unwrap().cursor().is_some());
    linear(&mut session, LinearCommand::Pop);
    assert_eq!(super::text(session.buffer().unwrap()), "newest<>");
}

// The circular boundary works for a single entry and for the full retained
// capacity, including repeated passes through the oldest and newest entries.
#[test]
fn rotation_limits() {
    for count in [1, 32] {
        let owned: Vec<_> = (0..count).map(|index| format!("entry-{index}")).collect();
        let borrowed: Vec<_> = owned.iter().map(String::as_str).collect();
        let mut session = stocked(&borrowed, "[]");
        session.execute(Command::Motion(Motion::ForwardChar));
        linear(&mut session, LinearCommand::Yank);
        for expected in owned.iter().rev().cycle().skip(1).take(count * 2) {
            linear(&mut session, LinearCommand::Pop);
            assert_eq!(
                super::text(session.buffer().unwrap()),
                format!("[{expected}]")
            );
        }
        assert_eq!(session.kill_ring().len(), count);
    }
}
