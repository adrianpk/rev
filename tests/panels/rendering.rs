use crossterm::event::Event;
use rev::display::Frame;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::*;

fn drawn(frame: &Frame, column: u16, row: u16) -> String {
    frame.rows()[usize::from(row)]
        .text()
        .graphemes(true)
        .scan(0, |offset, glyph| {
            let start = *offset;
            *offset += glyph.width();
            Some((start, glyph))
        })
        .find(|(start, _)| *start == usize::from(column))
        .unwrap()
        .1
        .to_string()
}

// Actual nested frames consume the panel topology: shared T junctions connect
// reserved cells while pane content and status retain their own visible rows.
#[test]
fn shared_separators() {
    for (first, second, peer, junction) in [
        (Split::Right, Split::Below, false, '┤'),
        (Split::Right, Split::Below, true, '├'),
        (Split::Below, Split::Right, false, '┴'),
        (Split::Below, Split::Right, true, '┬'),
    ] {
        let mut session = nested(first, second, peer);
        session.handle(Event::Resize(15, 13));
        let composition = Composition::new(session.windows(), area(0, 0, 15, 13), 0).unwrap();
        let frame = session.frame().unwrap();
        assert_eq!(drawn(&frame, 7, 5), junction.to_string());
        assert_eq!(frame.rows().len(), 13);
        assert!(frame.rows().iter().all(|row| row.text().width() == 15));
        for (_, panel) in composition.panels() {
            let content = panel.content();
            assert_eq!(drawn(&frame, content.column(), content.row()), "a");
            let status = panel.status().unwrap();
            assert_eq!(
                drawn(&frame, status.column(), status.row()),
                if panel.active() { ">" } else { " " }
            );
        }
    }
}

// Text, syntax, region, search and accepted hardware cursor share one lower-
// right panel origin. Peer points and separator/status cells remain unaffected.
#[test]
fn projected_overlays() {
    use crate::config_fixture::Fixture;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rev::command::{Motion, RegionCommand};
    use rev::syntax::Style;

    let fixture = Fixture::new();
    let source = "\tfn target() { let text = \"界e\u{301}\"; }\r\n\tfn second() {}";
    let path = fixture.write("main.rs", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 61,
            height: 13,
        },
    )
    .unwrap();
    session.execute(Command::Split(Split::Below));
    session.select_view(session.windows().order()[1]).unwrap();
    session.execute(Command::Split(Split::Right));
    session.select_view(session.windows().order()[2]).unwrap();
    let selected = session.windows().selected_id();
    let peers = session
        .windows()
        .iter()
        .filter(|(id, _)| *id != selected)
        .map(|(id, view)| (id, view.point))
        .collect::<Vec<_>>();
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("target".into()));
    let composition = Composition::new(session.windows(), area(0, 0, 61, 13), 0).unwrap();
    let panel = composition
        .panels()
        .iter()
        .find(|(id, _)| *id == selected)
        .unwrap()
        .1;
    let row = usize::from(panel.content().row());
    let frame = session.frame().unwrap();
    assert_eq!(
        frame.rows()[row].selection(),
        panel.clip(0, 4..13).map(|(_, columns)| columns)
    );
    assert_eq!(
        frame.rows()[row].search(),
        panel.clip(0, 7..13).map(|(_, columns)| columns)
    );
    assert!(
        frame.rows()[row]
            .syntax()
            .iter()
            .any(|paint| paint.style == Style::Keyword
                && paint.columns == panel.clip(0, 4..6).unwrap().1)
    );
    assert_eq!(
        drawn(&frame, panel.content().column() + 7, panel.content().row()),
        "t"
    );
    assert!(frame.rows()[5].syntax().is_empty());
    assert!(frame.rows()[5].search().is_none());
    assert!(frame.rows()[5].selection().is_none());
    assert!(
        frame.rows()[usize::from(panel.status().unwrap().row())]
            .search()
            .is_none()
    );
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    let frame = session.frame().unwrap();
    assert_eq!(frame.cursor(), panel.position(position(13, 0)));
    assert!(frame.rows().iter().all(|row| row.search().is_none()));
    assert_eq!(
        session
            .windows()
            .iter()
            .filter(|(id, _)| *id != selected)
            .map(|(id, view)| (id, view.point))
            .collect::<Vec<_>>(),
        peers
    );
    let buffer = session.buffer().unwrap();
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap(),
        source
    );
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}

