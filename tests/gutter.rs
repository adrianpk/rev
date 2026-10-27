#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::gutter::{CurrentLine, Mode, Overrides, Settings, Side};
use rev::panel::Composition;
use rev::view::View;
use rev::window::{Area, Windows};

fn area(column: u16, row: u16, width: u16, height: u16) -> Area {
    Area::new(column, row, width, height).unwrap()
}

// The real view/panel boundary produces checked constant-size geometry without
// modifying the Unicode/CRLF source or including status and prompt reservations.
#[test]
fn factory_projection() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\t界e\u{301}\r\n").unwrap();
    let buffer = buffers.get(id).unwrap();
    let before = (buffer.stamp(), buffer.state());
    let windows = Windows::new(View::new(buffer)).unwrap();
    let view = windows.selected_id();
    let composition = Composition::new(&windows, area(2, 3, 80, 24), 0).unwrap();
    let projection = composition.gutter(&windows, buffer, view).unwrap().unwrap();
    assert_eq!(projection.view, view);
    assert_eq!(projection.source, buffer.stamp());
    assert_eq!((projection.lines, projection.current), (2, 0));
    assert_eq!(projection.policy, Settings::default());
    assert!(projection.policy.visible);
    assert_eq!(projection.policy.mode, Mode::Absolute);
    assert_eq!(projection.policy.current_line, CurrentLine::Absolute);
    assert_eq!(projection.policy.side, Side::Left);
    assert_eq!(windows.selected().gutter.overrides(), Overrides::default());
    assert_eq!(projection.geometry.digits(), 4);
    assert_eq!(projection.geometry.numbers(), Some(area(2, 3, 6, 22)));
    assert_eq!(projection.geometry.text(), area(8, 3, 74, 22));
    assert_eq!(composition.panels()[0].1.status(), Some(area(2, 25, 80, 1)));
    assert_eq!(composition.prompt(), area(2, 26, 80, 1));
    assert_eq!(before, (buffer.stamp(), buffer.state()));
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}
