use rev::gutter::{Error, Format, Geometry, Settings, Side};
use rev::window::Area;

use super::area;

// Exact accepted edges reject unsupported minima/padding instead of admitting a
// field that could truncate or silently reinterpret its startup preferences.
#[test]
fn format_bounds() {
    for (name, digits, left, right, admitted) in [
        ("smallest", 1, 0, 0, true),
        ("factory", 4, 0, 1, true),
        ("largest", 20, 4, 4, true),
        ("zero", 0, 0, 0, false),
        ("too many digits", 21, 0, 0, false),
        ("left overflow", 4, 5, 0, false),
        ("right overflow", 4, 0, 5, false),
        ("byte maximum", u8::MAX, u8::MAX, u8::MAX, false),
    ] {
        let format = Format::new(digits, left, right);
        assert_eq!(format.is_some(), admitted, "{name}");
        if let Some(format) = format {
            assert_eq!(format.minimum_digits(), digits);
            assert_eq!(format.left_padding(), left);
            assert_eq!(format.right_padding(), right);
        }
    }
}

// Complete-buffer counts determine digits through decimal boundaries and the
// largest representable index, using no fixture or allocation per possible row.
#[test]
fn decimal_width() {
    for (lines, expected) in [
        (1, 1),
        (9, 1),
        (10, 2),
        (99, 2),
        (100, 3),
        (9_999, 4),
        (10_000, 5),
        (99_999, 5),
        (100_000, 6),
        (
            usize::MAX,
            u8::try_from(usize::MAX.to_string().len()).unwrap(),
        ),
    ] {
        for minimum in [1, 4, 20] {
            let settings = Settings {
                format: Format::new(minimum, 0, 1).unwrap(),
                ..Settings::default()
            };
            let geometry = Geometry::new(area(0, 0, 80, 24), lines, settings).unwrap();
            assert_eq!(
                geometry.digits(),
                expected.max(minimum),
                "{lines}/{minimum}"
            );
            assert_eq!(
                geometry.numbers().unwrap().width(),
                u16::from(expected.max(minimum)) + 2
            );
        }
    }
    assert_eq!(
        Geometry::new(area(0, 0, 80, 24), 0, Settings::default()),
        Err(Error::LineCount)
    );
}

// Side, visibility, padding and tiny/zero geometry are crossed at ordinary and
// maximal u16 origins. Number admission has priority over the outside marker;
// adjacent fields consume exactly their widths without a dormant folding cell.
#[test]
fn partition_grid() {
    for (side, format, visible) in [
        (Side::Left, Format::default(), true),
        (Side::Right, Format::default(), true),
        (Side::Left, Format::new(20, 4, 4).unwrap(), true),
        (Side::Right, Format::new(1, 0, 0).unwrap(), true),
        (Side::Left, Format::default(), false),
        (Side::Right, Format::default(), false),
    ] {
        for lines in [1, 10_000, usize::MAX] {
            check_partition_sizes(
                lines,
                Settings {
                    side,
                    format,
                    visible,
                    ..Settings::default()
                },
            );
        }
    }
}

fn check_partition_sizes(lines: usize, settings: Settings) {
    for (width, height) in (0..=40).flat_map(|w| [0, 1, 5].map(move |h| (w, h))) {
        for markers in [false, true] {
            check_partition(area(2, 3, width, height), lines, settings, markers);
            check_partition(
                area(u16::MAX - width, u16::MAX - height, width, height),
                lines,
                settings,
                markers,
            );
        }
    }
}

fn check_partition(content: Area, lines: usize, settings: Settings, markers: bool) {
    let geometry = Geometry::with_marker(content, lines, settings, markers).unwrap();
    let field = u16::from(geometry.digits())
        + u16::from(settings.format.left_padding())
        + u16::from(settings.format.right_padding())
        + 1;
    let fits = u32::from(content.width()) >= u32::from(field) + 3;
    let numbered = settings.visible && content.height() > 0 && fits;
    assert_eq!(geometry.numbers().is_some(), numbered);
    let reserved = if numbered { field } else { 0 };
    let marked = markers && content.height() > 0 && content.width() >= reserved + 4;
    assert_eq!(geometry.marker().is_some(), marked);
    let text = geometry.text();
    assert_eq!(text.width() + reserved + u16::from(marked), content.width());
    if numbered || marked {
        assert!(text.width() >= Geometry::MIN_TEXT_COLUMNS);
    }
    check_partition_fields(content, geometry, field, settings.side);
}

fn check_partition_fields(content: Area, geometry: Geometry, field: u16, side: Side) {
    let text = geometry.text();
    let mut fields = Vec::new();
    if let Some(marker) = geometry.marker() {
        assert_eq!(marker.width(), 1);
        fields.push(marker);
    }
    if let Some(numbers) = geometry.numbers() {
        assert_eq!(numbers.width(), field);
        fields.push(numbers);
    }
    fields.push(text);
    if side == Side::Right {
        fields.reverse();
    }
    let mut column = content.column();
    for field in fields {
        assert_eq!(field.column(), column);
        assert_eq!(
            (field.row(), field.height()),
            (content.row(), content.height())
        );
        column += field.width();
    }
    assert_eq!(column, content.column() + content.width());
}
