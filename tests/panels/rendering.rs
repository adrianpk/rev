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
