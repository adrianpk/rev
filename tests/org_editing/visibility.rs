use rev::command::OrgCommand;
use rev::coordinates::{Cell, Coordinates};

use super::*;

const OUTLINE: &str = "* Root\nbody\n** Child\nchild body\n*** Grand\ngrand body\n** Sibling\nsib body\n* Peer\npeer body\n";

fn visible(session: &Session) -> Vec<usize> {
    let buffer = session.buffer().unwrap();
    (0..session.view().visible_lines(buffer))
        .map(|row| session.view().source_row(buffer, row).unwrap())
        .collect()
}
fn action(session: &mut Session, command: OrgCommand) {
    session.execute(Command::Org(command));
    settle(session);
}

// Native TAB cycles heading-only, direct child headings and the entire subtree.
// S-TAB cycles overview, every heading and all source; the source stays intact.
#[test]
fn native_cycles() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    settle(&mut session);
    let stamp = session.buffer().unwrap().stamp();
    for expected in [
        vec![0, 8, 9, 10],
        vec![0, 2, 6, 8, 9, 10],
        (0..11).collect(),
    ] {
        session.handle(key(KeyCode::Tab));
        settle(&mut session);
        assert_eq!(visible(&session), expected, "{}", session.echo());
    }
    for expected in [vec![0, 8, 10], vec![0, 2, 4, 6, 8, 10], (0..11).collect()] {
        session.handle(key(KeyCode::BackTab));
        settle(&mut session);
        assert_eq!(visible(&session), expected, "{}", session.echo());
    }
    assert_eq!(text(&session), OUTLINE);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Each direct heading choice reaches the same source projection as native keys.
// Drawer commands hide only the admitted body, keeping both delimiters visible.
#[test]
fn direct_choices() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    for (command, expected) in [
        (OrgCommand::Overview, vec![0, 8, 10]),
        (OrgCommand::Contents, vec![0, 2, 4, 6, 8, 10]),
        (OrgCommand::ShowAll, (0..11).collect()),
        (OrgCommand::HideSubtree, vec![0, 8, 9, 10]),
        (OrgCommand::ShowChildren, vec![0, 2, 6, 8, 9, 10]),
        (OrgCommand::ShowSubtree, (0..11).collect()),
    ] {
        action(&mut session, command);
        assert_eq!(
            visible(&session),
            expected,
            "{command:?}: {}",
            session.echo()
        );
    }
    let source = "* Root\n:LOGBOOK:\nentry\n:END:\ntext\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("2".into()));
    session.handle(key(KeyCode::Enter));
    action(&mut session, OrgCommand::HideDrawer);
    assert_eq!(visible(&session), [0, 1, 3, 4, 5]);
    action(&mut session, OrgCommand::ShowDrawer);
    assert_eq!(visible(&session), (0..6).collect::<Vec<_>>());
    action(&mut session, OrgCommand::ToggleDrawer);
    assert_eq!(visible(&session), [0, 1, 3, 4, 5]);
    assert_eq!(text(&session), source);
}

// Requests can start analysis with coloring disabled; C-g retires the queued
// intent. A new explicit request after cancellation can reuse the one owned slot.
#[test]
fn pending_controls() {
    let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", OUTLINE);
    session.execute(Command::Org(OrgCommand::Overview));
    assert!(session.remaining_input(Instant::now()).is_some());
    session.execute(Command::KeyboardQuit);
    action(&mut session, OrgCommand::Overview);
    assert_eq!(visible(&session), [0, 8, 10]);
    action(&mut session, OrgCommand::ShowAll);
    assert_eq!(visible(&session), (0..11).collect::<Vec<_>>());
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.syntax().is_empty())
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// TAB never folds a containing ancestor when the actual source is plain body,
// malformed syntax or a literal block. Source immutability is the rejection.
#[test]
fn exact_targets() {
    let source = "* Root\n#+begin_example\n,* Fake\n#+end_example\ntext\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    for line in [2, 3, 5] {
        session.execute(Command::GotoLine);
        session.handle(Event::Paste(line.to_string()));
        session.handle(key(KeyCode::Enter));
        action(&mut session, OrgCommand::Cycle);
        assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 6);
        assert_eq!(text(&session), source);
    }
    let buffer = session.buffer().unwrap();
    assert_eq!(
        session.view().cell(buffer).unwrap(),
        Coordinates::new(buffer)
            .unwrap()
            .to_cell(session.view().point.position)
            .unwrap()
    );
    assert_eq!(
        session.view().cell(buffer).unwrap(),
        Cell { row: 4, column: 0 }
    );
}
