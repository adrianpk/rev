use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, History, LinearCommand, Motion, RegionCommand};
use rev::coordinates::Scalar;
use rev::session::Session;
use rev::view::{Size, View};

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn character(value: char) -> Event {
    key(KeyCode::Char(value), KeyModifiers::NONE)
}
fn control(value: char) -> Event {
    key(KeyCode::Char(value), KeyModifiers::CONTROL)
}

fn editor(config: &str, source: &str, at: usize) -> Session {
    let fixture = super::config_fixture::Fixture::new();
    let prepared = rev::config::prepare(&fixture.write("config.toml", config));
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    let mut buffers = Buffers::with_settings(prepared.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = Scalar(at);
    Session::new(
        buffers,
        view,
        Size {
            width: 32,
            height: 10,
        },
    )
    .unwrap()
}

fn count(session: &mut Session, value: &str) {
    session.handle(control('u'));
    for value in value.chars() {
        session.handle(character(value));
    }
}

fn named(session: &mut Session, name: &str) {
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
}

fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while session
        .remaining_input(Instant::now())
        .is_some_and(|wait| wait == Duration::ZERO)
    {
        assert!(
            Instant::now() < deadline,
            "counted native work did not settle"
        );
        session.expire_input(Instant::now());
    }
}

fn text(session: &Session) -> String {
    super::text(session.buffer().unwrap())
}

fn stocked(entries: &[&str], target: &str) -> Session {
    stocked_with("", entries, target)
}

fn stocked_with(config: &str, entries: &[&str], target: &str) -> Session {
    let mut session = editor(config, "", 0);
    for entry in entries {
        let len = session.buffer().unwrap().len_chars();
        session.replace(0..len, entry).unwrap();
        session.execute(Command::Motion(Motion::BufferStart));
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::Motion(Motion::BufferEnd));
        session.execute(Command::Linear(LinearCommand::Copy));
    }
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, target).unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session
}

// Literal and paste counts replace the admitted region once in either endpoint
// order. One native group restores the original Unicode source and point.
#[test]
fn literal_regions() {
    for (name, paste, backward) in [
        ("character", false, false),
        ("paste", true, false),
        ("backward region", true, true),
    ] {
        let mut session = editor("", "aold界z", if backward { 4 } else { 1 });
        session.execute(Command::Region(RegionCommand::Set));
        for _ in 0..3 {
            session.execute(Command::Motion(if backward {
                Motion::BackwardChar
            } else {
                Motion::ForwardChar
            }));
        }
        count(&mut session, "3");
        if paste {
            session.handle(Event::Paste("界".into()));
        } else {
            session.handle(character('界'));
        }
        settle(&mut session);
        assert_eq!(text(&session), "a界界界界z", "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "aold界z", "{name}");
    }
}

// Neighboring typing remains independent from one counted publication, and
// UTF-8 copies are not interpreted as repeated key events or commands.
#[test]
fn literal_groups() {
    for paste in [false, true] {
        let mut session = editor("", "", 0);
        session.insert('a');
        count(&mut session, "3");
        if paste {
            session.handle(Event::Paste("é界\r\n".into()));
        } else {
            session.handle(character('界'));
        }
        settle(&mut session);
        let repeated = if paste {
            "é界\r\n".repeat(3)
        } else {
            "界".repeat(3)
        };
        assert_eq!(text(&session), format!("a{repeated}"));
        assert_eq!(session.buffer().unwrap().history().len(), 2);
        session.insert('z');
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), format!("a{repeated}"));
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "a");
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), format!("a{repeated}"));
    }
}

// A supplied zero, negative count, NUL or oversized aggregate never publishes
// source, point, mark, ring or history. A maximum admitted count remains usable.
#[test]
fn insertion_admission() {
    for (name, number, payload) in [
        ("zero", "0", "x".into()),
        ("negative", "-2", "x".into()),
        ("nul", "3", "x\0".into()),
        ("aggregate", "1000000", "123456789".into()),
    ] {
        let mut session = editor("", "abc", 1);
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        count(&mut session, number);
        session.handle(Event::Paste(payload));
        settle(&mut session);
        assert_eq!(
            super::snapshot(session.buffer().unwrap(), &[session.view().point]),
            before,
            "{name}"
        );
    }
    let mut session = editor("", "", 0);
    count(&mut session, "1000000");
    session.handle(character('x'));
    settle(&mut session);
    assert_eq!(session.buffer().unwrap().len_bytes(), 1_000_000);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "");
}

