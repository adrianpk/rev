//! A structural rewrite preserves explicit logical targets; it does not evaluate
//! formulas or infer that a range's moved endpoints preserve its interior cells.
use super::*;
mod axis;
mod scan;
use axis::Axis;
use scan::{Reference, Scanner};
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
            let replacements = Scanner::new(&text, cancel).rewrite(&axes).map_err(|e| {
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
            self.rows.target(row.value)?;
        }
        if let Some(column) = &reference.column {
            self.columns.target(column.value)?;
        }
        Ok(())
    }
    fn range(&self, start: &Reference, end: &Reference) -> Result<(), String> {
        if start.row.is_some() != end.row.is_some()
            || start.column.is_some() != end.column.is_some()
        {
            return Err("Range endpoints require the same absolute coordinate form".into());
        }
        if let (Some(a), Some(b)) = (&start.row, &end.row) {
            self.rows.range(a.value, b.value)?;
        }
        if let (Some(a), Some(b)) = (&start.column, &end.column) {
            self.columns.range(a.value, b.value)?;
        }
        Ok(())
    }
}
