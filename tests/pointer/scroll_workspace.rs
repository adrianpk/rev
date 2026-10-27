use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{BufferCommand, Command, GutterCommand, Split, TreeCommand};
use rev::coordinates::Scalar;
use rev::display::Position;
use rev::panel::Composition;
use rev::session::{Action, Session};
use rev::view::View;
use rev::window::ViewId;

use super::scroll::{editor, key, lines, size, wheel};

fn origin(session: &Session, view: ViewId) -> Position {
    let layout = Composition::workspace(session.windows(), session.tree(), size(), 0).unwrap();
    let panel = layout
        .panels()
        .iter()
        .find(|(id, _)| *id == view)
        .unwrap()
        .1;
    Position {
        column: panel.content().column(),
        row: panel.content().row(),
    }
}

// Each nested leaf receives its own wheel reports despite different numbering
// policies. The active leaf and the other target retain their independent state.
#[test]
fn nested_targets() {
    let mut session = editor(&lines(30));
    let active = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let right = session.windows().order()[1];
    session.execute(Command::Split(Split::Below));
    let bottom = session.windows().order()[1];
    session.select_view(bottom).unwrap();
    session.execute(Command::Gutter(GutterCommand::Relative));
    session.execute(Command::Gutter(GutterCommand::ToggleSide));
    session.select_view(active).unwrap();
    let before = session.view().clone();
    let target = origin(&session, bottom);
    session.handle(wheel(false, target.column, target.row));
    assert_eq!(session.windows().get(bottom).unwrap().top, 3);
    assert_eq!(session.windows().get(right).unwrap().top, 0);
    let target = origin(&session, right);
    session.handle(wheel(false, target.column, target.row));
    assert_eq!(session.windows().get(right).unwrap().top, 3);
    assert_eq!(session.windows().get(bottom).unwrap().top, 3);
    assert_eq!(session.windows().selected_id(), active);
    assert_eq!(session.view(), &before);
    assert_eq!(
        session.frame().unwrap().cursor(),
        Some(Position { column: 6, row: 0 })
    );
}

// Wheel movement stays bound to the hovered buffer revision. Neither saved file
// is rewritten and the originally selected buffer/point cannot be substituted.
#[test]
fn different_sources() {
    let fixture = super::config_fixture::Fixture::new();
    let first = fixture.write("first.txt", &lines(30));
    let second = fixture.write("second.txt", &"right\r\n".repeat(30));
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&first).unwrap();
    let other = buffers.visit(&second).unwrap();
    let target_stamp = buffers.get(other).unwrap().stamp();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(buffers, view, size()).unwrap();
    let active = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste("second.txt".into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.select_view(active).unwrap();
    let before = session.view().clone();
    session.handle(wheel(false, 46, 0));
    assert_eq!(session.view(), &before);
    let target = session.windows().get(peer).unwrap();
    assert_eq!(
        (target.top, target.point.stamp, target.point.position),
        (3, target_stamp, Scalar(21))
    );
    session.select_view(peer).unwrap();
    let mut bytes = Vec::new();
    rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
    assert_eq!(bytes, "right\r\n".repeat(30).as_bytes());
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(std::fs::read(first).unwrap(), lines(30).as_bytes());
    assert_eq!(std::fs::read(second).unwrap(), bytes);
}

// Tree wheels consume only the existing loaded preorder even after a child is
// removed externally. Endpoints clamp and no wheel expands or rereads entries.
#[test]
fn loaded_tree_bounds() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("zz.txt", &lines(30));
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
    let before = session.view().clone();
    std::fs::remove_file(fixture.0.join("file-02")).unwrap();
    session.handle(wheel(false, 1, 0));
    assert_eq!(
        session.tree().selected().unwrap().path(),
        fixture.0.join("file-02")
    );
    for _ in 0..16 {
        session.handle(wheel(false, 1, 0));
    }
    assert_eq!(session.tree().selected().unwrap().path(), path);
    assert_eq!(session.tree().entries().len(), 18);
    assert_eq!(session.handle(wheel(false, 1, 0)), Action::Wait);
    assert!(session.frame().unwrap().rows()[0].text().contains("zz.txt"));
    for _ in 0..16 {
        session.handle(wheel(true, 1, 0));
    }
    assert_eq!(
        session.tree().selected().unwrap().path(),
        fixture.0.join("file-08")
    );
    assert_eq!(session.handle(wheel(true, 1, 0)), Action::Wait);
    assert!(!session.tree().focused());
    assert_eq!(session.view(), &before);
}
