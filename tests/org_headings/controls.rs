use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Mode;
use rev::command::OrgCommand;

use super::*;

pub(super) fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

pub(super) fn selector(session: &mut Session, name: &str, colon: bool) {
    session.handle(if colon {
        key(KeyCode::Char(':'))
    } else {
        Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
    });
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}

pub(super) fn toggle(session: &mut Session) {
    session.execute(Command::Org(OrgCommand::PrettyHeadings));
}

pub(super) fn reset(session: &mut Session) {
    session.execute(Command::Org(OrgCommand::ResetHeadingPresentation));
}

// Actual shared selectors dispatch the compiled commands under both profiles.
// Explicit choices equal to startup still retain provenance until reset.
#[test]
fn native_selectors() {
    for (profile, colon) in [("non-modal", false), ("modal", false), ("modal", true)] {
        let config = format!("editor.interaction_profile='{profile}'");
        let (fixture, mut session) = editor(&config, "*** Heading\n");
        settle(&mut session);
        let stamp = session.buffer().unwrap().stamp();
        let point = session.view().point;
        for (name, expected) in [
            ("org-pretty-headings-mode", Some(false)),
            ("org-pretty-headings-mode", Some(true)),
            ("org-reset-heading-presentation", None),
        ] {
            selector(&mut session, name, colon);
            assert_eq!(
                session.view().heading_presentation_override(),
                expected,
                "{profile}, colon={colon}"
            );
            assert_eq!(session.view().pretty_headings(), expected.unwrap_or(true));
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.view().point, point);
            assert_eq!(text(session.buffer().unwrap()), "*** Heading\n");
            let frame = session.frame().unwrap();
            assert!(body(&frame, 0).starts_with(if expected == Some(false) {
                "*** Heading"
            } else {
                "  ✸ Heading"
            }));
        }
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("config.toml")).unwrap(),
            config
        );
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// A false startup value is restored by reset; a readonly Org view admits both
// controls, while plain mode refuses them without changing its retained choice.
#[test]
fn admission_and_reset() {
    let (_fixture, mut buffers, view) = setup("org.headings.pretty=false", "** Heading\n");
    buffers
        .get_mut(view.point.stamp.buffer)
        .unwrap()
        .set_readonly(true);
    let mut session = Session::new(buffers, view, SIZE).unwrap();
    let stamp = session.buffer().unwrap().stamp();
    toggle(&mut session);
    settle(&mut session);
    assert!(session.view().pretty_headings());
    assert!(body(&session.frame().unwrap(), 0).starts_with(" ○ Heading"));
    reset(&mut session);
    assert!(!session.view().pretty_headings());
    assert_eq!(session.view().heading_presentation_override(), None);
    toggle(&mut session);
    session.execute(Command::Mode(Mode::Text));
    let origin = session.view().clone();
    for command in [
        OrgCommand::PrettyHeadings,
        OrgCommand::ResetHeadingPresentation,
    ] {
        session.execute(Command::Org(command));
        assert_eq!(session.echo(), "Native Org mode is not active");
        assert_eq!(session.view(), &origin);
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(session.buffer().unwrap().is_readonly());
}

// Canceling command entry never invokes the highlighted presentation command,
// even when it is the first exact match in a modal colon selector.
#[test]
fn cancelled_selector() {
    let (_fixture, mut session) = editor("editor.interaction_profile='modal'", "** Heading\n");
    settle(&mut session);
    let origin = session.view().clone();
    session.handle(key(KeyCode::Char(':')));
    session.handle(Event::Paste("org-pretty-headings-mode".into()));
    session.handle(key(KeyCode::Tab));
    session.frame().unwrap();
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(session.view(), &origin);
    assert_eq!(session.view().heading_presentation_override(), None);
    assert!(session.buffer().unwrap().history().is_empty());
}
