use super::*;
use crate::org::formula::{
    self, Program,
    eval::{Grid, Policy, ResultCell},
};

impl Snapshot {
    pub(super) fn recalculate(&self, table: &Table, cancel: &Cancellation) -> Result<Plan, String> {
        let range = table
            .formulas
            .first()
            .ok_or("No TBLFM formula for this table")?;
        let formula = self.text.byte_slice(range.clone()).to_string();
        let program = Program::parse(&formula, formula::Limits::default(), cancel)
            .map_err(|e| e.to_string())?;
        let grid = table.numeric_grid(cancel)?;
        let results = program
            .evaluate(&grid, Policy::default(), cancel)
            .map_err(|e| e.to_string())?;
        self.calculation_plan(table, results, cancel)
    }
    fn calculation_plan(
        &self,
        table: &Table,
        results: Vec<ResultCell>,
        cancel: &Cancellation,
    ) -> Result<Plan, String> {
        if results.is_empty() {
            return Err("No formula targets in the selected table".into());
        }
        let rows: Vec<_> = table.rows.iter().filter(|r| !r.separator).collect();
        let mut edits = Vec::new();
        let mut bytes = self.text.len_bytes();
        for result in results {
            check(cancel)?;
            let target = rows
                .get(result.cell.row - 1)
                .and_then(|r| r.cells.get(result.cell.column - 1))
                .ok_or("Formula target has no existing source field; source unchanged")?;
            bytes = bytes
                .checked_sub(target.source.len())
                .and_then(|n| n.checked_add(result.text.len()))
                .filter(|n| *n <= Limits::default().bytes)
                .ok_or("Recalculation exceeds native 8 MiB source/output limit")?;
            self.gap(&mut edits, target.source.clone(), &result.text)?;
        }
        // Ordinary buffer affinity maps every point and mark. This plan cannot
        // choose identities, revisions or a source-relative cursor after edits.
        Ok(Plan {
            edits,
            cursor: None,
            retained: Vec::new(),
        })
    }
}
impl Table {
    fn numeric_grid(&self, cancel: &Cancellation) -> Result<Grid, String> {
        let mut rows = Vec::new();
        let mut headers = None;
        let last_data = self.rows.iter().rposition(|r| !r.separator).unwrap_or(0);
        for (index, row) in self.rows.iter().enumerate() {
            check(cancel)?;
            if row.separator {
                if headers.is_none() && !rows.is_empty() && index < last_data {
                    headers = Some(rows.len());
                }
            } else {
                let mut cells = Vec::new();
                for cell in &row.cells {
                    check(cancel)?;
                    cells.push(cell.value.clone());
                }
                rows.push(cells);
            }
        }
        Grid::new(rows, headers.unwrap_or(0)).map_err(|e| e.to_string())
    }
}
