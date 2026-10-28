use super::headings::goto;
use super::properties::argument;
use super::timestamps::clock;
use super::visibility::action;
use super::*;
use rev::command::{History, Motion, OrgCommand as C};

fn point(session: &mut Session, count: usize) {
    for _ in 0..count {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
}
fn shifted(session: &mut Session, code: KeyCode) {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::SHIFT)));
    settle(session);
}

// Native S-Up/Down select the numeric field and retain that field for repeated
// adjustment, even when crossing midnight normalizes a localized weekday.
#[test]
fn native_timestamp_fields() {
    let source = "[2024-02-29 木 23:45-0:15 +1w]";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    point(&mut session, 14);
    shifted(&mut session, KeyCode::Up);
    assert_eq!(
        text(&session),
        "[2024-03-01 Fri 00:45-01:15 +1w]",
        "{}",
        session.echo()
    );
    shifted(&mut session, KeyCode::Up);
    assert_eq!(text(&session), "[2024-03-01 Fri 01:45-02:15 +1w]");
    shifted(&mut session, KeyCode::Down);
    assert_eq!(text(&session), "[2024-03-01 Fri 00:45-01:15 +1w]");
    shifted(&mut session, KeyCode::Left);
    assert_eq!(text(&session), "[2024-02-29 Thu 00:45-01:15 +1w]");
    assert_eq!(session.buffer().unwrap().history().len(), 4);
}

// Context actions preserve the earlier priority, task, tags and checkbox owners
// when no timestamp owns point. Literal blocks cannot select an ancestor task.
#[test]
fn timestamp_context_fallbacks() {
    for (name, source, code, expected) in [
        ("priority", "* H", KeyCode::Up, "* [#C] H"),
        ("task", "* TODO H", KeyCode::Right, "* DONE H"),
        ("previous task", "* DONE H", KeyCode::Left, "* TODO H"),
    ] {
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(clock);
        shifted(&mut session, code);
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
    }
    for (name, source, line) in [
        (
            "source",
            "* H\n#+begin_src text\n<2024-02-29 Thu>\n#+end_src\n",
            3,
        ),
        (
            "example",
            "* H\n#+begin_example\n<2024-02-29 Thu>\n#+end_example\n",
            3,
        ),
        ("comment", "* H\n# <2024-02-29 Thu>\n", 2),
    ] {
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(clock);
        goto(&mut session, line);
        point(&mut session, 3);
        shifted(&mut session, KeyCode::Up);
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// C-c C-c normalizes only the selected weekday. The same action still toggles
// a checkbox when point belongs to a list item rather than its timestamp.
#[test]
fn timestamp_normalization() {
    let source = "x <2024-02-29  Sun  9:05 +1w>";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    point(&mut session, 7);
    session.handle(control('c'));
    session.handle(control('c'));
    settle(&mut session);
    assert_eq!(
        text(&session),
        "x <2024-02-29 Thu 9:05 +1w>",
        "{}",
        session.echo()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    let (_fixture, mut session) = editor("", "notes.org", "- [ ] item <2024-02-29 Thu>\n");
    action(&mut session, C::CtrlCCtrlC);
    assert_eq!(text(&session), "- [X] item <2024-02-29 Thu>\n");
}

// Two successive timestamp commands append a checked range. Cancel leaves the
// first insertion intact, and undo of the second command restores it as one unit.
#[test]
fn successive_range_entry() {
    for (name, command, open, close) in [
        ("active", C::Timestamp, '<', '>'),
        ("inactive", C::TimestampInactive, '[', ']'),
    ] {
        let (_fixture, session) = editor("", "notes.org", "");
        let mut session = session.with_org_clock(clock);
        action(&mut session, command);
        argument(&mut session, "2024-02-29");
        let first = format!("{open}2024-02-29 Thu{close}");
        assert_eq!(text(&session), first, "{name}");
        action(&mut session, command);
        argument(&mut session, "2024-03-02");
        let range = format!("{first}--{open}2024-03-02 Sat{close}");
        assert_eq!(text(&session), range, "{name}: {}", session.echo());
        assert_eq!(session.view().point.position.0, range.len());
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), first);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "");
    }
}

// An intervening source/view action breaks successive-command authority. A new
// entry after ordinary typing cannot reuse the prior timestamp's source anchor.
#[test]
fn interrupted_range_entry() {
    let (_fixture, session) = editor("", "notes.org", "");
    let mut session = session.with_org_clock(clock);
    action(&mut session, C::Timestamp);
    argument(&mut session, "2024-02-29");
    session.handle(key(KeyCode::Char(' ')));
    action(&mut session, C::Timestamp);
    argument(&mut session, "2024-03-02");
    assert_eq!(
        text(&session),
        "<2024-02-29 Thu> <2024-03-02 Sat>",
        "{}",
        session.echo()
    );
    let first = text(&session);
    action(&mut session, C::Timestamp);
    session.handle(control('g'));
    assert_eq!(text(&session), first);
}

// Bracket adjustment changes the whole connected range's active status; changing
// the end's date checks the retained first date before any source publication.
#[test]
fn range_context_fields() {
    let source = "<2024-02-29 Thu>--<2024-03-01 Fri>";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    shifted(&mut session, KeyCode::Up);
    assert_eq!(text(&session), "[2024-02-29 Thu]--[2024-03-01 Fri]");
    point(&mut session, 27);
    shifted(&mut session, KeyCode::Down);
    assert_eq!(
        text(&session),
        "[2024-02-29 Thu]--[2024-02-29 Thu]",
        "{}",
        session.echo()
    );
    let before = text(&session);
    shifted(&mut session, KeyCode::Down);
    assert_eq!(text(&session), before);
    assert!(session.echo().contains("ordered"));
}

// Minute adjustment retains the minute field when an unpadded hour gains a digit.
// A second native press must advance one minute, rather than switching to hours.
#[test]
fn repeated_minute_field() {
    let (_fixture, session) = editor("", "notes.org", "[2024-02-29 木 9:59-10:15]");
    let mut session = session.with_org_clock(clock);
    point(&mut session, 16);
    shifted(&mut session, KeyCode::Up);
    assert_eq!(text(&session), "[2024-02-29 木 10:00-10:16]");
    shifted(&mut session, KeyCode::Up);
    assert_eq!(
        text(&session),
        "[2024-02-29 木 10:01-10:17]",
        "{}",
        session.echo()
    );
}

// Task-state context includes an unterminated heading's end without admitting
// its following body line or a blank line after a terminated heading.
#[test]
fn task_heading_end() {
    for (name, source, allowed) in [
        ("heading", "* TODO 界", true),
        ("body", "* TODO H\nbody", false),
        ("blank", "* TODO H\n", false),
    ] {
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(clock);
        session.execute(Command::Motion(Motion::BufferEnd));
        shifted(&mut session, KeyCode::Right);
        let expected = if allowed {
            source.replace("TODO", "DONE")
        } else {
            source.into()
        };
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
    }
}
