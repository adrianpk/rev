#![forbid(unsafe_code)]

use rev::display::Position;
use rev::panel::Panel;
use rev::window::Area;

fn area(column: u16, row: u16, width: u16, height: u16) -> Area {
    Area::new(column, row, width, height).unwrap()
}

fn position(column: u16, row: u16) -> Position {
    Position { column, row }
}

// Status consumes exactly the final row of the supplied outer bounds. Empty
// surfaces remain representable and cannot produce a document cursor.
#[test]
fn panel_bounds() {
    for (width, height, reserve, rows) in [
        (7, 4, true, 3),
        (7, 4, false, 4),
        (1, 1, true, 0),
        (0, 4, true, 3),
        (7, 0, true, 0),
    ] {
        let outer = area(4, 3, width, height);
        let panel = Panel::new(outer, reserve, true);
        assert_eq!(panel.outer(), outer);
        assert_eq!(panel.content(), area(4, 3, width, rows));
        assert_eq!(
            panel.status(),
            (reserve && height > 0).then(|| area(4, 3 + rows, width, 1))
        );
        assert!(panel.active());
        assert!(!Panel::new(outer, reserve, false).active());
        assert_eq!(
            panel.position(position(0, 0)).is_some(),
            width > 0 && rows > 0
        );
    }
}

// Translation and clipping use content bounds rather than outer/status bounds.
// Offscreen, reversed and empty spans never escape through native arithmetic.
#[test]
fn cell_clipping() {
    let panel = Panel::new(area(4, 3, 6, 3), true, false);
    assert_eq!(panel.position(position(5, 1)), Some(position(9, 4)));
    assert_eq!(panel.position(position(6, 0)), None);
    assert_eq!(panel.position(position(0, 2)), None);
    let cases = [
        (0, 0..6, Some((3, 4..10))),
        (1, 2..u16::MAX, Some((4, 6..10))),
        (2, 0..3, None),
        (0, 6..9, None),
        (0, 3..3, None),
        (0, 5..2, None),
    ];
    for (row, columns, expected) in cases {
        assert_eq!(panel.clip(row, columns), expected);
    }
}

// Half-open endpoints admit native maxima without wrapping. Empty endpoints
// are valid bounds, but a real cell beyond either axis is rejected.
#[test]
fn native_bounds() {
    assert!(Area::new(1, 0, u16::MAX, 1).is_none());
    assert!(Area::new(0, 1, 1, u16::MAX).is_none());
    assert!(Area::new(u16::MAX, u16::MAX, 0, 0).is_some());
    let panel = Panel::new(area(0, 0, u16::MAX, u16::MAX), true, false);
    assert_eq!(
        panel.position(position(u16::MAX - 1, u16::MAX - 2)),
        Some(position(u16::MAX - 1, u16::MAX - 2))
    );
    assert_eq!(panel.position(position(0, u16::MAX - 1)), None);
}
