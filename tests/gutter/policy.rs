use rev::gutter::{Change, CurrentLine, Format, Mode, Overrides, Policy, Settings, Side};
use rev::view::{Size, View};
use rev::window::{Error, Split, Windows};

use super::Buffers;

pub(super) fn startup() -> Settings {
    Settings {
        visible: false,
        mode: Mode::Relative,
        current_line: CurrentLine::Zero,
        side: Side::Right,
        format: Format::new(2, 2, 3).unwrap(),
    }
}

// Each leaf records explicit provenance even at its startup value. Mode changes
// retain hidden state; repeated updates are inert and reset restores all leaves.
#[test]
fn leaf_overrides() {
    let startup = startup();
    let mut policy = Policy::new(startup);
    for (name, change, expected, changed) in [
        (
            "startup-equal",
            Change::Visible(false),
            Overrides {
                visible: Some(false),
                ..Overrides::default()
            },
            true,
        ),
        (
            "mode",
            Change::Mode(Mode::Absolute),
            Overrides {
                visible: Some(false),
                mode: Some(Mode::Absolute),
                ..Overrides::default()
            },
            true,
        ),
        (
            "current",
            Change::CurrentLine(CurrentLine::Hidden),
            Overrides {
                visible: Some(false),
                mode: Some(Mode::Absolute),
                current_line: Some(CurrentLine::Hidden),
                side: None,
            },
            true,
        ),
        (
            "side",
            Change::Side(Side::Left),
            Overrides {
                visible: Some(false),
                mode: Some(Mode::Absolute),
                current_line: Some(CurrentLine::Hidden),
                side: Some(Side::Left),
            },
            true,
        ),
        (
            "repeat",
            Change::Side(Side::Left),
            Overrides {
                visible: Some(false),
                mode: Some(Mode::Absolute),
                current_line: Some(CurrentLine::Hidden),
                side: Some(Side::Left),
            },
            false,
        ),
        ("reset", Change::Reset, Overrides::default(), true),
        ("reset-again", Change::Reset, Overrides::default(), false),
    ] {
        assert_eq!(policy.apply(change), changed, "{name}");
        assert_eq!(policy.overrides(), expected, "{name}");
        assert_eq!(policy.startup(), startup);
        let effective = policy.effective();
        assert!(!effective.visible, "{name}");
        assert_eq!(effective.mode, expected.mode.unwrap_or(startup.mode));
        assert_eq!(
            effective.current_line,
            expected.current_line.unwrap_or(startup.current_line)
        );
        assert_eq!(effective.side, expected.side.unwrap_or(startup.side));
        assert_eq!(effective.format, startup.format);
    }
    assert_eq!(policy.effective(), startup);
}

// A split copies values and provenance by value. Independent updates/reset and
// deletion cannot affect its peer, and foreign/closed ViewIds are rejected.
#[test]
fn split_policy() {
    for direction in [Split::Below, Split::Right] {
        split_case(direction);
    }
}

fn split_case(direction: Split) {
    let buffers = Buffers::new().unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    let mut view = View::new(buffer);
    view.gutter = Policy::new(startup());
    view.gutter.apply(Change::Mode(Mode::Absolute));
    let mut windows = Windows::new(view).unwrap();
    let original = windows.selected_id();
    let before = windows.selected().clone();
    let peer = windows
        .split(
            direction,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap();
    assert_eq!(windows.get(peer).unwrap(), &before);
    for change in [
        Change::Visible(true),
        Change::Mode(Mode::Relative),
        Change::CurrentLine(CurrentLine::Hidden),
        Change::Side(Side::Left),
    ] {
        assert!(windows.change_gutter(peer, change).unwrap());
        assert_eq!(windows.get(original).unwrap(), &before);
    }
    assert!(windows.change_gutter(peer, Change::Reset).unwrap());
    assert_eq!(windows.get(peer).unwrap().gutter, Policy::new(startup()));
    let foreign = Windows::new(View::new(buffer)).unwrap().selected_id();
    assert_eq!(
        windows.change_gutter(foreign, Change::Reset),
        Err(Error::Missing(foreign))
    );
    assert_eq!(windows.delete_selected().unwrap(), peer);
    assert_eq!(windows.get(original), Err(Error::Missing(original)));
    assert_eq!(
        windows.change_gutter(original, Change::Visible(true)),
        Err(Error::Missing(original))
    );
    assert_eq!(windows.selected().gutter, Policy::new(startup()));
    assert_eq!(windows.iter().len(), 1);
    assert_eq!(windows.delete_others(), 0);
    assert!(buffer.history().is_empty());
}
