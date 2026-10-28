use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, TreeCommand};
use rev::session::{Action, Session};
use rev::view::View;

use super::autoscroll::{TICK, arm};
use super::config_fixture::Fixture;
use super::scroll::{key, lines, size, wheel};
use super::selection::{drag, range};

fn configured(settings: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let path = fixture.write("config.toml", settings);
    let startup = rev::config::prepare(&path);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create(&lines(150)).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    (fixture, Session::new(buffers, view, size()).unwrap())
}

fn named_toggle(session: &mut Session) {
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste("mouse-mode".into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
}

// Startup off rejects even explicitly injected native reports. M-x changes only
// session admission; source edits to the root cannot change the frozen snapshot.
#[test]
fn startup_override() {
    let (fixture, mut session) = configured("mouse.enabled = false\nmouse.scroll_lines = 7");
    let before = session.view().clone();
    let now = Instant::now();
    arm(&mut session, now);
    session.handle_at(wheel(false, 7, 0), now);
    assert_eq!(session.view(), &before);
    assert_eq!(session.remaining_input(now), None);
    named_toggle(&mut session);
    assert!(session.mouse_enabled());
    assert_eq!(session.echo(), "Mouse mode enabled");
    fixture.write(
        "config.toml",
        "mouse.enabled = true\nmouse.scroll_lines = 1",
    );
    session.handle(wheel(false, 7, 0));
    assert_eq!(session.view().top, 7);
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(Command::named("mouse-mode"), Some(Command::MouseMode));
}

// Runtime off cancels a live outside capture, retains its canonical region and
// removes all due work. Late reports remain inert until explicitly enabled.
#[test]
fn disable_capture() {
    let (_fixture, mut session) = configured("mouse.scroll_lines = 7");
    let now = Instant::now();
    arm(&mut session, now);
    session.expire_input(now + TICK);
    assert_eq!(session.view().top, 7);
    let before = session.view().clone();
    let selected = range(&session);
    session.execute(Command::MouseMode);
    assert!(!session.mouse_enabled());
    assert_eq!(session.remaining_input(now + TICK), None);
    assert_eq!(session.expire_input(now + TICK * 100), Action::Wait);
    assert_eq!(
        session.handle_at(drag(9, 11), now + TICK * 100),
        Action::Wait
    );
    assert_eq!(session.view(), &before);
    assert_eq!(range(&session), selected);
    assert!(!session.cancel_pointer_capture());
    session.execute(Command::MouseMode);
    assert!(session.mouse_enabled());
    assert_eq!(session.handle(drag(9, 11)), Action::Wait);
    assert_eq!(session.view(), &before);
}

// Autoscroll clamps a configured maximum against each current text-area height;
// a one-row pane advances one row, while ordinary wheel uses the full setting.
#[test]
fn configured_steps() {
    for (distance, height) in [(1, 12), (100, 12), (100, 3)] {
        let (_fixture, mut session) = configured(&format!("mouse.scroll_lines = {distance}"));
        session.handle(Event::Resize(80, height));
        let now = Instant::now();
        session.handle_at(
            super::focus::press(rev::display::Position { column: 7, row: 0 }),
            now,
        );
        session.handle_at(drag(7, height - 2), now);
        session.expire_input(now + TICK);
        assert_eq!(session.view().top, distance.min(usize::from(height - 2)));
        session.cancel_pointer_capture();
        session.handle(wheel(false, 7, 0));
        assert_eq!(
            session.view().top,
            distance + distance.min(usize::from(height - 2))
        );
    }
}

// The session-wide command works from tree M-x without moving keyboard focus.
#[test]
fn tree_toggle() {
    let (_fixture, mut session) = configured("");
    session.execute(Command::Tree(TreeCommand::Toggle));
    session.execute(Command::Tree(TreeCommand::Focus));
    let selected = session.windows().selected_id();
    named_toggle(&mut session);
    assert_eq!(session.echo(), "Mouse mode disabled");
    assert!(!session.mouse_enabled());
    assert!(session.tree().focused());
    assert_eq!(session.windows().selected_id(), selected);
}