// Native CRLF and display-column tab policy compose into one edit, rather than
// repeating the first indentation width or creating one undo group per step.
#[test]
fn insertion_policy() {
    for (name, config, source, at, command, expected) in [
        ("LF", "", "ab\n", 1, "newline", "a\n\n\nb\n"),
        ("CRLF", "", "ab\r\n", 1, "newline", "a\r\n\r\n\r\nb\r\n"),
        (
            "spaces",
            "editor.indent_width = 4",
            "界x",
            1,
            "insert-tab",
            "界          x",
        ),
        (
            "tabs",
            "editor.indent_style = 'tabs'",
            "界x",
            1,
            "insert-tab",
            "界\t\t\tx",
        ),
    ] {
        let mut session = editor(config, source, at);
        count(&mut session, "3");
        named(&mut session, command);
        settle(&mut session);
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
    }
}

// Signed deletes traverse native graphemes and complete CRLF separators; a
// supplied zero preserves even an active region and creates no history.
#[test]
fn signed_deletion() {
    for (name, at, number, command, expected) in [
        ("forward", 0, "2", "delete-char", "\r\nz"),
        (
            "backward",
            8,
            "2",
            "backward-delete-char",
            "e\u{301}👩\u{200d}💻",
        ),
        (
            "reverse forward",
            8,
            "-2",
            "delete-char",
            "e\u{301}👩\u{200d}💻",
        ),
        ("reverse backward", 0, "-2", "backward-delete-char", "\r\nz"),
        ("zero", 0, "0", "delete-char", "e\u{301}👩\u{200d}💻\r\nz"),
    ] {
        let source = "e\u{301}👩\u{200d}💻\r\nz";
        let mut session = editor("", source, at);
        count(&mut session, number);
        named(&mut session, command);
        settle(&mut session);
        assert_eq!(text(&session), expected, "{name}");
        if number != "0" {
            assert_eq!(session.buffer().unwrap().history().len(), 1);
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source);
        }
    }
}

// Supplied kill-line uses logical lines even when wrapped. Zero and negative
// counts retain the exact current/preceding line starts and CRLF bytes.
#[test]
fn logical_kills() {
    for (name, at, number, remaining, killed) in [
        ("forward", 2, "2", "abef", "c\r\nd\r\n"),
        ("zero", 6, "0", "abc\r\n\r\nef", "d"),
        ("backward", 9, "-1", "abc\r\nf", "d\r\ne"),
        ("two backward", 9, "-2", "f", "abc\r\nd\r\ne"),
    ] {
        let mut session = editor("editor.soft_wrap = true", "abc\r\nd\r\nef", at);
        count(&mut session, number);
        named(&mut session, "kill-line");
        settle(&mut session);
        assert_eq!(text(&session), remaining, "{name}");
        assert_eq!(session.killed_text().unwrap().to_string(), killed, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "abc\r\nd\r\nef");
    }
}

// Prefix entry and named selection preserve a directional kill chain; a
// counted span publishes once and prepends when its effective direction flips.
#[test]
fn word_chains() {
    let mut session = editor("", "one two three four", 0);
    count(&mut session, "2");
    named(&mut session, "kill-word");
    settle(&mut session);
    assert_eq!(text(&session), " three four");
    assert_eq!(session.killed_text().unwrap().to_string(), "one two");
    count(&mut session, "1");
    named(&mut session, "kill-word");
    settle(&mut session);
    assert_eq!(session.kill_ring().count(), 1);
    assert_eq!(session.killed_text().unwrap().to_string(), "one two three");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    let mut session = editor("", "one two three", 13);
    count(&mut session, "-2");
    named(&mut session, "kill-word");
    settle(&mut session);
    assert_eq!(text(&session), "one ");
    assert_eq!(session.killed_text().unwrap().to_string(), "two three");
}

// Numeric yank selects once, plain-prefix exchanges point/mark atomically and
// signed rotations retain one exact checked span without reordering the ring.
#[test]
fn ring_arguments() {
    let mut session = stocked(&["old", "é界", "NEW"], "<>");
    session.execute(Command::Motion(Motion::ForwardChar));
    count(&mut session, "2");
    named(&mut session, "yank");
    settle(&mut session);
    assert_eq!(text(&session), "<é界>");
    for (number, expected) in [("0", "<é界>"), ("-1", "<NEW>"), ("5", "<old>")] {
        count(&mut session, number);
        named(&mut session, "yank-pop");
        settle(&mut session);
        assert_eq!(text(&session), expected);
    }
    session.handle(control('u'));
    named(&mut session, "yank");
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(4));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(7)));
    assert_eq!(
        session
            .kill_ring()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["NEW", "é界", "old"]
    );
}

