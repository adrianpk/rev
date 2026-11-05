use rev::command::{Motion, Split, WindowCommand};

use super::controls::{reset, toggle};
use super::*;

// Splits copy explicit provenance by value. Rebinding the selected pane ignores
// its choice in plain text and restores it on returning to the native Org buffer.
#[test]
fn split_and_rebind() {
    let (fixture, mut session) = editor("", "*** Heading\n");
    settle(&mut session);
    toggle(&mut session);
    session.execute(Command::Split(Split::Right));
    assert!(
        session
            .windows()
            .iter()
            .all(|(_, v)| v.heading_presentation_override() == Some(false))
    );
    let selected = session.windows().selected_id();
    toggle(&mut session);
    for (id, view) in session.windows().iter() {
        assert_eq!(view.pretty_headings(), id == selected);
    }
    let plain = fixture.write("plain.txt", "*** Plain\n");
    session.load(&plain).unwrap();
    settle(&mut session);
    assert!(!session.buffer().unwrap().is_org());
    let frame = Frame::project(session.buffer().unwrap(), session.view(), SIZE, "").unwrap();
    assert!(body(&frame, 0).starts_with("*** Plain"));
    assert_eq!(session.view().heading_presentation_override(), Some(true));
    session.load(&fixture.0.join("notes.org")).unwrap();
    settle(&mut session);
    assert!(session.view().pretty_headings());
    reset(&mut session);
    assert_eq!(session.view().heading_presentation_override(), None);
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.view().heading_presentation_override(), Some(false));
    assert!(!session.view().pretty_headings());
    assert!(session.buffer().unwrap().history().is_empty());
}

// Native source publication and undo update both pane stamps and the observed
// level while leaving each pane's independent source/pretty preference intact.
#[test]
fn edit_and_undo() {
    let (_fixture, mut session) = editor("syntax.enabled=false", "** Heading\n");
    settle(&mut session);
    session.execute(Command::Split(Split::Right));
    toggle(&mut session);
    let selected = session.windows().selected_id();
    session.execute(Command::Motion(Motion::BufferStart));
    session.insert('*');
    settle(&mut session);
    for (id, view) in session.windows().iter() {
        let frame = Frame::project(session.buffer().unwrap(), view, SIZE, "").unwrap();
        assert_eq!(
            view.heading_presentation_override(),
            if id == selected { Some(false) } else { None }
        );
        assert!(body(&frame, 0).starts_with(if id == selected {
            "*** Heading"
        } else {
            "✸ Heading"
        }));
    }
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert!(!session.view().pretty_headings());
    assert_eq!(text(session.buffer().unwrap()), "** Heading\n");
    assert!(!session.buffer().unwrap().is_modified());
}

// A real checked disk reload is one ordinary undoable source change. Its shared
// view mapping retires old heading evidence without resetting either preference.
#[test]
fn reload_preferences() {
    let (fixture, mut session) = editor(
        "files.external_changes='ask'\nsyntax.enabled=false",
        "** Old\n",
    );
    settle(&mut session);
    session.execute(Command::Split(Split::Right));
    toggle(&mut session);
    let selected = session.windows().selected_id();
    fixture.write("notes.org", "*** New 界\n");
    session.execute(Command::RevertBuffer);
    let until = Instant::now() + Duration::from_secs(3);
    while text(session.buffer().unwrap()) != "*** New 界\n" {
        assert!(Instant::now() < until, "{}", session.echo());
        session.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(1));
    }
    settle(&mut session);
    for (id, view) in session.windows().iter() {
        assert_eq!(view.pretty_headings(), id != selected);
        let frame = Frame::project(session.buffer().unwrap(), view, SIZE, "").unwrap();
        assert!(body(&frame, 0).starts_with(if id == selected {
            "*** New 界"
        } else {
            "✸ New 界"
        }));
    }
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert!(!session.view().pretty_headings());
    assert_eq!(text(session.buffer().unwrap()), "** Old\n");
}

// Explicit re-enablement may request current structure after a canceled worker
// attempt, without editing source or introducing a background retry timer.
#[test]
fn cancelled_observation() {
    let (_fixture, mut session) = editor(
        "syntax.enabled=false\norg.headings.pretty=false",
        "** Heading\n",
    );
    settle(&mut session);
    toggle(&mut session);
    session.frame().unwrap();
    assert!(session.remaining_input(Instant::now()).is_some());
    session.execute(Command::KeyboardQuit);
    settle(&mut session);
    assert!(session.buffer().unwrap().org_highlights().is_none());
    assert_eq!(session.view().heading_presentation_override(), Some(true));
    toggle(&mut session);
    toggle(&mut session);
    settle(&mut session);
    assert!(body(&session.frame().unwrap(), 0).starts_with("○ Heading"));
    assert_eq!(text(session.buffer().unwrap()), "** Heading\n");
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.remaining_input(Instant::now()), None);
}
