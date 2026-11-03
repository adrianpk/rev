use rev::buffer::edit::Point;
use rev::coordinates::{Cell, Coordinates, Scalar};
use rev::display::Position;
use rev::window::Area;
use unicode_width::UnicodeWidthStr;

use super::*;

// Compare source and decorated frames through the same real coordinate owner:
// wrapping, cursors and hit targets must be identical at every visible cell.
#[test]
fn unchanged_geometry() {
    let source =
        "******** Heading 界e\u{301} with enough words to wrap around a narrow pane\nbody\n";
    for wrapped in [true, false] {
        let (fixture, mut buffers, mut view) =
            setup(&format!("editor.soft_wrap={wrapped}"), source);
        let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
        observe(buffer, Limits::default());
        let plain_settings = prepare(&fixture.write("plain.toml", "org.headings.pretty=false"));
        for width in [40, 80] {
            for left in if wrapped {
                vec![0]
            } else {
                vec![0, 2, 7, 8, 9]
            } {
                view.left = left;
                let mut plain = View::with_settings(buffer, &plain_settings.snapshot);
                plain.left = left;
                for point in 0..=8 {
                    view.point.position = Scalar(point);
                    plain.point.position = Scalar(point);
                    let size = Size { width, height: 12 };
                    let pretty = Frame::project(buffer, &view, size, "").unwrap();
                    let source = Frame::project(buffer, &plain, size, "").unwrap();
                    if left < 8 {
                        let suffix = body(&source, 0)[8 - left..].to_owned();
                        assert_eq!(
                            body(&pretty, 0),
                            format!("{}○{suffix}", " ".repeat(7 - left))
                        );
                    } else {
                        assert_eq!(body(&pretty, 0), body(&source, 0));
                    }
                    assert_eq!(
                        pretty.cursor(),
                        source.cursor(),
                        "wrap={wrapped}, left={left}, point={point}"
                    );
                    for row in pretty.rows() {
                        assert_eq!(row.text().width(), usize::from(width));
                    }
                    let area = Area::new(0, 0, width, 10).unwrap();
                    for row in 0..10 {
                        for column in 0..width {
                            let at = Position { row, column };
                            assert_eq!(
                                view.hit(buffer, area, at).unwrap(),
                                plain.hit(buffer, area, at).unwrap()
                            );
                        }
                    }
                    for row in 1..10 {
                        assert_eq!(pretty.rows()[row].text(), source.rows()[row].text());
                    }
                }
            }
        }
    }
}

// Concealed stars and the final marker retain separate source-backed selection
// cells, even when point enters the prefix; copying extracts the original stars.
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
    for point in 0..=3 {
        view.point.position = Scalar(point);
        let frame = Frame::project(buffer, &view, SIZE, "").unwrap();
        assert!(body(&frame, 0).starts_with("  ✸ Heading"));
        assert_eq!(
            frame.cursor(),
            Some(Position {
                row: 0,
                column: 7 + u16::try_from(point).unwrap()
            })
        );
        if point > 0 {
            assert_eq!(
                frame.rows()[0].selection(),
                Some(7..7 + u16::try_from(point).unwrap())
            );
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
        assert_eq!(hit.position, Scalar(usize::from(column)));
        assert_eq!(
            coordinates.to_cell(hit.position).unwrap(),
            Cell {
                row: 0,
                column: usize::from(column)
            }
        );
    }
    assert_eq!(text(buffer), "*** Heading\n");
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}
