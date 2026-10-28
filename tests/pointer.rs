#![forbid(unsafe_code)]

use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::input::{Input, Outcome};
use rev::pointer::{Event as PointerEvent, Kind};

#[path = "pointer/input_state.rs"]
mod input_state;

#[path = "pointer/focus.rs"]
mod focus;

#[path = "pointer/cells.rs"]
mod cells;
#[path = "configuration/fixture.rs"]
mod config_fixture;
#[path = "pointer/scroll.rs"]
mod scroll;
#[path = "pointer/scroll_bounds.rs"]
mod scroll_bounds;
#[path = "pointer/scroll_workspace.rs"]
mod scroll_workspace;
#[path = "pointer/selection.rs"]
mod selection;
#[path = "pointer/workspace.rs"]
mod workspace;

fn mouse(kind: MouseEventKind) -> MouseEvent {
    MouseEvent {
        kind,
        column: 17,
        row: 4,
        modifiers: KeyModifiers::SHIFT,
    }
}

// The native event boundary keeps terminal cells and modifiers intact; it
// admits exactly the primary-button and vertical-wheel operations in PTR01.
#[test]
fn native_admission() {
    let cases = [
        (
            "press",
            MouseEventKind::Down(MouseButton::Left),
            Some(Kind::Press),
        ),
        (
            "drag",
            MouseEventKind::Drag(MouseButton::Left),
            Some(Kind::Drag),
        ),
        (
            "release",
            MouseEventKind::Up(MouseButton::Left),
            Some(Kind::Release),
        ),
        ("wheel up", MouseEventKind::ScrollUp, Some(Kind::ScrollUp)),
        (
            "wheel down",
            MouseEventKind::ScrollDown,
            Some(Kind::ScrollDown),
        ),
        ("hover", MouseEventKind::Moved, None),
        ("wheel left", MouseEventKind::ScrollLeft, None),
        ("wheel right", MouseEventKind::ScrollRight, None),
        (
            "secondary press",
            MouseEventKind::Down(MouseButton::Right),
            None,
        ),
        (
            "secondary drag",
            MouseEventKind::Drag(MouseButton::Right),
            None,
        ),
        (
            "secondary release",
            MouseEventKind::Up(MouseButton::Right),
            None,
        ),
        (
            "middle press",
            MouseEventKind::Down(MouseButton::Middle),
            None,
        ),
        (
            "middle drag",
            MouseEventKind::Drag(MouseButton::Middle),
            None,
        ),
        (
            "middle release",
            MouseEventKind::Up(MouseButton::Middle),
            None,
        ),
    ];
    let mut input = Input::default();
    for (name, raw, kind) in cases {
        let event = PointerEvent::from_mouse(mouse(raw));
        assert_eq!(event.map(|event| event.kind), kind, "{name}");
        let expected = event.map_or(Outcome::Ignored, Outcome::Pointer);
        assert_eq!(input.handle(Event::Mouse(mouse(raw))), expected, "{name}");
        if let Some(event) = event {
            assert_eq!(
                (event.position.column, event.position.row),
                (17, 4),
                "{name}"
            );
            assert_eq!(event.modifiers, KeyModifiers::SHIFT, "{name}");
        }
    }
}

// Focused prompts keep their own input and draft; pointer events cannot become
// text, a candidate acceptance, a decision or a keyboard-focus transition.
#[test]
fn prompt_ownership() {
    for decision in [false, true] {
        let mut input = Input::default();
        if decision {
            input.decision("Replace? ").unwrap();
        } else {
            input.argument("Path: ", "retained").unwrap();
        }
        let focus = input.focus();
        for kind in [
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::ScrollDown,
        ] {
            assert_eq!(input.handle(Event::Mouse(mouse(kind))), Outcome::Ignored);
            assert_eq!(input.focus(), focus);
        }
        assert_eq!(
            input.prompt().map(|prompt| prompt.text()),
            (!decision).then_some("retained")
        );
    }
}
