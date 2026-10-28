use rev::buffer::Mode as BufferMode;
use rev::coordinates::{Byte, Coordinates, LineByte};
use rev::display::{Frame, Size};
use rev::gutter::Policy;
use rev::org::Source;

use super::*;

// Folded numbering uses canonical line identities and logical relative distance
// on either side. Current-line emphasis follows the same projected source row.
#[test]
fn source_numbers() {
    let source = (0..12)
        .map(|n| format!("* Row {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(BufferMode::Org).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let point = coordinates
        .from_line(LineByte { row: 8, column: 0 })
        .unwrap();
    let start = coordinates
        .to_byte(
            coordinates
                .from_line(LineByte { row: 1, column: 0 })
                .unwrap(),
        )
        .unwrap();
    let end = coordinates.to_byte(point).unwrap();
    let hidden = Source::new(buffer)
        .range(Byte(start.0)..Byte(end.0))
        .unwrap();
    for (name, mode, side, current, labels) in [
        (
            "absolute left",
            Mode::Absolute,
            Side::Left,
            CurrentLine::Absolute,
            [1, 9, 10],
        ),
        (
            "relative right",
            Mode::Relative,
            Side::Right,
            CurrentLine::Zero,
            [8, 0, 1],
        ),
    ] {
        let mut view = View::new(buffer);
        view.point.position = point;
        view.gutter = Policy::new(Settings {
            mode,
            side,
            current_line: current,
            ..Settings::default()
        });
        view.set_folds(buffer, std::slice::from_ref(&hidden))
            .unwrap();
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 24,
                height: 8,
            },
            "",
        )
        .unwrap();
        for (row, label) in labels.into_iter().enumerate() {
            let text = frame.rows()[row].text();
            let number = format!("{label:>4} ");
            assert!(
                match side {
                    Side::Left => text.starts_with(&format!("{number}│")),
                    Side::Right => text.ends_with(&format!("│{number}")),
                },
                "{name}: {text:?}"
            );
        }
        let column = if side == Side::Left { 3 } else { 22 };
        let cells = surface::draw(&frame);
        assert_eq!(
            cells[&(1, column)].foreground,
            Some(rev::config::Palette::default().gutter.current_line),
            "{name}"
        );
        assert_eq!(view.cell(buffer).unwrap().row, 8);
    }
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}
