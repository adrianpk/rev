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
