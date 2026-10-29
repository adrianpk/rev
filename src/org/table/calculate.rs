use super::*;
use crate::org::formula::{
    Program,
    eval::{Cell as NumericCell, Grid, ResultCell, Selection},
};

impl Snapshot {
    pub(super) fn calculation_requested(&self, table: &Table, command: C) -> bool {
        command.recalculation()
            || (command == C::CtrlCCtrlC && self.formula_range(table, command).is_ok())
    }
    pub(super) fn recalculate(
        &self,
        table: &Table,
        command: C,
        cancel: &Cancellation,
    ) -> Result<Plan, String> {
        let range = self.formula_range(table, command)?;
        let formula = self.text.byte_slice(range.clone()).to_string();
        let program = Program::parse(&formula, self.syntax, cancel).map_err(|e| e.to_string())?;
        let grid = table.numeric_grid(cancel)?;
        let results = program
            .evaluate_selected(
                &grid,
                self.calculation_selection(table, command)?,
                self.numeric,
                cancel,
            )
            .map_err(|e| e.to_string())?;
        self.calculation_plan(table, results, cancel)
    }
    pub(super) fn calculation_context(&self, nodes: &[structure::Raw], command: C) -> usize {
        if command != C::CtrlCCtrlC && !command.recalculation() {
            return self.point;
        }
        nodes
            .iter()
            .find(|n| n.kind == K::TableFormula && n.source.contains(&self.point))
            .and_then(|n| n.parent)
            .filter(|p| nodes[*p].kind == K::Table)
            .map_or(self.point, |p| nodes[p].content.start)
    }
    fn formula_range<'a>(&self, table: &'a Table, command: C) -> Result<&'a Range<usize>, String> {
        let chosen = if command == C::CtrlCCtrlC {
            let line = self.text.byte_to_line(self.point);
            table
                .formulas
                .iter()
                .find(|r| self.text.byte_to_line(r.start) == line)
        } else {
            table.formulas.first()
        };
        chosen.ok_or_else(|| "No TBLFM formula for this table".into())
    }
    fn calculation_selection(&self, table: &Table, command: C) -> Result<Selection, String> {
        if !matches!(command, C::TableRecalculateCell | C::TableRecalculateColumn) {
            return Ok(Selection::Table);
        }
        let at = table.position(&self.text, self.point);
        if !table.source.contains(&self.point) || table.rows[at.row].separator {
            return Err("Select an existing data field for cell/column recalculation".into());
        }
        let row = table.rows[..=at.row]
            .iter()
            .filter(|r| !r.separator)
            .count();
        let cell = NumericCell {
            row,
            column: at.column + 1,
        };
        Ok(if command == C::TableRecalculateCell {
            Selection::Cell(cell)
        } else {
            Selection::Column(cell.column)
        })
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
