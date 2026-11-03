use crate::buffer::Buffer;
use crate::coordinates::Cell;
use crate::view::View;

/// Replace only visible ASCII prefix cells after canonical clipping/wrapping.
/// Every source star retains its cell; no cursor or hit mapping is changed.
pub(super) fn project(buffer: &Buffer, view: &View, from: Cell, width: u16, text: &mut String) {
    let Cell { row, column: left } = from;
    let policy = &view.headings;
    if !view.pretty_headings() {
        return;
    }
    let Some(level) = buffer.org_heading_level(row) else {
        return;
    };
    let right = level.min(left.saturating_add(usize::from(width)));
    if left >= right {
        return;
    }
    let cells = right - left;
    if !text
        .get(..cells)
        .is_some_and(|s| s.bytes().all(|b| b == b'*'))
    {
        return;
    }
    let index = if policy.cycle_bullets {
        (level - 1) % policy.bullets.len()
    } else {
        (level - 1).min(policy.bullets.len() - 1)
    };
    let mut prefix = if policy.hide_leading_stars { " " } else { "*" }.repeat(cells);
    if right == level {
        prefix.pop();
        prefix.push_str(&policy.bullets[index]);
    }
    text.replace_range(..cells, &prefix);
}
