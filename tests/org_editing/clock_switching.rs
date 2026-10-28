use super::clocks::clock;
use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{BufferCommand, History, OrgCommand as C};

// Switching within either source order closes the old interval and opens the
// selected entry together, with one transaction and independently expected text.
#[test]
fn clock_switching() {
    let source = "* First\nbody\n* Second\ntext";
    for (first, next) in [(1, 6), (3, 1)] {
        let (_fixture, session) = editor("", "clock.org", source);
        let mut session = session.with_org_clock(clock);
        goto(&mut session, first);
        action(&mut session, C::ClockIn);
        let opened = text(&session);
        action(&mut session, C::ClockIn);
        assert_eq!(text(&session), opened);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        goto(&mut session, next);
        action(&mut session, C::ClockIn);
        let switched = text(&session);
        assert_eq!(
            switched.matches("CLOCK:").count(),
            2,
            "{}: {switched}",
            session.echo()
        );
        assert_eq!(switched.matches(" =>  0:00").count(), 1);
        action(&mut session, C::ClockGoto);
        let expected_row = if first == 1 { 5 } else { 0 };
        assert_eq!(super::headings::row(&session), expected_row);
        action(&mut session, C::ClockOut);
        assert_eq!(
            text(&session).matches(" =>  0:00").count(),
            2,
            "{}",
            session.echo()
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), switched);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), opened);
    }
}

// A different buffer switches through checked native publication without saving
// either file implicitly. Invalid destination admission cannot close the source.
#[test]
fn clock_cross_buffer() {
    let (fixture, session) = editor("", "first.org", "* First\nbody");
    let mut session = session.with_org_clock(clock);
    action(&mut session, C::ClockIn);
    let opened = text(&session);
    let second = fixture.write("second.org", "* Second\ntext");
    session.load(&second).unwrap();
    action(&mut session, C::ClockIn);
    assert!(
        text(&session).contains("CLOCK: [2024-03-01 Fri 14:30]\n"),
        "{}",
        session.echo()
    );
    session.execute(Command::Buffer(BufferCommand::Switch));
    argument(&mut session, "first.org");
    assert_eq!(
        text(&session),
        opened.replace(
            "[2024-03-01 Fri 14:30]",
            "[2024-03-01 Fri 14:30]--[2024-03-01 Fri 14:30] =>  0:00"
        )
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("first.org")).unwrap(),
        "* First\nbody"
    );
    assert_eq!(std::fs::read_to_string(second).unwrap(), "* Second\ntext");
    action(&mut session, C::ClockGoto);
    assert!(text(&session).starts_with("* Second"));
    action(&mut session, C::ClockCancel);
    assert_eq!(text(&session), "* Second\ntext");
}

// Task completion, log insertion and clock-out share one history boundary even
// when LOGBOOK insertions meet the open line or zero removal deletes its wrapper.
#[test]
fn clock_completion() {
    for (config, record) in [
        ("", true),
        ("org.clock.remove_zero=true", false),
        ("org.clock.out_when_done=false", true),
    ] {
        let config = format!("org.todo.log_done=true\norg.todo.log_drawer='LOGBOOK'\n{config}");
        let (_fixture, session) = editor(
            &config,
            "clock.org",
            "#+TODO: TODO | DONE(!)\n* TODO Task\nbody",
        );
        let mut session = session.with_org_clock(clock).with_task_clock(stamp);
        goto(&mut session, 2);
        action(&mut session, C::ClockIn);
        let opened = text(&session);
        action(&mut session, C::NextTodo);
        let changed = text(&session);
        assert!(
            changed.contains("* DONE Task\nCLOSED: [2024-03-01 Fri 14:30]"),
            "{}: {changed}",
            session.echo()
        );
        assert!(changed.contains("- State \"DONE\" from \"TODO\" [2024-03-01 Fri 14:30]"));
        assert_eq!(changed.contains("CLOCK:"), record);
        if config.contains("out_when_done=false") {
            assert!(!changed.contains(" => "));
        } else {
            assert_eq!(changed.contains(" =>  0:00"), record);
        }
        assert_eq!(session.buffer().unwrap().history().len(), 2);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), opened);
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), changed);
    }
}

// A cancelled completion note leaves its clock running and source exact. The
// accepted note retains the already captured end rather than measuring typing.
#[test]
fn clock_completion_note() {
    let (_fixture, session) = editor(
        "org.todo.log_drawer='LOGBOOK'",
        "clock.org",
        "#+TODO: TODO(t) | DONE(d@)\n* TODO Task\nbody",
    );
    let mut session = session.with_org_clock(clock);
    goto(&mut session, 2);
    action(&mut session, C::ClockIn);
    let opened = text(&session);
    action(&mut session, C::NextTodo);
    session.handle(control('g'));
    assert_eq!(text(&session), opened);
    action(&mut session, C::NextTodo);
    session = session.with_org_clock(failed);
    argument(&mut session, "finished 界");
    assert!(text(&session).contains(" =>  0:00"), "{}", session.echo());
    assert!(text(&session).contains("finished 界"));
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}
fn failed(_: &rev::config::Time) -> Result<rev::org::time::ZonedOperation, String> {
    Err("must retain captured completion instant".into())
}

fn stamp() -> Result<String, String> {
    Ok("[2024-03-01 Fri 14:30]".into())
}

// A stale previous source fails after destination observation but before either
// buffer publishes. No guessed task match or partial close/open is allowed.
#[test]
fn clock_switch_rejection() {
    let (fixture, session) = editor("", "first.org", "* First\nbody");
    let mut session = session.with_org_clock(clock);
    action(&mut session, C::ClockIn);
    session.handle(Event::Paste("changed".into()));
    let previous = text(&session);
    let second = fixture.write("second.org", "* Second\ntext");
    session.load(&second).unwrap();
    action(&mut session, C::ClockIn);
    assert!(
        session.echo().contains("source changed"),
        "{}",
        session.echo()
    );
    assert_eq!(text(&session), "* Second\ntext");
    assert!(session.buffer().unwrap().history().is_empty());
    session.execute(Command::Buffer(BufferCommand::Switch));
    argument(&mut session, "first.org");
    assert_eq!(text(&session), previous);
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}
