use rev::command::{Command, History, KillCommand, Motion, RegionCommand, Split};
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

// Empty ranges are true boundary no-ops: no text/history/dirty/selection change
// and no replacement of text retained by an earlier successful kill.
#[test]
fn boundary_kills() {
    for (source, position, command) in [
        ("", 0, KillCommand::Line),
        ("", 0, KillCommand::Word),
        ("", 0, KillCommand::BackwardWord),
        ("a", 1, KillCommand::Line),
        ("a", 1, KillCommand::Word),
        ("a", 0, KillCommand::BackwardWord),
    ] {
        let mut session = super::editor(source, position, false);
        session.execute(Command::Region(RegionCommand::Set));
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        kill(&mut session, command);
        assert_eq!(session.echo(), "Buffer boundary");
        assert_eq!(
            super::snapshot(session.buffer().unwrap(), &[session.view().point]),
            before
        );
        assert!(session.buffer().unwrap().mark_active());
        assert_eq!(retained(&session), None);
    }
    let mut session = super::editor("aa", 0, false);
    kill(&mut session, KillCommand::Word);
    kill(&mut session, KillCommand::Line);
    assert_eq!(retained(&session).as_deref(), Some("aa"));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Every kill obeys read-only admission without consuming an active region or
// publishing a retained fragment, revision, point, mark or history change.
#[test]
fn readonly_kills() {
    for command in [
        KillCommand::Line,
        KillCommand::Word,
        KillCommand::BackwardWord,
    ] {
        let mut session = super::editor("aa bb", 3, true);
        session.execute(Command::Region(RegionCommand::Set));
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        kill(&mut session, command);
        assert!(session.echo().contains("read-only"));
        assert_eq!(
            super::snapshot(session.buffer().unwrap(), &[session.view().point]),
            before
        );
        assert!(session.buffer().unwrap().mark_active());
        assert_eq!(retained(&session), None);
    }
}

// Non-kill commands end continuity even when point returns to the same scalar
// or the command changes only interaction state rather than document bytes.
#[test]
fn command_breaks() {
    for commands in [
        [
            Command::Motion(Motion::ForwardChar),
            Command::Motion(Motion::BackwardChar),
        ],
        [Command::Region(RegionCommand::Set), Command::KeyboardQuit],
        [
            Command::History(History::Undo),
            Command::History(History::Redo),
        ],
        [Command::FindFile, Command::KeyboardQuit],
        [Command::KeyboardQuit, Command::KeyboardQuit],
    ] {
        let mut session = super::editor("aa bb cc", 0, false);
        kill(&mut session, KillCommand::Word);
        for command in commands {
            session.execute(command);
        }
        kill(&mut session, KillCommand::Word);
        assert_eq!(retained(&session).as_deref(), Some(" bb"));
        assert_eq!(super::text(session.buffer().unwrap()), " cc");
    }
}

// Input notices, prefixes and native edits also break continuity. A no-op
// replacement still counts as an intervening non-kill native edit request.
#[test]
fn input_breaks() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    for event in [
        Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT)),
        super::chord('x'),
    ] {
        let mut session = super::editor("aa bb cc", 0, false);
        kill(&mut session, KillCommand::Word);
        session.handle(event);
        kill(&mut session, KillCommand::Word);
        assert_eq!(retained(&session).as_deref(), Some(" bb"));
    }
    let mut session = super::editor("aa bb cc", 0, false);
    kill(&mut session, KillCommand::Word);
    assert!(!session.replace(0..0, "").unwrap());
    kill(&mut session, KillCommand::Word);
    assert_eq!(retained(&session).as_deref(), Some(" bb"));
    session.insert('X');
    session.execute(Command::BackwardDeleteChar);
    kill(&mut session, KillCommand::Word);
    assert_eq!(retained(&session).as_deref(), Some(" cc"));
}

// Shared points and stored mark use the existing checked edit map; successful
// kills deactivate selection and undo restores exactly the killed fragment.
#[test]
fn shared_kills() {
    let mut session = super::editor("aa bb cc", 3, false);
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Split(Split::Right));
    let sibling = session.windows().order()[1];
    session.execute(Command::Motion(Motion::BackwardWord));
    session.execute(Command::Motion(Motion::BackwardWord));
    kill(&mut session, KillCommand::Word);
    assert_eq!(super::text(session.buffer().unwrap()), "aa  cc");
    assert_eq!(session.view().point.position, Scalar(3));
    assert_eq!(
        session.windows().get(sibling).unwrap().point.position,
        Scalar(6)
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(3)));
    assert!(!session.buffer().unwrap().mark_active());
    session.execute(Command::History(History::Undo));
    assert_eq!(super::text(session.buffer().unwrap()), "aa bb cc");
    assert_eq!(
        session.windows().get(sibling).unwrap().point.position,
        Scalar(8)
    );
    assert!(!session.buffer().unwrap().mark_active());
}

// Leaving and returning to the same live view cannot silently resume its chain,
// even if both identities and the qualified point again match the last kill.
#[test]
fn view_breaks() {
    let mut session = super::editor("aa bb cc", 0, false);
    session.execute(Command::Split(Split::Right));
    let first = session.windows().selected_id();
    let second = session.windows().order()[1];
    kill(&mut session, KillCommand::Word);
    session.select_view(second).unwrap();
    session.select_view(first).unwrap();
    kill(&mut session, KillCommand::Word);
    assert_eq!(retained(&session).as_deref(), Some(" bb"));
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}

// A transient owner refuses all native kill commands, retaining its active
// document selection, qualified point, text and history throughout cancellation.
#[test]
fn focused_kills() {
    use crossterm::event::KeyCode;
    use rev::command::BufferCommand;
    for command in [
        Command::FindFile,
        Command::Buffer(BufferCommand::List),
        Command::Buffer(BufferCommand::Kill),
    ] {
        let mut session = super::editor("aa bb", 3, false);
        session.insert('X');
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(command);
        if command == Command::Buffer(BufferCommand::Kill) {
            session.handle(super::press(KeyCode::Enter));
        }
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        for command in [
            KillCommand::Line,
            KillCommand::Word,
            KillCommand::BackwardWord,
        ] {
            kill(&mut session, command);
            assert!(session.echo().contains("owns input focus"));
            assert_eq!(
                super::snapshot(session.buffer().unwrap(), &[session.view().point]),
                before
            );
            assert!(session.buffer().unwrap().mark_active());
            assert_eq!(retained(&session), None);
        }
        session.execute(Command::KeyboardQuit);
        assert!(!session.buffer().unwrap().mark_active());
    }
}

// Release events never kill, while an actual repeat of the same meta command
// remains a consecutive kill with its own independent undo record.
#[test]
fn kill_phases() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    let mut session = super::editor("aa bb cc", 0, false);
    for kind in [
        KeyEventKind::Press,
        KeyEventKind::Release,
        KeyEventKind::Repeat,
    ] {
        session.handle(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('d'),
            KeyModifiers::ALT,
            kind,
        )));
    }
    assert_eq!(super::text(session.buffer().unwrap()), " cc");
    assert_eq!(retained(&session).as_deref(), Some("aa bb"));
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}
