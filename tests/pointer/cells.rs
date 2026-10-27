use rev::buffer::{Buffer, Buffers};
use rev::coordinates::Scalar;
use rev::display::{Frame, Position, Size};
use rev::gutter::{Change, Side};
use rev::view::View;
use rev::window::Area;

fn hit(view: &View, buffer: &Buffer, column: u16, row: u16) -> Option<usize> {
    view.hit(
        buffer,
        Area::new(0, 0, 40, 5).unwrap(),
        Position { column, row },
    )
    .unwrap()
    .map(|point| {
        assert_eq!(point.stamp, buffer.stamp());
        point.position.0
    })
}

// Display cells inside complete tab/wide/control/ZWJ clusters select their
// source start, never an interior Unicode scalar or CRLF byte.
#[test]
fn glyph_boundaries() {
    let mut buffers = Buffers::new().unwrap();
    for (name, text, expected) in [
        (
            "tab and clusters",
            "a\t界e\u{301}👩‍💻\r\nz",
            &[0, 1, 1, 1, 2, 2, 3, 5, 5, 8][..],
        ),
        ("control notation", "a\x1bb", &[0, 1, 1, 2, 3][..]),
        ("leading combining", "\u{301}x", &[0, 1, 2][..]),
    ] {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let view = View::new(buffer);
        for (column, &scalar) in expected.iter().enumerate() {
            assert_eq!(
                hit(&view, buffer, 6 + column as u16, 0),
                Some(scalar),
                "{name}/{column}"
            );
        }
        let mut bytes = Vec::new();
        rev::file::write(buffer, &mut bytes).unwrap();
        assert_eq!(bytes, text.as_bytes());
        assert!(buffer.history().is_empty());
    }
}

// A represented EOL, including the empty final row, is a real boundary.
// Trailing fill and rows beyond EOF have no source destination.
#[test]
fn eof_padding() {
    let mut buffers = Buffers::new().unwrap();
    for (name, text, row, column, expected) in [
        ("empty", "", 0, 0, Some(0)),
        ("empty fill", "", 0, 1, None),
        ("crlf eol", "ab\r\n", 0, 2, Some(2)),
        ("line fill", "ab\r\n", 0, 3, None),
        ("final empty", "ab\r\n", 1, 0, Some(4)),
        ("final fill", "ab\r\n", 1, 1, None),
        ("beyond eof", "ab\r\n", 2, 0, None),
        ("unterminated", "ab", 0, 2, Some(2)),
    ] {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(
            hit(&View::new(buffer), buffer, 6 + column, row),
            expected,
            "{name}"
        );
    }
}

// Rendered edge fragments are blank fill; a press there may focus but cannot
// jump behind the viewport. Complete neighboring glyphs retain their mapping.
#[test]
fn clipped_cells() {
    let mut buffers = Buffers::new().unwrap();
    for (name, text, left, width, rendered, expected) in [
        (
            "left tab",
            "a\tb",
            2,
            10,
            "  b ",
            &[None, None, Some(2), Some(3)][..],
        ),
        (
            "left wide",
            "a界b",
            2,
            9,
            " b ",
            &[None, Some(2), Some(3)][..],
        ),
        (
            "right wide",
            "ab界",
            0,
            9,
            "ab ",
            &[Some(0), Some(1), None][..],
        ),
        (
            "right tab",
            "ab\t",
            0,
            9,
            "ab ",
            &[Some(0), Some(1), None][..],
        ),
    ] {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.left = left;
        let frame = Frame::project(buffer, &view, Size { width, height: 5 }, "").unwrap();
        assert_eq!(
            frame.rows()[0].text().chars().skip(6).collect::<String>(),
            rendered,
            "{name}"
        );
        let content = Area::new(0, 0, width, 3).unwrap();
        for (column, &expected) in expected.iter().enumerate() {
            let point = view
                .hit(
                    buffer,
                    content,
                    Position {
                        column: 6 + column as u16,
                        row: 0,
                    },
                )
                .unwrap();
            assert_eq!(point.map(|p| p.position.0), expected, "{name}/{column}");
        }
    }
}

// Current tab policy, both gutter sides and nonzero row/column origins must
// agree. Hiding or suppressing the gutter gives the same cells back to text.
#[test]
fn policy_origins() {
    let fixture = super::config_fixture::Fixture::new();
    for width in [1, 4, 8, 32] {
        let config = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let mut buffers = Buffers::with_settings(rev::config::prepare(&config).snapshot).unwrap();
        let id = buffers.create("head\nx\t界\n").unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.top = 1;
        for side in [Side::Left, Side::Right] {
            view.gutter.apply(Change::Side(side));
            let content = Area::new(2, 3, 44, 3).unwrap();
            let origin = if side == Side::Left { 8 } else { 2 };
            let stop = width.max(2) as u16;
            let point = view
                .hit(
                    buffer,
                    content,
                    Position {
                        column: origin + stop + 1,
                        row: 3,
                    },
                )
                .unwrap()
                .unwrap();
            assert_eq!(point.position, Scalar(7));
        }
        view.gutter.apply(Change::Visible(false));
        assert_eq!(hit(&view, buffer, 0, 0), Some(5));
        view.gutter.apply(Change::Visible(true));
        assert_eq!(
            view.hit(
                buffer,
                Area::new(2, 3, 3, 1).unwrap(),
                Position { column: 2, row: 3 }
            )
            .unwrap()
            .unwrap()
            .position,
            Scalar(5)
        );
    }
}

// Old revisions, foreign buffers and overflowing viewport origins reject
// before they can publish an unchecked point or mutate a view.
#[test]
fn checked_candidates() {
    use rev::buffer::edit::Replacement;

    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("abc").unwrap();
    let foreign = buffers.create("xyz").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    assert!(
        view.hit(
            buffers.get(foreign).unwrap(),
            Area::new(0, 0, 40, 5).unwrap(),
            Position { column: 6, row: 0 }
        )
        .is_err()
    );
    let buffer = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "!".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert!(
        view.hit(
            buffer,
            Area::new(0, 0, 40, 5).unwrap(),
            Position { column: 6, row: 0 }
        )
        .is_err()
    );
    let mut current = View::new(buffer);
    current.left = usize::MAX;
    assert!(
        current
            .hit(
                buffer,
                Area::new(0, 0, 40, 5).unwrap(),
                Position { column: 7, row: 0 }
            )
            .is_err()
    );
    current.left = 0;
    current.top = usize::MAX;
    assert!(
        current
            .hit(
                buffer,
                Area::new(0, 0, 40, 5).unwrap(),
                Position { column: 6, row: 1 }
            )
            .is_err()
    );
    assert_eq!(view.point.position, Scalar(0));
}
