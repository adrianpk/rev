use super::headings::goto;
use super::properties::argument;
use super::repeaters::clock;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand as C};
use rev::org::time::ZonedOperation;
use std::sync::atomic::{AtomicUsize, Ordering};

fn denied_clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    Err("clock must not be acquired for read-only source".into())
}

// The read-only boundary rejects repeat completion before clock acquisition or
// draft creation, and ordinary inspection cannot advance a repeated schedule.
#[test]
fn repeat_readonly() {
    let source = "* TODO H\nSCHEDULED: <2024-02-28 Wed +1d>\n";
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
    .with_org_clock(denied_clock);
    for command in [C::Todo, C::NextTodo, C::SetTodo] {
        action(&mut session, command);
        assert!(session.echo().contains("read-only"), "{}", session.echo());
    }
    action(&mut session, C::InspectTodo);
    assert!(session.echo().contains("TODO"));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Malformed recurrence, planning, properties and source-year overflow reject
// before publication; even a late metadata failure cannot publish advanced dates.
#[test]
fn repeat_atomic_rejections() {
    for (name, source, error) in [
        (
            "hour clock",
            "* TODO H\nSCHEDULED: <2024-02-28 Wed +1h>\n",
            "Hourly",
        ),
        (
            "range",
            "* TODO H\n<2024-02-28 Wed +1w>--<2024-03-01 Fri +1d>\n",
            "ordered",
        ),
        (
            "reset",
            "* TODO H\nSCHEDULED: <2024-02-28 Wed +1d>\n:PROPERTIES:\n:REPEAT_TO_STATE: MISSING\n:END:\n",
            "REPEAT_TO_STATE",
        ),
        (
            "duplicate property",
            "* TODO H\nSCHEDULED: <2024-02-28 Wed +1d>\n:PROPERTIES:\n:LAST_REPEAT: old\n:LAST_REPEAT: duplicate\n:END:\n",
            "Duplicate local",
        ),
        (
            "duplicate fact",
            "* TODO H\nSCHEDULED: <2024-02-28 Wed +1d> SCHEDULED: <2024-02-28 Wed +1d>\n",
            "Duplicate Org SCHEDULED",
        ),
        (
            "year limit",
            "* TODO H\nSCHEDULED: <9999-12-31 Fri +1d>\n",
            "0001..9999",
        ),
    ] {
        let (_fixture, session) = editor("", "notes.org", source);
        let mut session = session.with_org_clock(clock);
        let origin = session.view().clone();
        action(&mut session, C::NextTodo);
        assert!(session.echo().contains(error), "{name}: {}", session.echo());
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.view(), &origin, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Exactly 64 owned ranges may complete together. The next range exhausts the
// operation budget without a partial keyword, date, history or property edit.
#[test]
fn repeat_range_limit() {
    for (name, count, allowed) in [("at limit", 64, true), ("over limit", 65, false)] {
        let source = format!("* TODO H\n{}\n", "<2024-02-28 Wed +1d> ".repeat(count));
        let (_fixture, session) = editor("org.todo.log_repeat='none'", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::NextTodo);
        if allowed {
            assert_eq!(
                text(&session),
                source.replace("2024-02-28 Wed", "2024-02-29 Thu"),
                "{name}: {}",
                session.echo()
            );
            assert_eq!(session.buffer().unwrap().history().len(), 1);
            session.execute(Command::History(History::Undo));
        } else {
            assert!(
                session.echo().contains("64 repeating"),
                "{}",
                session.echo()
            );
            assert!(session.buffer().unwrap().history().is_empty());
        }
        assert_eq!(text(&session), source, "{name}");
    }
}

static CAPTURES: AtomicUsize = AtomicUsize::new(0);
fn counted_clock(policy: &rev::config::Time) -> Result<ZonedOperation, String> {
    CAPTURES.fetch_add(1, Ordering::SeqCst);
    super::timestamps::dst_clock(policy)
}

// Invalid offset retries and a Unicode note retain one captured clock and zone.
// Neither stage publishes source until the complete transition is accepted.
#[test]
fn repeat_clock_snapshot() {
    let source = "* TODO H\nSCHEDULED: <2024-10-26 Sat 02:30 +1d>\n";
    let (_fixture, session) = editor("org.todo.log_repeat='note'", "notes.org", source);
    let mut session = session.with_org_clock(counted_clock);
    action(&mut session, C::NextTodo);
    argument(&mut session, "0");
    assert!(session.echo().contains("observed"));
    argument(&mut session, "2");
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    argument(&mut session, "completed 界");
    assert!(text(&session).contains("<2024-10-27 Sun 02:30 +1d>"));
    assert!(text(&session).contains("completed 界"));
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// The real file boundary preserves BOM/CRLF and independent shared anchors.
// Saving and undoing a repeat operate on one canonical text/history owner.
#[test]
fn repeat_shared_save() {
    let source = "* TODO 界e\u{301}\r\nSCHEDULED: <2024-02-28 Wed +1d>\r\n* Peer\r\nmarker";
    let (fixture, session) = editor("", "notes.org", &format!("\u{feff}{source}"));
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
    goto(&mut session, 1);
    action(&mut session, C::NextTodo);
    let result = text(&session);
    assert!(
        result.contains("SCHEDULED: <2024-02-29 Thu +1d>\r\n"),
        "{}",
        session.echo()
    );
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        result[..result.find("marker").unwrap()].chars().count()
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("notes.org")).unwrap(),
        format!("\u{feff}{result}").as_bytes()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        original
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("notes.org")).unwrap(),
        format!("\u{feff}{source}").as_bytes()
    );
}
