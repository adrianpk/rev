use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use rev::buffer::Buffers;
use rev::command::{Command, Split, TreeCommand};
use rev::coordinates::Scalar;
use rev::display::{Position, Size};
use rev::session::{Action, Session};
use rev::view::View;

pub(super) fn size() -> Size {
    Size {
        width: 80,
        height: 12,
    }
}

pub(super) fn editor(source: &str) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(buffers, view, size()).unwrap()
}

pub(super) fn lines(count: usize) -> String {
    (0..count).map(|row| format!("line-{row:02}\n")).collect()
}

pub(super) fn wheel(backward: bool, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: if backward {
            MouseEventKind::ScrollUp
        } else {
            MouseEventKind::ScrollDown
        },
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

pub(super) fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn tree_row(session: &mut Session) -> String {
    let width = usize::from(session.tree().width(size()));
    session.frame().unwrap().rows()[0]
        .text()
        .chars()
        .take(width)
        .collect()
}

// The inactive leaf scrolls three logical rows while editing focus, active
// navigation and keyboard prefixes remain owned by the original pane.
#[test]
fn inactive_target() {
    let mut session = editor(&lines(30));
    let active = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    let before = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    session.handle(key(KeyCode::Char('x'), KeyModifiers::CONTROL));
    let echo = session.echo().to_owned();
    assert_eq!(session.handle(wheel(false, 46, 0)), Action::Redraw);
    assert_eq!(session.windows().selected_id(), active);
    assert_eq!(session.view(), &before);
    assert_eq!(session.echo(), echo);
    let target = session.windows().get(peer).unwrap();
    assert_eq!((target.top, target.point.position), (3, Scalar(24)));
    let frame = session.frame().unwrap();
    assert!(frame.rows()[0].text().contains("line-00"));
    assert!(frame.rows()[0].text().contains("line-03"));
    assert_eq!(frame.cursor(), Some(Position { column: 6, row: 0 }));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    session.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert_eq!(session.echo(), "Quit");
    assert_eq!(session.handle(wheel(true, 46, 0)), Action::Redraw);
    let target = session.windows().get(peer).unwrap();
    assert_eq!((target.top, target.point.position), (0, Scalar(24)));
    assert_eq!(session.view(), &before);
}

// Wheel routing is independent of workspace keyboard focus in both directions:
// an inactive tree scrolls loaded rows, and tree focus survives a document wheel.
#[test]
fn tree_target() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("source.txt", &lines(30));
    for index in 0..16 {
        fixture.write(&format!("file-{index:02}"), "retained");
    }
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(buffers, view, size()).unwrap();
    session.execute(Command::Tree(TreeCommand::Toggle));
    session.execute(Command::Tree(TreeCommand::Expand));
    session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
    let root_row = tree_row(&mut session);
    let before = session.view().clone();
    let entries = session.tree().entries().len();
    assert_eq!(session.handle(wheel(false, 1, 0)), Action::Redraw);
    assert!(!session.tree().focused());
    assert_eq!(session.view(), &before);
    assert_eq!(session.tree().entries().len(), entries);
    assert_eq!(
        session.tree().selected().unwrap().path(),
        fixture.0.join("file-02")
    );
    assert!(
        session.frame().unwrap().rows()[0]
            .text()
            .contains("file-02")
    );
    session.handle(super::focus::press(Position { column: 1, row: 0 }));
    let selected = session.tree().selected().unwrap().path().to_owned();
    assert_eq!(session.handle(wheel(false, 40, 0)), Action::Redraw);
    assert!(session.tree().focused());
    assert_eq!(session.tree().selected().unwrap().path(), selected);
    assert_eq!(
        (session.view().top, session.view().point.position),
        (3, Scalar(24))
    );
    assert_eq!(
        session.frame().unwrap().cursor(),
        Some(Position { column: 0, row: 0 })
    );
    session.handle(wheel(true, 1, 0));
    assert!(session.tree().focused());
    assert_eq!(session.tree().selected().unwrap().path(), selected);
    assert_eq!(tree_row(&mut session), root_row);
    assert!(session.buffer().unwrap().history().is_empty());
}
