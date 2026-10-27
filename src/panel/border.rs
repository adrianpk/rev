use crate::display::Position;
use crate::window::Area;

pub(super) fn glyph(borders: &[Area], position: Position) -> Option<char> {
    if !contains(borders, position) {
        return None;
    }
    let north = position
        .row
        .checked_sub(1)
        .is_some_and(|row| contains(borders, Position { row, ..position }));
    let east = position
        .column
        .checked_add(1)
        .is_some_and(|column| contains(borders, Position { column, ..position }));
    let south = position
        .row
        .checked_add(1)
        .is_some_and(|row| contains(borders, Position { row, ..position }));
    let west = position
        .column
        .checked_sub(1)
        .is_some_and(|column| contains(borders, Position { column, ..position }));
    let arms =
        u8::from(north) | (u8::from(east) << 1) | (u8::from(south) << 2) | (u8::from(west) << 3);
    // N/E/S/W bit order covers straight edges, corners and shared junctions.
    Some(
        [
            ' ', '│', '─', '└', '│', '│', '┌', '├', '─', '┘', '─', '┴', '┐', '┤', '┬', '┼',
        ][usize::from(arms)],
    )
}

fn contains(borders: &[Area], position: Position) -> bool {
    borders.iter().any(|area| {
        (area.column()..area.column() + area.width()).contains(&position.column)
            && (area.row()..area.row() + area.height()).contains(&position.row)
    })
}