// A fully shared crossing is painted once between four real editing surfaces,
// with no boxed pane corners or overwriting of the adjacent content cells.
#[test]
fn crossed_frame() {
    let mut session = editor();
    session.execute(Command::Split(Split::Right));
    let right = session.windows().order()[1];
    session.execute(Command::Split(Split::Below));
    session.select_view(right).unwrap();
    session.execute(Command::Split(Split::Below));
    session.handle(Event::Resize(15, 13));
    let frame = session.frame().unwrap();
    assert_eq!(drawn(&frame, 7, 5), "┼");
    assert_eq!(drawn(&frame, 6, 5), "─");
    assert_eq!(drawn(&frame, 8, 5), "─");
    assert_eq!(drawn(&frame, 7, 4), "│");
    assert_eq!(drawn(&frame, 7, 6), "│");
    assert_eq!(drawn(&frame, 0, 0), "a");
    assert_eq!(drawn(&frame, 8, 6), "a");
}

// Mixed-width text clips at a pane seam without a partial wide glyph. Focus
// cycling and tiny/growing frames retain independent points, layout and clean
// source state while cursors stay inside the selected panel's content bounds.
#[test]
fn resized_surfaces() {
    use rev::command::{Motion, WindowCommand};

    let mut session = nested(Split::Right, Split::Below, false);
    session.handle(Event::Resize(11, 9));
    let frame = session.frame().unwrap();
    assert_eq!(drawn(&frame, 4, 0), " ");
    assert_eq!(drawn(&frame, 5, 0), "│");
    assert_eq!(drawn(&frame, 10, 0), " ");
    let ids = session.windows().order();
    for (index, id) in ids.iter().copied().enumerate() {
        session.select_view(id).unwrap();
        session.execute(Command::Motion(Motion::BufferStart));
        for _ in 0..index {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
    }
    let points = session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.point))
        .collect::<Vec<_>>();
    let layout = session.windows().layout().clone();
    let before = (
        session.buffer().unwrap().stamp(),
        session.buffer().unwrap().state(),
    );
    let selected = session.windows().selected_id();
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.windows().selected_id(), ids[0]);
    session.select_view(selected).unwrap();
    for dimensions in [
        Size {
            width: 0,
            height: 0,
        },
        Size {
            width: 1,
            height: 1,
        },
        Size {
            width: 7,
            height: 5,
        },
        Size {
            width: 7,
            height: 6,
        },
        Size {
            width: 11,
            height: 9,
        },
        Size {
            width: 31,
            height: 13,
        },
    ] {
        session.handle(Event::Resize(dimensions.width, dimensions.height));
        bounded_surface(&mut session, dimensions);
    }
    assert_eq!(session.windows().layout(), &layout);
    assert_eq!(session.windows().order(), ids);
    assert_eq!(session.windows().selected_id(), selected);
    assert_eq!(
        session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.point))
            .collect::<Vec<_>>(),
        points
    );
    assert_eq!(
        (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state()
        ),
        before
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}

fn bounded_surface(session: &mut Session, size: Size) {
    let composition = Composition::new(session.windows(), area(0, 0, size.width, size.height), 0);
    let frame = session.frame().unwrap();
    assert_eq!(
        frame.rows().len(),
        if size.width == 0 {
            0
        } else {
            usize::from(size.height)
        }
    );
    assert!(
        frame
            .rows()
            .iter()
            .all(|row| row.text().width() == usize::from(size.width))
    );
    if let Some(composition) = composition {
        let panel = composition
            .panels()
            .iter()
            .find(|(_, panel)| panel.active())
            .unwrap()
            .1;
        let cursor = frame.cursor().unwrap();
        assert!(cursor.column >= panel.content().column());
        assert!(cursor.row >= panel.content().row());
        assert!(cursor.column < panel.content().column() + panel.content().width());
        assert!(cursor.row < panel.content().row() + panel.content().height());
    } else {
        assert!(frame.cursor().is_none());
    }
}
