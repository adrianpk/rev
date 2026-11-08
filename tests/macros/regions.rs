use super::*;
use rev::command::{Motion, RegionCommand};

fn prepare(
    recording_source: &str,
    events: impl IntoIterator<Item = Event>,
    source: &str,
) -> Session {
    let mut session = editor("", recording_source);
    record(&mut session, events);
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, source).unwrap();
    session
}
fn select(session: &mut Session, start: usize, end: Option<usize>) {
    session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..start {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.execute(Command::Region(RegionCommand::Set));
    if let Some(end) = end {
        for _ in start..end {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
    } else {
        session.execute(Command::Motion(Motion::BufferEnd));
    }
}
fn apply(session: &mut Session) {
    feed(session, [control('x'), control('k'), character('r')]);
    settle(session);
}

// Half-open logical line beginnings determine the original target set. Mid-line
// starts, reversed selections and a next-line endpoint do not add extra targets.
#[test]
fn boundaries() {
    for (name, start, end, reverse, expected) in [
        ("all", 0, None, false, ">a\n>b\n>c\n"),
        ("next-line end", 0, Some(4), false, ">a\n>b\nc\n"),
        ("mid-line start", 1, Some(5), false, "a\n>b\n>c\n"),
        ("reversed", 0, Some(4), true, ">a\n>b\nc\n"),
        ("empty", 0, Some(0), false, "a\nb\nc\n"),
    ] {
        let mut session = prepare("", [character('>')], "a\nb\nc\n");
        select(&mut session, start, end);
        if reverse {
            session.execute(Command::Region(RegionCommand::Exchange));
        }
        apply(&mut session);
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.macro_status(), Status::Idle, "{name}");
        assert_eq!(session.macro_stop(), None, "{name}");
    }
}

// Visual wrapping and Unicode graphemes do not manufacture logical targets;
// each actual source line gets one counter value and one native undo unit.
#[test]
fn unicode_wrap() {
    let source = format!("{}\ne\u{301}界\nlast", "界".repeat(100));
    let mut session = prepare("", [key(KeyCode::F(3))], &source);
    session.handle(Event::Resize(24, 10));
    settle(&mut session);
    select(&mut session, 0, None);
    apply(&mut session);
    let expected = format!("1{}\n2e\u{301}界\n3last", "界".repeat(100));
    assert_eq!(text(&session), expected);
    session.execute(Command::History(History::Undo));
    assert_eq!(
        text(&session),
        format!("1{}\n2e\u{301}界\nlast", "界".repeat(100))
    );
    session.execute(Command::History(History::Undo));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Newly inserted rows are not targets. Insertion at a future original line
// follows its original text, instead of retargeting the inserted line.
#[test]
fn inserted_lines() {
    let mut session = prepare("", [Event::Paste("new\n>".into())], "a\nb\nc");
    select(&mut session, 0, None);
    apply(&mut session);
    assert_eq!(text(&session), "new\n>a\nnew\n>b\nnew\n>c");
    let mut session = prepare(
        "a\nb\nc\n",
        [
            control('n'),
            control('a'),
            Event::Paste("new\n".into()),
            control('p'),
            control('p'),
            control('a'),
            character('>'),
        ],
        "a\nb\nc\n",
    );
    select(&mut session, 0, Some(4));
    apply(&mut session);
    assert_eq!(text(&session), ">a\nnew\n>b\nnew\nc\n");
    assert_eq!(session.macro_stop(), None);
}

// Deleting a future source line tombstones that target. Surviving originals
// still run once; deleting just their first character does not delete the line.
#[test]
fn deleted_lines() {
    let source = "a\nb\nc\nd\ne\nf\n";
    let mut session = prepare(
        source,
        [
            control('n'),
            control('a'),
            control('k'),
            control('k'),
            control('p'),
            control('a'),
            character('>'),
        ],
        source,
    );
    select(&mut session, 0, Some(8));
    apply(&mut session);
    assert_eq!(text(&session), ">a\n>c\ne\nf\n");
    assert_eq!(session.macro_stop(), None);
    let mut session = prepare(
        "a\nb\nc\n",
        [
            control('n'),
            control('a'),
            control('d'),
            character('X'),
            control('p'),
            control('a'),
            character('>'),
        ],
        "a\nb\nc\n",
    );
    select(&mut session, 0, Some(4));
    apply(&mut session);
    assert_eq!(text(&session), ">a\n>X\nX\n");
    assert_eq!(session.macro_stop(), None);
}

// Large target admission refuses before source edits and cannot exceed the
// exact native anchor ceiling. Empty/unmarked source has no implicit target.
#[test]
fn bounds() {
    for count in [65_536, 65_537] {
        let source = "x\n".repeat(count);
        let mut session = prepare("", [character('>')], &source);
        select(&mut session, 0, None);
        session.execute(Command::Macro(MacroCommand::ApplyLines));
        if count == 65_536 {
            assert_eq!(session.macro_status(), Status::Playing);
            session.handle(control('g'));
        } else {
            assert_eq!(
                session.macro_stop().unwrap().reason,
                StopReason::RegionLimit
            );
        }
        assert_eq!(text(&session), source);
    }
    let mut session = prepare("", [character('>')], "a\nb\n");
    apply(&mut session);
    assert_eq!(text(&session), "a\nb\n");
    assert_eq!(session.macro_status(), Status::Idle);
}

// Cancellation retains a completed region iteration as its own undo unit.
// An out-of-band source publication invalidates the admitted continuation.
#[test]
fn cancellation() {
    for external in [false, true] {
        let mut session = prepare("", [character('>')], "a\nb\nc\n");
        select(&mut session, 0, None);
        session.execute(Command::Macro(MacroCommand::ApplyLines));
        session.expire_input(Instant::now());
        assert_eq!(text(&session), ">a\nb\nc\n");
        if external {
            session.replace(1..1, "external").unwrap();
            settle(&mut session);
            assert_eq!(
                session.macro_stop().unwrap().reason,
                StopReason::OwnerChanged
            );
            assert_eq!(text(&session), ">externala\nb\nc\n");
        } else {
            session.handle(control('g'));
            assert_eq!(session.macro_stop().unwrap().reason, StopReason::Canceled);
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), "a\nb\nc\n");
        }
        assert_eq!(session.remaining_input(Instant::now()), None);
    }
}
