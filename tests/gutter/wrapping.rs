use rev::coordinates::{Byte, Coordinates, Scalar};
use rev::display::{Frame, Position, Size};
use rev::gutter::{CurrentLine, Mode, Policy, Settings, Side};
use rev::org::Source;
use rev::view::View;

use super::{Buffers, area, surface};

fn number(frame: &Frame, row: usize, side: Side, width: usize) -> String {
    let text: Vec<_> = frame.rows()[row].text().chars().collect();
    let start = if side == Side::Left { 0 } else { width - 5 };
    text[start..start + 4]
        .iter()
        .collect::<String>()
        .trim()
        .to_owned()
}

// Number labels and current-line emphasis belong only to the first fragment.
// Relative distances, final empty source lines and padding retain logical IDs
// under both sides and every current-line form, including point on a continuation.
#[test]
fn logical_labels() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("one two three\nnextline\n").unwrap();
    let buffer = buffers.get(id).unwrap();
    for side in [Side::Left, Side::Right] {
        for mode in [Mode::Absolute, Mode::Relative] {
            for current_line in [
                CurrentLine::Absolute,
                CurrentLine::Zero,
                CurrentLine::Hidden,
            ] {
                let mut view = View::new(buffer);
                view.bookmark_markers = false;
                view.point.position = Scalar(5);
                view.gutter = Policy::new(Settings {
                    side,
                    mode,
                    current_line,
                    ..Settings::default()
                });
                let frame = Frame::project(
                    buffer,
                    &view,
                    Size {
                        width: 12,
                        height: 10,
                    },
                    "",
                )
                .unwrap();
                let current = match current_line {
                    CurrentLine::Absolute => "1",
                    CurrentLine::Zero => "0",
                    CurrentLine::Hidden => "",
                };
                let next = if mode == Mode::Absolute { "2" } else { "1" };
                let final_line = if mode == Mode::Absolute { "3" } else { "2" };
                let labels = (0..8)
                    .map(|r| number(&frame, r, side, 12))
                    .collect::<Vec<_>>();
                assert_eq!(
                    labels,
                    [current, "", "", next, "", final_line, "", ""],
                    "{side:?}/{mode:?}/{current_line:?}"
                );
                let cells = surface::draw(&frame);
                let column = if side == Side::Left { 3 } else { 10 };
                assert!(cells[&(0, column)].bold);
                for row in 1..8 {
                    assert!(!cells[&(row, column)].bold);
                }
                assert_eq!(
                    frame.cursor(),
                    Some(Position {
                        column: if side == Side::Left { 7 } else { 1 },
                        row: 1
                    })
                );
                let size = Size {
                    width: 12,
                    height: 3,
                };
                let mut clipped = view.clone();
                clipped.reveal(buffer, size).unwrap();
                let clipped = Frame::project(buffer, &clipped, size, "").unwrap();
                assert_eq!(number(&clipped, 0, side, 12), "");
                assert!(!surface::draw(&clipped)[&(0, column)].bold);
            }
        }
    }
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}

// A scrolled continuation keeps an empty number field; source-relative labels
// skip folded lines. Changing width or folds immediately changes hit targets
// and labels through current geometry, without a retained painted hit map.
#[test]
fn fold_and_width() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("one two three\nhidden\nlast row").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(rev::buffer::Mode::Org).unwrap();
    let source = Source::new(buffer).range(Byte(14)..Byte(21)).unwrap();
    for side in [Side::Left, Side::Right] {
        let mut view = View::new(buffer);
        view.bookmark_markers = false;
        view.point.position = Scalar(5);
        view.gutter = Policy::new(Settings {
            side,
            mode: Mode::Relative,
            ..Settings::default()
        });
        view.set_folds(buffer, std::slice::from_ref(&source))
            .unwrap();
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 12,
                height: 8,
            },
            "",
        )
        .unwrap();
        assert_eq!(number(&frame, 3, side, 12), "2");
        let origin = if side == Side::Left { 6 } else { 0 };
        let pos = Position {
            column: origin + 2,
            row: 3,
        };
        assert_eq!(
            view.hit(buffer, area(0, 0, 12, 6), pos)
                .unwrap()
                .unwrap()
                .position,
            Scalar(23)
        );
        let resized = Frame::project(
            buffer,
            &view,
            Size {
                width: 14,
                height: 8,
            },
            "",
        )
        .unwrap();
        assert_eq!(number(&resized, 2, side, 14), "2");
        assert_eq!(number(&resized, 3, side, 14), "");
        assert_eq!(
            view.hit(
                buffer,
                area(0, 0, 14, 6),
                Position {
                    column: origin,
                    row: 3
                }
            )
            .unwrap()
            .unwrap()
            .position,
            Scalar(29)
        );
        view.clear_folds();
        assert_eq!(
            view.hit(buffer, area(0, 0, 12, 6), pos)
                .unwrap()
                .unwrap()
                .position,
            Scalar(16)
        );
        // Native viewport scrolling is exercised by the interaction tests;
        // this retained fragment observation also pins its source identity.
        let fragments = view.wrapped_rows(buffer, 6, 6).unwrap();
        assert_eq!(fragments[1].source.start, Scalar(4));
        assert_eq!(
            Coordinates::new(buffer)
                .unwrap()
                .to_cell(fragments[1].source.start)
                .unwrap()
                .row,
            0
        );
    }
}
