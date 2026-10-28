use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand as C};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use std::time::{Duration, UNIX_EPOCH};

pub(super) fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    let env = ZoneEnvironment {
        localtime: "/unused-localtime".into(),
        zoneinfo: "/unused-zoneinfo".into(),
        tz: None,
    };
    ZonedOperation::capture(&ZoneSelection::Utc, &env, || {
        Ok(UNIX_EPOCH + Duration::from_secs(1709303400))
    })
}
fn undo(session: &mut Session, source: &str, expected: &str) {
    assert_eq!(text(session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(session), source);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(session), expected);
}

// Repeating completion resets the keyword and publishes its next date, property
// and actual DONE history together. Parent progress observes the reset state.
#[test]
fn repeater_completion() {
    let source = "* Parent [0/1]\n** TODO H\nSCHEDULED: <2024-02-28 Wed +1d>\nbody\n";
    let expected = "* Parent [0/1]\n** TODO H\nSCHEDULED: <2024-02-29 Thu +1d>\n:PROPERTIES:\n:LAST_REPEAT: [2024-03-01 Fri 14:30]\n:END:\n- State \"DONE\" from \"TODO\" [2024-03-01 Fri 14:30]\nbody\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    goto(&mut session, 2);
    action(&mut session, C::NextTodo);
    undo(&mut session, source, expected);
}

// Repeat logging selects none/time/note independently of ordinary CLOSED. C-g
// cancels an entire note transition without resetting state or advancing dates.
#[test]
fn repeater_logging() {
    let source = "* TODO H\nSCHEDULED: <2024-02-28 Wed ++1d>\n";
    for (name, policy, note) in [
        ("none", "none", false),
        ("time", "time", false),
        ("note", "note", true),
    ] {
        let (_fixture, session) = editor(
            &format!("org.todo.log_repeat='{policy}'\norg.todo.log_done=true"),
            "notes.org",
            source,
        );
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::NextTodo);
        if note {
            assert_eq!(text(&session), source);
            session.handle(control('g'));
            assert!(session.buffer().unwrap().history().is_empty());
            action(&mut session, C::NextTodo);
            argument(&mut session, "completed step");
        }
        let result = text(&session);
        assert!(
            result.starts_with("* TODO H\nSCHEDULED: <2024-03-02 Sat ++1d>\n"),
            "{name}: {}",
            session.echo()
        );
        assert!(!result.contains("CLOSED:"), "{name}");
        assert_eq!(result.contains(":LAST_REPEAT:"), policy != "none", "{name}");
        assert_eq!(
            result.contains("- State \"DONE\""),
            policy != "none",
            "{name}"
        );
        assert_eq!(result.contains("completed step"), note, "{name}");
        undo(&mut session, source, &result);
    }
}

// Reset selects the first unfinished state in the workflow or an explicit local
// REPEAT_TO_STATE. Existing property spelling and unknown metadata remain exact.
#[test]
fn repeater_reset_state() {
    for (name, property, expected) in [
        ("default", "", "OPEN"),
        ("local", ":REPEAT_TO_STATE: WAIT\n", "WAIT"),
    ] {
        let source = format!(
            "#+SEQ_TODO: OPEN WAIT | DONE\n* WAIT H\nDEADLINE: <2024-02-28 Wed .+1d>\n:PROPERTIES:\n{property}:Other: keep\n:LAST_REPEAT: old\n:END:\n"
        );
        let (_fixture, session) = editor("org.todo.log_drawer='LOGBOOK'", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        goto(&mut session, 2);
        action(&mut session, C::NextTodo);
        let result = text(&session);
        assert!(
            result.contains(&format!("* {expected} H\n")),
            "{name}: {}",
            session.echo()
        );
        assert!(result.contains("DEADLINE: <2024-03-02 Sat .+1d>"));
        assert!(result.contains(&format!(
            "{property}:Other: keep\n:LAST_REPEAT: [2024-03-01 Fri 14:30]\n:END:\n:LOGBOOK:"
        )));
        assert!(result.contains("- State \"DONE\" from \"WAIT\""));
        undo(&mut session, &source, &result);
    }
}

// Only real active timestamps in the task's own section participate. Source
// blocks, inactive dates and child tasks cannot reset or advance the parent.
#[test]
fn repeater_source_ownership() {
    for (name, tail, repeats) in [
        ("plain", "body <2024-02-28 Wed +1d>\n", true),
        ("inactive", "[2024-02-28 Wed +1d]\n", false),
        (
            "source",
            "#+begin_src text\n<2024-02-28 Wed +1d>\n#+end_src\n",
            false,
        ),
        (
            "child",
            "** TODO Child\nSCHEDULED: <2024-02-28 Wed +1d>\n",
            false,
        ),
    ] {
        let source = format!("* TODO H\n{tail}");
        let (_fixture, session) = editor("org.todo.log_repeat='none'", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::InspectTodo);
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
        action(&mut session, C::NextTodo);
        let expected = if repeats {
            source.replace("2024-02-28 Wed", "2024-02-29 Thu")
        } else {
            source.replacen("TODO", "DONE", 1)
        };
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        undo(&mut session, &source, &expected);
    }
}

// A repeat removes stale CLOSED and non-repeating SCHEDULED while retaining a
// repeating deadline and neighboring source; first-only delays disappear once.
#[test]
fn repeater_planning_facts() {
    let source = "* TODO H\r\nCLOSED: [2024-02-27 Tue] SCHEDULED: <2024-02-28 Wed> DEADLINE: <2024-02-28 Wed +1d --2d>\r\nbody\r\n";
    let (_fixture, session) = editor("org.todo.log_repeat='none'", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    action(&mut session, C::NextTodo);
    let result = text(&session);
    assert!(result.starts_with("* TODO H\r\n"), "{}", session.echo());
    assert!(result.contains("DEADLINE: <2024-02-29 Thu +1d>\r\nbody\r\n"));
    assert!(!result.contains("CLOSED:") && !result.contains("SCHEDULED:"));
    undo(&mut session, source, &result);
}

// Two explicit backlog completions on the same captured date remain two real
// events. LAST_REPEAT updates one property and each complete transition undoes.
#[test]
fn repeater_same_day() {
    let source = "* TODO H\nSCHEDULED: <2024-02-27 Tue +1d>\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    action(&mut session, C::NextTodo);
    let first = text(&session);
    action(&mut session, C::NextTodo);
    let result = text(&session);
    assert!(
        result.contains("<2024-02-29 Thu +1d>"),
        "{}",
        session.echo()
    );
    assert_eq!(result.matches(":LAST_REPEAT:").count(), 1);
    assert_eq!(result.matches("- State \"DONE\"").count(), 2);
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), first);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Explicit selection of another finished keyword does not steal the originating
// workflow's reset state; each sequence retains its first unfinished keyword.
#[test]
fn repeater_workflow_reset() {
    let source = "#+SEQ_TODO: TODO WAIT | DONE\n#+SEQ_TODO: NEXT | FINISHED\n* WAIT H\nSCHEDULED: <2024-02-28 Wed +1d>\n";
    let (_fixture, session) = editor("org.todo.log_repeat='none'", "notes.org", source);
    let mut session = session.with_org_clock(clock);
    goto(&mut session, 3);
    action(&mut session, C::SetTodo);
    argument(&mut session, "FINISHED");
    assert!(
        text(&session).contains("* TODO H\nSCHEDULED: <2024-02-29 Thu +1d>"),
        "{}",
        session.echo()
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}
