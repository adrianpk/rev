use super::*;
use rev::command::{BufferCommand, Command, OrgCommand as C};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use std::time::{Duration, Instant, UNIX_EPOCH};

fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
        Ok(UNIX_EPOCH + Duration::from_secs(1709303400))
    })
}
fn owned(fixture: &Fixture) -> Session {
    let path = fixture.file("clock.org", b"* Task\nbody");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock)
}
fn command(session: &mut Session, command: C) {
    session.execute(Command::Org(command));
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            break;
        }
        assert!(Instant::now() < until);
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
fn argument(session: &mut Session, value: &str) -> Action {
    session.handle(Event::Paste(value.into()));
    session.handle(press(KeyCode::Enter))
}

// Quit asks about even a saved active interval. Cancellation retains tracking;
// keep preserves open source, while close publishes before normal save decisions.
#[test]
fn clock_quit() {
    for choice in ["keep", "close"] {
        let fixture = Fixture::new();
        let mut session = owned(&fixture);
        command(&mut session, C::ClockIn);
        session.execute(Command::SaveBuffer);
        let opened = text(&session);
        session.execute(Command::Quit);
        assert!(session.echo().contains("detach"));
        cancel(&mut session);
        command(&mut session, C::ClockGoto);
        assert_eq!(text(&session), opened);
        session.execute(Command::Quit);
        let result = argument(&mut session, choice);
        if choice == "keep" {
            assert_eq!(result, Action::Exit);
            assert_eq!(
                fs::read_to_string(fixture.0.join("clock.org")).unwrap(),
                opened
            );
        } else {
            assert_eq!(result, Action::Redraw);
            assert!(session.echo().contains("y=save"));
            assert!(text(&session).contains(" =>  0:00"));
            assert_eq!(answer(&mut session, 'y'), Action::Exit);
            assert_eq!(
                fs::read_to_string(fixture.0.join("clock.org")).unwrap(),
                text(&session)
            );
        }
    }
}

// Closing the clock owner cannot silently drop its interval. Both choices still
// use the ordinary per-buffer save/discard flow; C-g retains all source/views.
#[test]
fn clock_close_buffer() {
    let fixture = Fixture::new();
    let mut session = owned(&fixture);
    command(&mut session, C::ClockIn);
    let opened = text(&session);
    session.execute(Command::Buffer(BufferCommand::Kill));
    argument(&mut session, "");
    assert!(session.echo().contains("detach"));
    cancel(&mut session);
    assert_eq!(text(&session), opened);
    session.execute(Command::Buffer(BufferCommand::Kill));
    argument(&mut session, "");
    argument(&mut session, "keep");
    assert!(session.echo().contains("y=save"));
    answer(&mut session, 'y');
    assert_eq!(
        fs::read_to_string(fixture.0.join("clock.org")).unwrap(),
        opened
    );
    assert!(!text(&session).contains("CLOCK:"));
    command(&mut session, C::ClockGoto);
    assert!(session.echo().contains("No active"));
}

// A stale active locator cannot close an unrelated range during quit. The
// failed close leaves source intact; an explicit keep can detach unresolved time.
#[test]
fn clock_stale_exit() {
    let fixture = Fixture::new();
    let mut session = owned(&fixture);
    command(&mut session, C::ClockIn);
    session.replace(0..0, "changed").unwrap();
    let source = text(&session);
    session.execute(Command::Quit);
    argument(&mut session, "close");
    assert!(session.echo().contains("source changed"));
    assert_eq!(text(&session), source);
    session.execute(Command::Quit);
    argument(&mut session, "keep");
    cancel(&mut session);
    assert_eq!(text(&session), source);
    command(&mut session, C::ClockGoto);
    assert!(session.echo().contains("No active"));
}
