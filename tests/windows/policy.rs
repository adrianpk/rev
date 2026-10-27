use rev::buffer::edit::Replacement;
use rev::config::prepare;
use rev::coordinates::Cell;
use rev::display::Frame;
use rev::window::{Split, Windows};

use super::{Buffers, Scalar, Session, View, size};
use crate::config_fixture::Fixture;

// Nested views read one changed buffer policy even though its canonical stamp
// stays unchanged. Reprojection and cache synchronization preserve source state.
#[test]
fn shared_policy() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "editor.tab_width = 8");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\tb\nrow").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "!".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.set_mark(Some(Scalar(1))).unwrap();
    buffer.set_readonly(true);
    let before = (
        buffer.stamp(),
        buffer.state(),
        buffer.mark(),
        buffer.is_modified(),
        buffer.history().len(),
        buffer.redo_history().len(),
    );
    let mut view = View::new(buffer);
    view.point.position = Scalar(3);
    view.column = Some(4);
    let mut windows = Windows::new(view).unwrap();
    let dimensions = size(41, 9);
    windows.split(Split::Right, dimensions).unwrap();
    windows.split(Split::Below, dimensions).unwrap();
    buffer.apply_settings(&startup.snapshot);
    let frame = Frame::compose(&buffers, &windows, dimensions, "").unwrap();
    let geometry = windows.geometry(dimensions).unwrap();
    for (id, area) in geometry.panes() {
        let view = windows.get(*id).unwrap();
        let buffer = buffers.get(view.point.stamp.buffer).unwrap();
        assert_eq!(view.cell(buffer).unwrap(), Cell { row: 0, column: 8 });
        let row = frame.rows()[usize::from(area.row())].text();
        let column = usize::from(area.column());
        assert_eq!(&row[column..column + 9], "!a      b");
        let mut synchronized = view.clone();
        synchronized.reveal(buffer, dimensions).unwrap();
        assert_eq!(synchronized.point, view.point);
        assert_eq!(synchronized.column, None);
        assert_eq!((synchronized.top, synchronized.left), (view.top, view.left));
    }
    let buffer = buffers.get(id).unwrap();
    assert_eq!(
        (
            buffer.stamp(),
            buffer.state(),
            buffer.mark(),
            buffer.is_modified(),
            buffer.history().len(),
            buffer.redo_history().len()
        ),
        before
    );
    assert!(buffer.is_readonly());
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap(),
        "!a\tb\nrow"
    );
}

// A cached column from width four cannot drive vertical motion under a new
// width. The refreshed goal survives a short row and returns to its real stop.
#[test]
fn policy_goals() {
    use rev::command::{Command, Scroll};

    let fixture = Fixture::new();
    for (width, goal) in [(1, 2), (2, 2), (4, 4), (8, 8), (32, 32)] {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::new().unwrap();
        let id = buffers
            .create(&format!("a\tb\nx\n{}", "0123456789".repeat(7)))
            .unwrap();
        let mut view = View::new(buffers.get(id).unwrap());
        view.point.position = Scalar(2);
        view.column = Some(4);
        buffers
            .get_mut(id)
            .unwrap()
            .apply_settings(&startup.snapshot);
        let mut session = Session::new(buffers, view, size(40, 8)).unwrap();
        session.execute(Command::Scroll(Scroll::NextLine));
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            Cell { row: 1, column: 1 }
        );
        assert_eq!(session.view().column, Some(goal));
        session.execute(Command::Scroll(Scroll::NextLine));
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            Cell {
                row: 2,
                column: goal
            }
        );
        assert_eq!(session.view().column, Some(goal));
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Changes to indentation alone cannot invalidate an unchanged display goal.
// Retiring the final buffer also retains the registry's supplied global policy.
#[test]
fn policy_lifecycle() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "editor.indent_width = 2");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.initial();
    let buffer = buffers.get_mut(id).unwrap();
    let mut view = View::new(buffer);
    view.column = Some(7);
    buffer.apply_settings(&prepare(&root).snapshot);
    view.reveal(buffer, size(12, 5)).unwrap();
    assert_eq!(view.column, Some(7));
    let root = fixture.write("config.toml", "editor.tab_width = 32");
    let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
    let kill = buffers.prepare_kill(buffers.initial()).unwrap();
    let replacement = buffers.kill(kill).unwrap();
    assert_eq!(buffers.get(replacement).unwrap().policy().tab_width, 32);
}
