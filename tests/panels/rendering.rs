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
