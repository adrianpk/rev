use rev::buffer::edit::Point;
use rev::coordinates::{Cell, Coordinates, Scalar};
use rev::display::Position;
use rev::window::Area;
use unicode_width::UnicodeWidthStr;

use super::*;

// Flat pretty prefixes conceal cells without losing source positions. Source
// presentation keeps each original star; clipping and wrapping share hit maps.
#[test]
fn source_and_pretty_geometry() {
    let source =
        "******** Heading 界e\u{301} with enough words to wrap around a narrow pane\nbody\n";
    for wrapped in [true, false] {
        for pretty in [true, false] {
            let (_, mut buffers, mut view) = setup(
                &format!("editor.soft_wrap={wrapped}\norg.headings.pretty={pretty}"),
                source,
            );
            let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
            observe(buffer, Limits::default());
            check_heading_viewports(buffer, &mut view, wrapped, pretty);
            assert_eq!(text(buffer), source);
            assert!(buffer.history().is_empty());
        }
    }
}

fn check_heading_viewports(buffer: &Buffer, view: &mut View, wrapped: bool, pretty: bool) {
    for width in [40, 80] {
        for left in if wrapped {
            vec![0]
        } else {
            vec![0, 2, 7, 8, 9]
        } {
            view.left = left;
            check_heading_projection(buffer, view, width, wrapped, pretty);
        }
    }
}

fn check_heading_projection(
    buffer: &Buffer,
    view: &mut View,
    width: u16,
    wrapped: bool,
    pretty: bool,
) {
    let left = view.left;
    for point in 0usize..=8 {
        view.point.position = Scalar(point);
        let frame = Frame::project(buffer, view, Size { width, height: 12 }, "").unwrap();
        if left == 0 {
            assert!(body(&frame, 0).starts_with(if pretty {
                "○ Heading"
            } else {
                "******** Heading"
            }));
        }
        let column = point.saturating_sub(if pretty { 7 } else { 0 });
        let expected = column.checked_sub(left).map(|column| Position {
            row: 0,
            column: 7 + u16::try_from(column).unwrap(),
        });
        assert_eq!(
            frame.cursor(),
            expected,
            "{wrapped}/{pretty}/{left}/{point}"
        );
        if let Some(cursor) = frame.cursor() {
            let hit = view
                .hit(buffer, Area::new(0, 0, width, 10).unwrap(), cursor)
                .unwrap()
                .unwrap();
            assert_eq!(
                hit.position,
                Scalar(if pretty { point.max(7) } else { point })
            );
        }
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width))
        );
    }
}

// Hidden stars remain independently reachable through source motions and copy.
// Their shared screen cell resolves pointer input to the final visible star.
#[test]
fn prefix_selection() {
    let (_fixture, mut buffers, mut view) = setup("", "*** Heading\n");
    let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
    observe(buffer, Limits::default());
    buffer
        .activate_mark(Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        })
        .unwrap();
    for point in 0usize..=3 {
        view.point.position = Scalar(point);
        let frame = Frame::project(buffer, &view, SIZE, "").unwrap();
        assert!(body(&frame, 0).starts_with("✸ Heading"));
        assert_eq!(
            frame.cursor(),
            Some(Position {
                row: 0,
                column: 7 + u16::try_from(point.saturating_sub(2)).unwrap()
            })
        );
        assert_eq!(
            frame.rows()[0].selection(),
            if point == 3 { Some(7..8) } else { None }
        );
        if point > 0 {
            let region = buffer.region(view.point).unwrap().unwrap();
            assert_eq!(
                buffer.slice(&region).unwrap().to_string(),
                "*".repeat(point)
            );
        }
    }
    let coordinates = Coordinates::new(buffer).unwrap();
    let area = Area::new(0, 0, 80, 22).unwrap();
    for column in 0..3 {
        let hit = view
            .hit(
                buffer,
                area,
                Position {
                    row: 0,
                    column: 7 + column,
                },
            )
            .unwrap()
            .unwrap();
        let source_column = usize::from(column) + 2;
        assert_eq!(hit.position, Scalar(source_column));
        assert_eq!(
            coordinates.to_cell(hit.position).unwrap(),
            Cell {
                row: 0,
                column: source_column
            }
        );
    }
    assert_eq!(text(buffer), "*** Heading\n");
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}
