use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use rev::display::{Frame, Size};
use rev::view::View;
use unicode_width::UnicodeWidthStr;

// The complete status keeps identity/state/access/position on the left and
// right-aligns language using only its own row, independently of document cells.
#[test]
fn complete_groups() {
    for (name, source, point, expected, mode) in [
        ("scratch", "", 0, "-- RW -- L1 C1", "Text"),
        ("Unicode", "\t界e\u{301}\r\n", 4, "-- RW -- L1 C8", "Text"),
    ] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.point.position = Scalar(point);
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 80,
                height: 5,
            },
            "notice",
        )
        .unwrap();
        let expected = format!("> {} {expected}", buffer.name());
        let row = frame.rows()[3].text();
        assert!(row.starts_with(&expected), "{name}: {row:?}");
        assert!(row.ends_with(mode));
        assert_eq!(row.width(), 80);
        assert_eq!(
            row,
            format!(
                "{expected}{}{mode}",
                " ".repeat(80 - expected.width() - mode.len())
            )
        );
        assert!(frame.rows()[4].text().starts_with("notice"));
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}
