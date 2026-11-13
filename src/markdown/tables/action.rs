use crate::command::MarkdownCommand as C;

use super::*;

impl Work {
    pub(super) fn operate(&self, command: C, cancel: &Cancellation) -> Result<Edit, String> {
        let mut shape = self.table.shape.clone()?;
        let origin = if command == C::TableCreate {
            (0, 0, 0)
        } else {
            self.position(&shape)
        };
        let target = match structure::destination(command, origin, &mut shape, cancel)? {
            Some(target) => target,
            None => destination(command, origin, &shape),
        };
        if target.0 == shape.rows.len() {
            append(&mut shape)?;
        }
        let layout = self.action_layout(command, &shape, cancel)?;
        check(cancel)?;
        self.plan(layout, target)
    }

    fn action_layout(
        &self,
        command: C,
        shape: &Shape,
        cancel: &Cancellation,
    ) -> Result<layout::Layout, String> {
        let mut layout = self.layout(shape, cancel)?;
        if command == C::TableCreate {
            layout.isolate(self);
            self.admit_created(&layout, shape, cancel)?;
        }
        Ok(layout)
    }

    fn position(&self, shape: &Shape) -> (usize, usize, usize) {
        let row = shape
            .rows
            .iter()
            .position(|row| row.bytes.contains(&self.point))
            .unwrap_or(shape.rows.len() - 1);
        let cells = &shape.rows[row].cells;
        let column = cells
            .iter()
            .position(|cell| self.point <= cell.end)
            .unwrap_or(shape.alignments.len() - 1);
        let offset = cells.get(column).map_or(0, |cell| {
            let at = self.point.clamp(cell.start, cell.end);
            self.text.byte_slice(cell.start..at).len_chars()
        });
        (row, column, offset)
    }
}

fn destination(command: C, at: (usize, usize, usize), shape: &Shape) -> (usize, usize, usize) {
    match command {
        C::TableCreate | C::TableAlign => at,
        C::TableNextField => next_field(at, shape.alignments.len()),
        C::TablePreviousField => previous_field(at, shape.alignments.len()),
        C::TableNextRow | C::Return => (next_row(at.0), at.1, 0),
        _ => unreachable!("table work admits only table field actions"),
    }
}

fn next_field(at: (usize, usize, usize), columns: usize) -> (usize, usize, usize) {
    if at.1 + 1 < columns && at.0 != 1 {
        (at.0, at.1 + 1, 0)
    } else {
        (next_row(at.0), 0, 0)
    }
}

fn previous_field(at: (usize, usize, usize), columns: usize) -> (usize, usize, usize) {
    match at {
        (1, column, _) => (0, column, 0),
        (row, column, _) if column > 0 => (row, column - 1, 0),
        (row, _, _) if row > 1 => (if row == 2 { 0 } else { row - 1 }, columns - 1, 0),
        _ => (0, 0, 0),
    }
}

fn next_row(row: usize) -> usize {
    if row < 2 { 2 } else { row + 1 }
}

fn append(shape: &mut Shape) -> Result<(), String> {
    observe::dimensions(shape.alignments.len(), shape.rows.len() - 1)?;
    let last = shape.rows.last().expect("admitted table has delimiter");
    let prefix = last.prefix.clone();
    let end = last.ending.clone();
    shape.rows.push(Row {
        bytes: end.end..end.end,
        prefix,
        ending: end,
        cells: Vec::new(),
    });
    Ok(())
}
