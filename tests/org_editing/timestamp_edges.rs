use super::headings::goto;
use super::properties::argument;
use super::timestamps::{clock, dst_clock};
use super::visibility::action;
use super::*;
use rev::command::{History, Motion, OrgCommand as C};
use rev::org::time::ZonedOperation;
use std::sync::atomic::{AtomicUsize, Ordering};

// Read-only timestamp actions reject before acquiring a draft. Calendar-only
// inspection remains usable and cannot insert its selected date into source.
#[test]
fn timestamp_readonly() {
    let source = "* H\n[2024-02-29 木 9:05]\n";
    let (mut buffers, id) = super::folds::buffer(source);
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(true);
    let view = View::new(buffer);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock);
    let origin = session.view().clone();
    for command in [
        C::Timestamp,
        C::TimestampInactive,
        C::Schedule,
        C::Deadline,
        C::TimestampUp,
        C::TimestampDownDay,
        C::DateFromCalendar,
        C::CtrlCCtrlC,
    ] {
        action(&mut session, command);
        assert!(
            session.echo().contains("read-only"),
            "{command:?}: {}",
            session.echo()
        );
        assert_eq!(session.view(), &origin);
    }
    action(&mut session, C::GotoCalendar);
    argument(&mut session, "2024-03-02");
    assert!(session.echo().contains("date selected"));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Both date and offset drafts lose authority when a public canonical edit
// changes their retained source revision. Only that external edit is undoable.
#[test]
fn stale_timestamp_drafts() {
    for (name, offset) in [("date", false), ("offset", true)] {
        let source = "* H\n";
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(dst_clock);
        action(&mut session, C::Timestamp);
        if offset {
            argument(&mut session, "2024-10-27 02:30");
            assert!(session.echo().contains("Ambiguous"));
        }
        session.replace(0..0, "foreign ").unwrap();
        session.handle(key(KeyCode::Enter));
        assert!(
            session.echo().contains("source or view changed"),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), format!("foreign {source}"));
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

// Source observation counts UTF-8 bytes, not scalar characters. The boundary
// admits one 4096-byte logical line but rejects the next complete wide glyph.
#[test]
fn timestamp_line_bound() {
    for (name, count, allowed) in [("inside", 1365, true), ("outside", 1366, false)] {
        let source = "界".repeat(count);
        let (_fixture, session) = editor("", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        session.execute(Command::Motion(Motion::BufferEnd));
        action(&mut session, C::Timestamp);
        if allowed {
            argument(&mut session, "2024-02-29");
            assert_eq!(
                text(&session),
                format!("{source}<2024-02-29 Thu>"),
                "{name}"
            );
        } else {
            assert!(session.echo().contains("4096"), "{}", session.echo());
            assert_eq!(text(&session), source);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}

// Editing an end must resolve the retained first endpoint too. Fold choices
// may permit a backward wall clock, but reversed UTC instants cannot publish.
#[test]
fn timestamp_range_offsets() {
    let source = "<2024-10-27 Sun 02:30>--<2024-10-27 Sun 02:15>";
    for (name, first, end, accepted) in [("ordered", "1", "2", true), ("reversed", "2", "1", false)]
    {
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(dst_clock);
        for _ in 0..28 {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        action(&mut session, C::Timestamp);
        argument(&mut session, "02:10");
        assert!(
            session.echo().contains("Ambiguous"),
            "{name}: {}",
            session.echo()
        );
        argument(&mut session, first);
        assert!(
            session.echo().contains("Ambiguous"),
            "{name}: {}",
            session.echo()
        );
        argument(&mut session, end);
        if accepted {
            assert_eq!(text(&session), source.replace("02:15", "02:10"));
            assert_eq!(session.buffer().unwrap().history().len(), 1);
            session.execute(Command::History(History::Undo));
        } else {
            assert!(session.echo().contains("ends before"), "{}", session.echo());
            assert!(session.buffer().unwrap().history().is_empty());
            session.handle(control('g'));
        }
        assert_eq!(text(&session), source, "{name}");
    }
}

static CAPTURES: AtomicUsize = AtomicUsize::new(0);
fn counted_clock(policy: &rev::config::Time) -> Result<ZonedOperation, String> {
    CAPTURES.fetch_add(1, Ordering::SeqCst);
    dst_clock(policy)
}

// Calendar motion and rejected offset input reuse the operation's original
// clock/zone observation; retries must not observe a different instant.
#[test]
fn timestamp_clock_snapshot() {
    let (_fixture, session) = editor("", "notes.org", "");
    let mut session = session.with_org_clock(counted_clock);
    action(&mut session, C::Timestamp);
    session.execute(Command::Org(C::CalendarForwardDay));
    session.handle(key(KeyCode::Home));
    for _ in 0.."<2024-10-28 Mon>".len() {
        session.handle(key(KeyCode::Delete));
    }
    argument(&mut session, "2024-10-27 02:30");
    assert!(session.echo().contains("Ambiguous"));
    argument(&mut session, "0");
    assert!(session.echo().contains("observed"));
    session.handle(key(KeyCode::Home));
    session.handle(key(KeyCode::Delete));
    argument(&mut session, "2");
    assert_eq!(text(&session), "<2024-10-27 Sun 02:30>");
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
}

// One timestamp transaction maps independent shared views and its inverse
// restores their canonical anchors, including Unicode before the peer point.
#[test]
fn timestamp_shared_views() {
    let source = "* H\n[2024-02-29 木 9:05]\n* Peer\nmarker\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    goto(&mut session, 4);
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    let peer = session
        .windows()
        .order()
        .iter()
        .copied()
        .find(|id| *id != session.windows().selected_id())
        .unwrap();
    let original = session.windows().get(peer).unwrap().point.position;
    goto(&mut session, 2);
    action(&mut session, C::TimestampInactive);
    argument(&mut session, "2024-03-01 10:15");
    let expected = source.replace("2024-02-29 木 9:05", "2024-03-01 Fri 10:15");
    assert_eq!(text(&session), expected);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        expected[..expected.find("marker").unwrap()].chars().count()
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        original
    );
}

// Planning metadata uses literal inline coloring but owns real timestamps.
// Its end-of-line context actions must remain available with CRLF separators.
#[test]
fn timestamp_planning_context() {
    for fact in ["SCHEDULED", "DEADLINE"] {
        let source = format!("* H\r\n{fact}: <2026-10-30 Fri 09:30-10:00>\r\nbody");
        let (_fixture, session) = editor("", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        goto(&mut session, 2);
        session.execute(Command::Motion(Motion::LineEnd));
        action(&mut session, C::ShiftRight);
        assert_eq!(
            text(&session),
            source.replace("2026-10-30 Fri", "2026-10-31 Sat"),
            "{}",
            session.echo()
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}
