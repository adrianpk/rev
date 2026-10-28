use super::clocks::clock;
use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{BufferCommand, History, OrgCommand as C};
use rev::org::time::ZonedOperation;
use rev::session::Action;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

fn unavailable(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    Err("owned clock acquisition failed".into())
}
// Read-only rejection precedes clock acquisition. A failed capture or invalid
// source context leaves text, point, stamp and semantic history unchanged.
#[test]
fn clock_admission() {
    let source = "* Task\nbody";
    let (mut buffers, id) = super::folds::buffer(source);
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(unavailable);
    action(&mut session, C::ClockIn);
    assert!(session.echo().contains("read-only"), "{}", session.echo());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    // Keep the over-limit source exact, with ordinary comment rows so the
    // fixture tests entry admission without spending its deadline on markup.
    let excess = format!(
        "* Task\n{}{}",
        ("# ".to_owned() + &"x".repeat(125) + "\n").repeat(2047),
        "#".repeat(122)
    );
    assert_eq!(excess.len(), 262145);
    for (name, source, row, wanted) in [
        (
            "literal point",
            "* Task\n#+begin_src sh\necho x\n#+end_src\n",
            3,
            "unavailable",
        ),
        (
            "malformed",
            "* Task\n:PROPERTIES:\n:X: keep\n",
            1,
            "Malformed",
        ),
        (
            "duplicate",
            "* Task\n:PROPERTIES:\n:CLOCK_INTO_DRAWER: A\n:CLOCK_INTO_DRAWER: B\n:END:\n",
            1,
            "Duplicate",
        ),
        ("source cap", &excess, 1, "262144"),
    ] {
        let (_fixture, session) = editor("", "clock.org", source);
        let mut session = session.with_org_clock(clock);
        goto(&mut session, row);
        let before = session.view().clone();
        action(&mut session, C::ClockIn);
        assert!(
            session.echo().contains(wanted),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.view(), &before, "{name}");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

static CAPTURES: AtomicUsize = AtomicUsize::new(0);
fn counted(policy: &rev::config::Time) -> Result<ZonedOperation, String> {
    CAPTURES.fetch_add(1, Ordering::SeqCst);
    clock(policy)
}
// An active source interval installs no idle deadline or repaint. Advancing the
// host clock through many seconds cannot observe time or mutate source/history.
#[test]
fn clock_idle() {
    let (_fixture, session) = editor("", "clock.org", "* Task\nbody");
    let mut session = session.with_org_clock(counted);
    CAPTURES.store(0, Ordering::SeqCst);
    action(&mut session, C::ClockIn);
    let stamp = session.buffer().unwrap().stamp();
    let opened = text(&session);
    let now = Instant::now();
    for seconds in [1, 2, 60, 3600, 86400] {
        assert_eq!(
            session.remaining_input(now + Duration::from_secs(seconds)),
            None
        );
        assert_eq!(
            session.expire_input(now + Duration::from_secs(seconds)),
            Action::Wait
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(text(&session), opened);
    }
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
    action(&mut session, C::ClockCancel);
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
    assert_eq!(session.remaining_input(Instant::now()), None);
}

// Captures and stale locators fail atomically. Literal incomplete clocks do not
// block the parent; a real child interval remains outside the parent's source.
#[test]
fn clock_failures() {
    let source = "* Parent\n#+begin_example\nCLOCK: [2024-01-01 Mon 10:00]\n#+end_example\n** Child\nCLOCK: [2024-01-01 Mon 11:00]\n";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(unavailable);
    let origin = session.view().clone();
    action(&mut session, C::ClockIn);
    assert!(session.echo().contains("acquisition failed"));
    assert_eq!(session.view(), &origin);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    session = session.with_org_clock(clock);
    action(&mut session, C::ClockIn);
    let opened = text(&session);
    session = session.with_org_clock(unavailable);
    action(&mut session, C::ClockOut);
    assert!(session.echo().contains("acquisition failed"));
    assert_eq!(text(&session), opened);
    session.handle(Event::Paste("unrelated".into()));
    let changed = text(&session);
    for command in [C::ClockOut, C::ClockCancel, C::ClockGoto] {
        action(&mut session, command);
        assert!(
            session.echo().contains("source changed"),
            "{}",
            session.echo()
        );
        assert_eq!(text(&session), changed);
    }
}

// Real file saves retain BOM, CRLF, Unicode and unterminated EOF. Both shared
// views map through each clock operation, and undo restores the original bytes.
#[test]
fn clock_shared_save() {
    let source = "* Task 界e\u{301}\r\nbody\r\n* Peer\r\nmarker";
    let (fixture, session) = editor("", "clock.org", &format!("\u{feff}{source}"));
    let mut session = session.with_org_clock(clock);
    goto(&mut session, 4);
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    let peer = *session
        .windows()
        .order()
        .iter()
        .find(|id| **id != session.windows().selected_id())
        .unwrap();
    let original = session.windows().get(peer).unwrap().point.position;
    goto(&mut session, 1);
    action(&mut session, C::ClockIn);
    let opened = text(&session);
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("clock.org")).unwrap(),
        format!("\u{feff}{opened}").as_bytes()
    );
    action(&mut session, C::ClockOut);
    let closed = text(&session);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        closed[..closed.find("marker").unwrap()].chars().count()
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("clock.org")).unwrap(),
        format!("\u{feff}{closed}").as_bytes()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), opened);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        original
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("clock.org")).unwrap(),
        format!("\u{feff}{source}").as_bytes()
    );
}

// Out/cancel address the session-owned source even from a plain buffer. The
// invoking view remains exact, while goto explicitly selects the checked task.
#[test]
fn clock_foreign_view() {
    for command in [C::ClockOut, C::ClockCancel, C::ClockGoto] {
        let (fixture, session) = editor("", "clock.org", "* Task\nbody");
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::ClockIn);
        let path = fixture.write("plain.txt", "plain 界\n");
        session.execute(Command::FindFile);
        argument(&mut session, path.to_str().unwrap());
        let other = session.view().clone();
        action(&mut session, command);
        if command == C::ClockGoto {
            assert!(text(&session).starts_with("* Task\n:LOGBOOK:\nCLOCK:"));
        } else {
            assert_eq!(session.view(), &other);
            assert_eq!(text(&session), "plain 界\n");
            session.execute(Command::Buffer(BufferCommand::Switch));
            argument(&mut session, "clock.org");
            let expected = if command == C::ClockCancel {
                "* Task\nbody"
            } else {
                "* Task\n:LOGBOOK:\nCLOCK: [2024-03-01 Fri 14:30]--[2024-03-01 Fri 14:30] =>  0:00\n:END:\nbody"
            };
            assert_eq!(text(&session), expected, "{}", session.echo());
        }
    }
}