// Native history groups stay separate and stop at their boundaries. Negative
// and zero requests do not mutate history or revive a special yank capture.
#[test]
fn history_groups() {
    let mut session = editor("", "", 0);
    for value in ["a", "b", "c"] {
        let at = session.buffer().unwrap().len_chars();
        session.replace(at..at, value).unwrap();
    }
    for (number, name, expected, groups) in [
        ("2", "undo", "a", 1),
        ("1", "undo-redo", "ab", 2),
        ("0", "undo", "ab", 2),
        ("-1", "undo", "ab", 2),
        ("1000000", "undo", "", 0),
        ("1000000", "undo-redo", "abc", 3),
    ] {
        count(&mut session, number);
        named(&mut session, name);
        settle(&mut session);
        assert_eq!(text(&session), expected, "{number} {name}");
        assert_eq!(session.buffer().unwrap().history().len(), groups);
    }
}

// Large traversal can be canceled before publication; native history is the
// progressive exception and retains completed groups when cancellation arrives.
#[test]
fn cooperative_edits() {
    let source = "x".repeat(5000);
    let mut session = editor("", &source, 0);
    count(&mut session, "5000");
    named(&mut session, "delete-char");
    assert_eq!(text(&session), source);
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    let mut session = editor("history.group_limit = 1000", "", 0);
    for at in 0..300 {
        session.replace(at..at, "x").unwrap();
    }
    count(&mut session, "300");
    named(&mut session, "undo");
    let completed = 300 - session.buffer().unwrap().history().len();
    assert!((1..=256).contains(&completed));
    let retained = text(&session);
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(text(&session), retained);
    assert_eq!(session.buffer().unwrap().history().len(), 300 - completed);
}

// Deletion consumes a live region once in either endpoint order. Zero leaves
// that selection intact; a refused read-only request preserves every owner.
#[test]
fn deletion_regions() {
    for (number, backward) in [("10", false), ("-10", true), ("0", false)] {
        let mut session = editor("", "aoldz", if backward { 4 } else { 1 });
        session.execute(Command::Region(RegionCommand::Set));
        for _ in 0..3 {
            session.execute(Command::Motion(if backward {
                Motion::BackwardChar
            } else {
                Motion::ForwardChar
            }));
        }
        let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
        count(&mut session, number);
        named(&mut session, "delete-char");
        settle(&mut session);
        if number == "0" {
            assert_eq!(
                super::snapshot(session.buffer().unwrap(), &[session.view().point]),
                before
            );
            assert!(session.buffer().unwrap().mark_active());
        } else {
            assert_eq!(text(&session), "az");
            assert_eq!(session.buffer().unwrap().history().len(), 1);
        }
    }
    let mut session = super::editor("abc", 1, true);
    let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
    for command in ["delete-char", "kill-word", "newline", "insert-tab"] {
        count(&mut session, "2");
        named(&mut session, command);
        settle(&mut session);
        assert_eq!(
            super::snapshot(session.buffer().unwrap(), &[session.view().point]),
            before,
            "{command}"
        );
        assert_eq!(session.kill_ring().count(), 0);
    }
}

// A revision change revokes a disposable kill before text or ring publication.
// Input queued after that request resumes under the existing FIFO owner.
#[test]
fn stale_preparation() {
    let source = "word ".repeat(1000);
    let mut session = editor("", &source, 0);
    count(&mut session, "1000000");
    named(&mut session, "kill-word");
    assert_eq!(text(&session), source);
    session.handle(character('!'));
    session.replace(0..0, "z").unwrap();
    settle(&mut session);
    assert_eq!(text(&session), format!("z!{source}"));
    assert_eq!(session.kill_ring().count(), 0);
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}

// Insert/Replace profiles retain their source entry rules, but each supplied
// literal count forms one isolated undo unit rather than joining modal typing.
#[test]
fn modal_entries() {
    for (entry, expected) in [('i', "界界abc"), ('R', "界界c")] {
        let mut session = editor("editor.interaction_profile = 'modal'", "abc", 0);
        session.handle(character(entry));
        count(&mut session, "2");
        session.handle(character('界'));
        settle(&mut session);
        assert_eq!(text(&session), expected);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.handle(character('z'));
        named(&mut session, "undo");
        settle(&mut session);
        assert_eq!(text(&session), expected);
        named(&mut session, "undo");
        settle(&mut session);
        assert_eq!(text(&session), "abc");
    }
}

// A plain-prefix yank records exchanged mark/point in the native transaction;
// undo/redo preserve its endpoints, and a stale rotation never touches source.
#[test]
fn yank_orientation() {
    let mut session = stocked(&["é界"], "<>");
    session.execute(Command::Motion(Motion::ForwardChar));
    let before_mark = session.buffer().unwrap().mark();
    session.handle(control('u'));
    named(&mut session, "yank");
    assert_eq!(text(&session), "<é界>");
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(3)));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "<>");
    assert_eq!(session.buffer().unwrap().mark(), before_mark);
    session.execute(Command::History(History::Redo));
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(3)));
    count(&mut session, "-1");
    named(&mut session, "yank-pop");
    assert_eq!(text(&session), "<é界>");
    assert_eq!(session.echo(), "No current yank to replace");
}

