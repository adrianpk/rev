//! Structural rewrites share native formula syntax, but require absolute target
//! identities and preserve every interior range coordinate before publication.
use super::*;
use crate::org::formula::{Component, Coordinate, Expression, Program, Reference};
mod axis;
mod emit;
use axis::Axis;
struct Axes {
    rows: Axis,
    columns: Axis,
}
impl Table {
    pub(super) fn dimensions(&self) -> (usize, usize) {
        (
            self.rows.iter().filter(|r| !r.separator).count(),
            self.columns,
        )
    }
}
impl Snapshot {
    pub(super) fn rewrite(
        &self,
        table: &Table,
        dimensions: (usize, usize),
        plan: &mut Plan,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if table.formulas.is_empty() {
            return Ok(());
        }
        let axes = Axes::new(table, dimensions)?;
        for source in &table.formulas {
            check(cancel)?;
            let text = self.text.byte_slice(source.clone()).to_string();
            let replacements = axes.rewrite(&text, cancel).map_err(|e| {
                format!("Unsupported formula reference rewriting: {e}; source unchanged")
            })?;
            for (range, value) in replacements {
                self.gap(
                    &mut plan.edits,
                    source.start + range.start..source.start + range.end,
                    &value,
                )?;
            }
        }
        Ok(())
    }
}
impl Axes {
    fn new(table: &Table, dimensions: (usize, usize)) -> Result<Self, String> {
        let rows = table
            .rows
            .iter()
            .filter(|r| !r.separator)
            .map(|r| r.old_row);
        Ok(Self {
            rows: Axis::new(dimensions.0, rows)?,
            columns: Axis::new(dimensions.1, table.column_order.iter().copied())?,
        })
    }
    fn validate(&self, reference: &Reference) -> Result<(), String> {
        if let Some(row) = &reference.row {
            self.rows.target(absolute(row)?)?;
        }
        if let Some(column) = &reference.column {
            self.columns.target(absolute(column)?)?;
        }
        Ok(())
    }
    fn range(&self, start: &Reference, end: &Reference) -> Result<(), String> {
        if start.row.is_some() != end.row.is_some()
            || start.column.is_some() != end.column.is_some()
        {
            return Err("Range endpoints require the same absolute coordinate form".into());
        }
        interval(&self.rows, &start.row, &end.row)?;
        interval(&self.columns, &start.column, &end.column)?;
        Ok(())
    }
}
fn absolute(component: &Component) -> Result<usize, String> {
    match component.coordinate {
        Coordinate::Absolute(value) => Ok(value),
        _ => Err("Current/relative coordinates are unsupported for structural rewriting".into()),
    }
}

fn interval(axis: &Axis, start: &Option<Component>, end: &Option<Component>) -> Result<(), String> {
    if let (Some(start), Some(end)) = (start, end) {
        axis.range(absolute(start)?, absolute(end)?)?;
    }
    Ok(())
}
