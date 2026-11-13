use crate::command::MarkdownCommand as C;

use super::*;

pub(super) fn destination(
    command: C,
    at: (usize, usize, usize),
    shape: &mut Shape,
    cancel: &Cancellation,
) -> Result<Option<(usize, usize, usize)>, String> {
    match command {
        C::TableInsertRow => insert_body(shape, at, false).map(Some),
        C::MetaReturn => insert_body(shape, at, true).map(Some),
        C::TableDeleteRow => delete_body(shape, at).map(Some),
        C::TableInsertColumn => insert_column(shape, at, cancel).map(Some),
        C::TableDeleteColumn => delete_column(shape, at, cancel).map(Some),
        _ => Ok(None),
    }
}

fn insert_body(
    shape: &mut Shape,
    at: (usize, usize, usize),
    below: bool,
) -> Result<(usize, usize, usize), String> {
    if at.0 == 1 {
        return Err("Cannot insert a Markdown body row at the delimiter".into());
    }
    observe::dimensions(shape.alignments.len(), shape.rows.len() - 1)?;
    let index = if at.0 == 0 {
        2
    } else {
        at.0 + usize::from(below)
    };
    // A list header can own its opening marker. Repeat a body/delimiter prefix,
    // never that marker, and leave synthetic prefix/ending bytes unretained.
    let template = &shape.rows[index.min(shape.rows.len() - 1)];
    shape.rows.insert(
        index,
        Row {
            bytes: template.ending.end..template.ending.end,
            prefix: template.prefix.clone(),
            ending: template.ending.clone(),
            cells: Vec::new(),
        },
    );
    Ok((index, at.1, 0))
}

fn delete_body(
    shape: &mut Shape,
    at: (usize, usize, usize),
) -> Result<(usize, usize, usize), String> {
    if at.0 < 2 {
        return Err("Cannot delete a Markdown header or delimiter as a body row".into());
    }
    let removed = shape.rows.remove(at.0);
    let last = shape.rows.len() - 1;
    if at.0 > last {
        // Transfer EOF newline identity when removing the final body row.
        shape.rows[last].ending = removed.ending;
    }
    let row = at.0.min(last);
    Ok((if row == 1 { 0 } else { row }, at.1, 0))
}

fn insert_column(
    shape: &mut Shape,
    at: (usize, usize, usize),
    cancel: &Cancellation,
) -> Result<(usize, usize, usize), String> {
    let columns = shape.alignments.len();
    observe::dimensions(columns + 1, shape.rows.len() - 2)?;
    for row in &mut shape.rows {
        check(cancel)?;
        let empty = row.ending.start..row.ending.start;
        // Short GFM body rows still have logical empty fields at the right.
        row.cells.resize(columns, empty.clone());
        row.cells.insert(at.1, empty);
    }
    shape.alignments.insert(at.1, Alignment::None);
    Ok((if at.0 == 1 { 0 } else { at.0 }, at.1, 0))
}

fn delete_column(
    shape: &mut Shape,
    at: (usize, usize, usize),
    cancel: &Cancellation,
) -> Result<(usize, usize, usize), String> {
    let columns = shape.alignments.len();
    if columns == 1 {
        return Err("Cannot delete the final Markdown table column".into());
    }
    for row in &mut shape.rows {
        check(cancel)?;
        let empty = row.ending.start..row.ending.start;
        row.cells.resize(columns, empty);
        row.cells.remove(at.1);
    }
    shape.alignments.remove(at.1);
    let column = at.1.min(shape.alignments.len() - 1);
    Ok((if at.0 == 1 { 0 } else { at.0 }, column, 0))
}