// The generated-text bound counts UTF-8 bytes: exactly 8 MiB is one undoable
// publication; a product just beyond it is refused before source/history change.
#[test]
fn byte_boundary() {
    let unit = format!("{}ab", "界".repeat(2730));
    assert_eq!(unit.len(), 8192);
    let mut session = editor("", "", 0);
    count(&mut session, "1024");
    session.handle(Event::Paste(unit.clone()));
    settle(&mut session);
    assert_eq!(session.buffer().unwrap().len_bytes(), 8 * 1024 * 1024);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(session.buffer().unwrap().stamp().revision, 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "");
    let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
    count(&mut session, "1024");
    session.handle(Event::Paste(format!("{unit}x")));
    settle(&mut session);
    assert_eq!(
        super::snapshot(session.buffer().unwrap(), &[session.view().point]),
        before
    );
}

// A supplied line count ignores actual soft-wrapped visual fragments. Its
// absent counterpart keeps the existing visual kill behavior on the same text.
#[test]
fn wrapped_line_count() {
    let first = "word ".repeat(30);
    let source = format!("{first}\r\nnext\r\nlast");
    let mut ordinary = editor("editor.soft_wrap = true", &source, 0);
    ordinary.execute(Command::Kill(rev::command::KillCommand::Line));
    settle(&mut ordinary);
    let ordinary_kill = ordinary.killed_text().unwrap().to_string();
    assert!(ordinary_kill.len() < first.len());
    let mut counted = editor("editor.soft_wrap = true", &source, 0);
    count(&mut counted, "1");
    named(&mut counted, "kill-line");
    settle(&mut counted);
    assert_eq!(
        counted.killed_text().unwrap().to_string(),
        format!("{first}\r\n")
    );
    assert_eq!(text(&counted), "next\r\nlast");
    assert_eq!(counted.buffer().unwrap().history().len(), 1);
    counted.execute(Command::History(History::Undo));
    assert_eq!(text(&counted), source);
}

// The documented third-entry binding selects old text once, while negative
// word kills prepend through the same chain in both ordinary/modal entry.
#[test]
fn integrated_ring() {
    for profile in ["non-modal", "modal"] {
        let config = format!("editor.interaction_profile='{profile}'");
        let mut session = stocked_with(&config, &["old", "middle", "new"], "<>");
        if profile == "modal" {
            session.execute(Command::Modal(rev::command::ModalCommand::Insert));
        }
        count(&mut session, "3");
        session.handle(control('y'));
        settle(&mut session);
        assert_eq!(text(&session), "old<>");
        assert_eq!(session.view().point.position, Scalar(3));
        let mut session = editor(
            &format!("editor.interaction_profile='{profile}'"),
            "one two three",
            13,
        );
        if profile == "modal" {
            session.execute(Command::Modal(rev::command::ModalCommand::Insert));
        }
        for expected in ["three", "two three"] {
            count(&mut session, "-1");
            named(&mut session, "kill-word");
            settle(&mut session);
            assert_eq!(
                session.killed_text().unwrap().to_string(),
                expected,
                "{profile}"
            );
            assert_eq!(session.kill_ring().count(), 1);
        }
        assert_eq!(text(&session), "one ");
        assert_eq!(session.buffer().unwrap().history().len(), 2);
        count(&mut session, "2");
        named(&mut session, "undo");
        settle(&mut session);
        assert_eq!(text(&session), "one two three");
        count(&mut session, "2");
        named(&mut session, "undo-redo");
        settle(&mut session);
        assert_eq!(text(&session), "one ");
    }
}

// Every nonnegative insertion policy treats zero as inert and refuses a
// negative count, including native tab and source-specific newline preparation.
#[test]
fn insertion_refusal() {
    for profile in ["non-modal", "modal"] {
        for number in ["0", "-2"] {
            for command in ["newline", "insert-tab"] {
                check_insertion_refusal(profile, number, command);
            }
        }
    }
}

fn check_insertion_refusal(profile: &str, number: &str, command: &str) {
    let mut session = editor(
        &format!("editor.interaction_profile='{profile}'"),
        "a\r\nb",
        1,
    );
    if profile == "modal" {
        session.execute(Command::Modal(rev::command::ModalCommand::Insert));
    }
    let before = super::snapshot(session.buffer().unwrap(), &[session.view().point]);
    count(&mut session, number);
    named(&mut session, command);
    settle(&mut session);
    assert_eq!(
        super::snapshot(session.buffer().unwrap(), &[session.view().point]),
        before,
        "{profile} {number} {command}"
    );
}
