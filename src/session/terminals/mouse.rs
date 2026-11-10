use alacritty_terminal::term::TermMode;
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

/// Encode exactly one admitted child event. Coordinates arrive clipped to its
/// grid; classic X10 coordinates have a smaller representable domain than SGR.
pub(super) fn encode(event: MouseEvent, mode: TermMode) -> Option<Vec<u8>> {
    let (mut button, release) = button(event.kind, mode)?;
    button += modifiers(event.modifiers);
    let column = u32::from(event.column) + 1;
    let row = u32::from(event.row) + 1;
    if mode.contains(TermMode::SGR_MOUSE) {
        let end = if release { 'm' } else { 'M' };
        return Some(format!("\x1b[<{button};{column};{row}{end}").into_bytes());
    }
    if release {
        button = 3 + modifiers(event.modifiers);
    }
    legacy(button, column, row, mode.contains(TermMode::UTF8_MOUSE))
}

fn button(kind: MouseEventKind, mode: TermMode) -> Option<(u8, bool)> {
    match kind {
        MouseEventKind::Down(button) => Some((number(button), false)),
        MouseEventKind::Up(button) => Some((number(button), true)),
        MouseEventKind::Drag(button)
            if mode.intersects(TermMode::MOUSE_DRAG | TermMode::MOUSE_MOTION) =>
        {
            Some((number(button) + 32, false))
        }
        MouseEventKind::Moved if mode.contains(TermMode::MOUSE_MOTION) => Some((35, false)),
        _ => wheel(kind),
    }
}

fn wheel(kind: MouseEventKind) -> Option<(u8, bool)> {
    match kind {
        MouseEventKind::ScrollUp => Some((64, false)),
        MouseEventKind::ScrollDown => Some((65, false)),
        MouseEventKind::ScrollLeft => Some((66, false)),
        MouseEventKind::ScrollRight => Some((67, false)),
        _ => None,
    }
}

fn number(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    }
}

fn modifiers(modifiers: KeyModifiers) -> u8 {
    4 * u8::from(modifiers.contains(KeyModifiers::SHIFT))
        + 8 * u8::from(modifiers.contains(KeyModifiers::ALT))
        + 16 * u8::from(modifiers.contains(KeyModifiers::CONTROL))
}

fn legacy(button: u8, column: u32, row: u32, utf8: bool) -> Option<Vec<u8>> {
    let limit = if utf8 { 2015 } else { 223 };
    if column > limit || row > limit {
        return None;
    }
    let mut bytes = b"\x1b[M".to_vec();
    bytes.push(button + 32);
    if utf8 {
        for coordinate in [column + 32, row + 32] {
            let mut encoded = [0; 4];
            bytes.extend(
                char::from_u32(coordinate)?
                    .encode_utf8(&mut encoded)
                    .as_bytes(),
            );
        }
    } else {
        bytes.extend([
            u8::try_from(column).expect("legacy mouse column is at most 223") + 32,
            u8::try_from(row).expect("legacy mouse row is at most 223") + 32,
        ]);
    }
    Some(bytes)
}
