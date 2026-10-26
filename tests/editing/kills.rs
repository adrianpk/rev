use rev::command::{Command, History, KillCommand};
use rev::coordinates::Scalar;
use rev::session::{Action, Session};

fn kill(session: &mut Session, command: KillCommand) {
    assert_eq!(session.execute(Command::Kill(command)), Action::Redraw);
}

fn retained(session: &Session) -> Option<String> {
    session.killed_text().map(ToString::to_string)
}

// Kill-line stops before the separator until point is at EOL. Its next call
// removes the complete logical separator, retaining LF/CRLF bytes unchanged.
#[test]
fn line_separators() {
    for newline in ["\n", "\r\n"] {
        let source = format!("ab界{newline}last");
        let mut session = super::editor(&source, 1, false);
        kill(&mut session, KillCommand::Line);
        assert_eq!(
            super::text(session.buffer().unwrap()),
            format!("a{newline}last")
        );
        assert_eq!(retained(&session).as_deref(), Some("b界"));
        kill(&mut session, KillCommand::Line);
        assert_eq!(super::text(session.buffer().unwrap()), "alast");
        assert_eq!(retained(&session), Some(format!("b界{newline}")));
        assert_eq!(session.view().point.position, Scalar(1));
        assert_eq!(session.buffer().unwrap().history().len(), 2);
        session.execute(Command::History(History::Undo));
        assert_eq!(
            super::text(session.buffer().unwrap()),
            format!("a{newline}last")
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(super::text(session.buffer().unwrap()), source);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// Word kills share movement's grapheme-aware Unicode word policy: skip whole
// separators and remove one word run, never splitting an accent or ZWJ emoji.
#[test]
fn word_ranges() {
    for (source, position, command, remaining, removed, point) in [
        (
            "...e\u{301}界_2!",
            0,
            KillCommand::Word,
            "!",
            "...e\u{301}界_2",
            0,
        ),
        (
            "é👩\u{200d}💻—界",
            1,
            KillCommand::Word,
            "é",
            "👩\u{200d}💻—界",
            1,
        ),
        ("a\r\n界!", 1, KillCommand::Word, "a!", "\r\n界", 1),
        ("é_2...", 6, KillCommand::BackwardWord, "", "é_2...", 0),
        (
            "a!e\u{301}",
            4,
            KillCommand::BackwardWord,
            "a!",
            "e\u{301}",
            2,
        ),
        (
            "é👩\u{200d}💻—界",
            5,
            KillCommand::BackwardWord,
            "界",
            "é👩\u{200d}💻—",
            0,
        ),
    ] {
        let mut session = super::editor(source, position, false);
        kill(&mut session, command);
        assert_eq!(
            super::text(session.buffer().unwrap()),
            remaining,
            "{source}"
        );
        assert_eq!(retained(&session).as_deref(), Some(removed), "{source}");
        assert_eq!(session.view().point.position, Scalar(point), "{source}");
        session.execute(Command::History(History::Undo));
        assert_eq!(super::text(session.buffer().unwrap()), source);
        assert_eq!(session.view().point.position, Scalar(position));
    }
}

// Forward fragments append and backward fragments prepend, even when the
// direction changes without a command or cursor discontinuity between kills.
#[test]
fn directional_text() {
    for (position, commands, remaining, removed) in [
        (0, [KillCommand::Word, KillCommand::Word], " cc", "aa bb"),
        (
            8,
            [KillCommand::BackwardWord, KillCommand::BackwardWord],
            "aa ",
            "bb cc",
        ),
        (
            3,
            [KillCommand::Word, KillCommand::BackwardWord],
            " cc",
            "aa bb",
        ),
        (
            5,
            [KillCommand::BackwardWord, KillCommand::Word],
            "aa ",
            "bb cc",
        ),
    ] {
        let mut session = super::editor("aa bb cc", position, false);
        for command in commands {
            kill(&mut session, command);
        }
        assert_eq!(retained(&session).as_deref(), Some(removed));
        assert_eq!(super::text(session.buffer().unwrap()), remaining);
        assert_eq!(session.buffer().unwrap().history().len(), 2);
    }
}

// Retained-text coalescing does not group history: each kill undoes alone,
// adjacent typing remains separate, and redo restores each exact edit.
#[test]
fn separate_history() {
    let mut session = super::editor("aa bb cc", 0, false);
    session.insert('X');
    kill(&mut session, KillCommand::Word);
    kill(&mut session, KillCommand::Word);
    assert_eq!(retained(&session).as_deref(), Some("aa bb"));
    assert_eq!(session.buffer().unwrap().history().len(), 3);
    for expected in ["X bb cc", "Xaa bb cc", "aa bb cc"] {
        session.execute(Command::History(History::Undo));
        assert_eq!(super::text(session.buffer().unwrap()), expected);
    }
    for expected in ["Xaa bb cc", "X bb cc", "X cc"] {
        session.execute(Command::History(History::Redo));
        assert_eq!(super::text(session.buffer().unwrap()), expected);
    }
}
