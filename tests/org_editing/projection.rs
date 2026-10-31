use rev::buffer::edit::Point;
use rev::coordinates::{Cell, Coordinates, Scalar};
use rev::display::{Frame, Position};
use rev::org::{Cancellation, ColorScan, Limits};
use rev::window::Area;

use super::folds::{buffer, range};
use super::*;

const SOURCE: &str = "* A\nhidden1\nhidden2\n* B 界e\u{301}\n:LOGBOOK:\nsecret\n:END:\n* C\n";

// Painting, numbering, selection and hit testing use the exact same visible
// row map. Labels/status/regions retain logical source identity across gaps.
#[test]
fn source_display() {
    let (mut buffers, id) = buffer(SOURCE);
    let buffer = buffers.get_mut(id).unwrap();
    let colors = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    buffer.install_org_coloring(colors).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let point = coordinates.from_cell(Cell { row: 3, column: 2 }).unwrap();
    buffer
        .activate_mark(Point {
            stamp: buffer.stamp(),
            position: Scalar(2),
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = point;
    view.set_folds(buffer, &[range(buffer, 1..3), range(buffer, 5..6)])
        .unwrap();
    let size = Size {
        width: 40,
        height: 10,
    };
    let frame = Frame::project(buffer, &view, size, "").unwrap();
    for (local, label, text) in [
        (0, "   1 │", "* A"),
        (1, "   4 │", "* B 界e\u{301}"),
        (2, "   5 │", ":LOGBOOK:"),
        (3, "   7 │", ":END:"),
        (4, "   8 │", "* C"),
    ] {
        assert!(
            frame.rows()[local]
                .text()
                .starts_with(&format!("{label} {text}"))
        );
    }
    assert!(
        !frame
            .rows()
            .iter()
            .any(|row| row.text().contains("hidden") || row.text().contains("secret"))
    );
    assert_eq!(frame.cursor(), Some(Position { row: 1, column: 9 }));
    assert_eq!(frame.rows()[1].selection(), Some(7..9));
    assert!(!frame.rows()[1].syntax().is_empty());
    assert!(frame.rows()[8].text().contains("L4 C3"));
    let area = Area::new(0, 0, 40, 8).unwrap();
    let hit = view
        .hit(buffer, area, Position { row: 1, column: 9 })
        .unwrap()
        .unwrap();
    assert_eq!(hit.position, point);
    // Both the number separator and the independent marker reject source hits.
    for column in [5, 6] {
        assert_eq!(
            view.hit(buffer, area, Position { row: 1, column }).unwrap(),
            None
        );
    }
    assert_eq!(
        view.hit(buffer, area, Position { row: 6, column: 9 })
            .unwrap(),
        None
    );
    let region = buffer.region(view.point).unwrap().unwrap();
    assert!(
        buffer
            .slice(&region)
            .unwrap()
            .to_string()
            .contains("hidden2")
    );
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}

fn folded() -> Session {
    let (buffers, id) = buffer(SOURCE);
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.set_folds(buffer, &[range(buffer, 1..3), range(buffer, 5..6)])
        .unwrap();
    Session::new(
        buffers,
        view,
        Size {
            width: 60,
            height: 12,
        },
    )
    .unwrap()
}

// Visible search overlays project gaps; a hidden match reveals only its range.
// Cancellation restores the original independent view, not a guessed row index.
#[test]
fn search_reveal() {
    let mut session = folded();
    let original = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    for (query, visible, row) in [("* C", 6, 4), ("hidden2", 8, 2)] {
        session.execute(Command::SearchForward);
        session.handle(Event::Paste(query.into()));
        let frame = session.frame().unwrap();
        assert_eq!(
            session.view().visible_lines(session.buffer().unwrap()),
            visible,
            "{query}"
        );
        assert!(frame.rows()[row].search().is_some(), "{query}");
        session.execute(Command::KeyboardQuit);
        assert_eq!(session.view(), &original, "{query}");
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), SOURCE);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Native vertical/page/viewport motion advances displayed rows while keeping
// canonical points. Targeted source navigation still reveals hidden source.
#[test]
fn visible_motion() {
    let mut session = folded();
    session.execute(Command::Scroll(rev::command::Scroll::NextLine));
    let cell = session.view().cell(session.buffer().unwrap()).unwrap();
    assert_eq!(cell.row, 3);
    assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 6);
    assert_eq!(session.frame().unwrap().cursor().unwrap().row, 1);
    session.execute(Command::Scroll(rev::command::Scroll::PreviousLine));
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        0
    );
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("3".into()));
    session.handle(key(KeyCode::Enter));
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        2
    );
    assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 8);
    assert_eq!(session.frame().unwrap().cursor().unwrap().row, 2);
}
