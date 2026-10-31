use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::command::{Command, Split};
use rev::display::Position;
use rev::session::Session;
use rev::view::Size;

use super::config_fixture::Fixture;
use super::focus::press;

pub(super) fn editor(fixture: &Fixture) -> Session {
    let source = fixture.write("root/source.txt", "retained\ntext");
    fixture.write("root/alpha.txt", "alpha");
    fixture.write("root/bravo.txt", "bravo");
    let mut session = Session::empty(Size {
        width: 80,
        height: 12,
    })
    .unwrap();
    session.load(&source).unwrap();
    session.load(&fixture.0.join("root")).unwrap();
    session.execute(Command::KeyboardQuit);
    session
}

pub(super) fn source(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

pub(super) fn cell(session: &mut Session, name: &str) -> Position {
    let frame = session.frame().unwrap();
    let row = frame
        .rows()
        .iter()
        .find(|row| row.text().split('│').next().unwrap().contains(name))
        .unwrap();
    Position {
        column: 1,
        row: row.row(),
    }
}

// A native primary click selects the rendered file and opens it in the remembered
// editing pane. Other views and dirty canonical source/history remain intact.
#[test]
fn clicked_file() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let original = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    session.insert('!');
    session.frame().unwrap();
    let point = session.windows().get(original).unwrap().point;
    let stamp = session.buffer().unwrap().stamp();
    let history = session.buffer().unwrap().history().len();
    let target = cell(&mut session, "bravo.txt");
    session.handle(press(target));
    assert_eq!(
        session.buffer().unwrap().file().unwrap().path(),
        fixture.0.join("root/bravo.txt")
    );
    assert_eq!(
        session.tree().selected().unwrap().path(),
        fixture.0.join("root/bravo.txt")
    );
    assert_eq!(session.windows().selected_id(), peer);
    assert!(!session.tree().focused());
    assert_eq!(source(&session), "bravo");
    assert_eq!(session.windows().get(original).unwrap().point, point);
    session.select_view(original).unwrap();
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), history);
    assert_eq!(source(&session), "!retained\ntext");
    assert_eq!(
        std::fs::read(fixture.0.join("root/source.txt")).unwrap(),
        b"retained\ntext"
    );
    assert_eq!(
        std::fs::read(fixture.0.join("root/bravo.txt")).unwrap(),
        b"bravo"
    );
}

fn mouse(kind: MouseEventKind, position: Position) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column: position.column,
        row: position.row,
        modifiers: KeyModifiers::NONE,
    })
}

// Directory and root presses use the native toggle, once per press. Blank,
// status cells focus only; dividers and later drag/release cannot activate text.
#[test]
fn directory_cells() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    fixture.write("root/child/nested.txt", "nested");
    session.execute(Command::Tree(rev::command::TreeCommand::Focus));
    session.execute(Command::Tree(rev::command::TreeCommand::Refresh));
    session.execute(Command::KeyboardQuit);
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    let root = fixture.0.join("root");
    let target = cell(&mut session, "child/");
    for expanded in [true, false, true] {
        session.handle(press(target));
        assert_eq!(
            session.tree().selected().unwrap().path(),
            root.join("child")
        );
        assert_eq!(session.tree().selected().unwrap().expanded(), expanded);
        assert_eq!(session.tree().root(), Some(root.as_path()));
        assert!(session.tree().focused());
        session.handle(mouse(
            MouseEventKind::Drag(MouseButton::Left),
            Position { column: 40, row: 2 },
        ));
        session.handle(mouse(MouseEventKind::Up(MouseButton::Left), target));
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
    }
    session.handle(press(Position { column: 1, row: 0 }));
    assert_eq!(session.tree().entries().len(), 1);
    assert!(!session.tree().selected().unwrap().expanded());
    session.execute(Command::KeyboardQuit);
    let composition = rev::panel::Composition::workspace(
        session.windows(),
        session.tree(),
        Size {
            width: 80,
            height: 12,
        },
        0,
    )
    .unwrap();
    let panel = composition.tree().unwrap();
    let status = panel.status().unwrap();
    for position in [
        Position { column: 1, row: 1 },
        Position {
            column: status.column(),
            row: status.row(),
        },
        Position {
            column: panel.content().width(),
            row: 0,
        },
    ] {
        session.handle(press(position));
        assert_eq!(session.tree().entries().len(), 1);
        assert!(!session.tree().selected().unwrap().expanded());
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
    }
    assert!(session.tree().focused());
    session.handle(press(Position { column: 1, row: 0 }));
    assert!(session.tree().selected().unwrap().expanded());
    assert_eq!(source(&session), "!retained\ntext");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Identical clipped labels cannot determine identity. The second native row
// opens its full Unicode path while the retained first file remains untouched.
#[test]
fn clipped_names() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let prefix = "a 界e\u{301}".repeat(6);
    let first = fixture.write(&format!("root/{prefix}-first.txt"), "first");
    let second = fixture.write(&format!("root/{prefix}-second.txt"), "second");
    session.execute(Command::Tree(rev::command::TreeCommand::Focus));
    session.execute(Command::Tree(rev::command::TreeCommand::Refresh));
    session.execute(Command::KeyboardQuit);
    let frame = session.frame().unwrap();
    assert_eq!(
        frame.rows()[1].text().split('│').next(),
        frame.rows()[2].text().split('│').next()
    );
    session.handle(press(Position { column: 1, row: 2 }));
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), second);
    assert_eq!(source(&session), "second");
    assert_eq!(std::fs::read(first).unwrap(), b"first");
    assert_eq!(std::fs::read(second).unwrap(), b"second");
}

// Missing, replaced and invalid files retain the clicked cached identity and
// dirty source. Activation never substitutes a new sorted row or a new file.
#[test]
fn failed_entry() {
    for failure in ["missing", "directory", "invalid"] {
        let fixture = Fixture::new();
        let mut session = editor(&fixture);
        session.insert('!');
        let target = cell(&mut session, "bravo.txt");
        let stamp = session.buffer().unwrap().stamp();
        let path = fixture.0.join("root/bravo.txt");
        match failure {
            "missing" => std::fs::remove_file(&path).unwrap(),
            "directory" => {
                std::fs::remove_file(&path).unwrap();
                std::fs::create_dir(&path).unwrap();
            }
            _ => std::fs::write(&path, b"bad\xff").unwrap(),
        }
        fixture.write("root/aaa.txt", "unrelated");
        session.handle(press(target));
        assert!(!session.echo().is_empty(), "{failure}");
        assert!(session.tree().focused(), "{failure}");
        assert_eq!(session.tree().selected().unwrap().path(), path);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(source(&session), "!retained\ntext");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(
            std::fs::read(fixture.0.join("root/source.txt")).unwrap(),
            b"retained\ntext"
        );
        assert_eq!(
            std::fs::read(fixture.0.join("root/aaa.txt")).unwrap(),
            b"unrelated"
        );
    }
}

// Wheel, resize and refresh change the current visible preorder. The next click
// uses the rendered path there, preserving the original selected editing pane.
#[test]
fn refreshed_viewport() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    for index in 0..20 {
        fixture.write(
            &format!("root/file{index:02}.txt"),
            &format!("file {index}"),
        );
    }
    session.execute(Command::Tree(rev::command::TreeCommand::Focus));
    session.execute(Command::Tree(rev::command::TreeCommand::Refresh));
    session.execute(Command::KeyboardQuit);
    session.handle(Event::Resize(100, 8));
    session.frame().unwrap();
    for _ in 0..2 {
        session.handle(super::scroll::wheel(false, 1, 0));
    }
    let frame = session.frame().unwrap();
    let row = &frame.rows()[2];
    let label = row.text().split('│').next().unwrap().trim();
    let name = label.split_whitespace().last().unwrap().to_owned();
    assert!(name.starts_with("file"));
    let remembered = session.windows().selected_id();
    session.handle(press(Position {
        column: 1,
        row: row.row(),
    }));
    assert_eq!(
        session.buffer().unwrap().file().unwrap().path(),
        fixture.0.join("root").join(&name)
    );
    assert_eq!(session.windows().selected_id(), remembered);
    fixture.write("root/aa-added.txt", "new first row");
    session.execute(Command::Tree(rev::command::TreeCommand::Focus));
    session.execute(Command::Tree(rev::command::TreeCommand::Refresh));
    session.handle(Event::Resize(80, 12));
    let target = cell(&mut session, "aa-added.txt");
    session.handle(press(target));
    assert_eq!(source(&session), "new first row");
    assert_eq!(
        session.tree().root(),
        Some(fixture.0.join("root").as_path())
    );
}
