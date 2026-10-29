use super::*;
mod convert;
mod plan;
mod sort;

pub(super) fn command(command: C) -> C {
    match command {
        C::MetaLeft => C::TableMoveColumnLeft,
        C::MetaRight => C::TableMoveColumnRight,
        C::MetaUp => C::TableMoveRowUp,
        C::MetaDown => C::TableMoveRowDown,
        C::OutdentItemTree => C::TableDeleteColumn,
        C::IndentItemTree => C::TableInsertColumn,
        other => other,
    }
}
pub(super) fn structural(command: C) -> bool {
    matches!(
        command,
        C::TableMoveColumnLeft
            | C::TableMoveColumnRight
            | C::TableInsertColumn
            | C::TableDeleteColumn
            | C::TableMoveRowUp
            | C::TableMoveRowDown
            | C::TableKillRow
            | C::TableSort
            | C::TableToList
    )
}
impl Snapshot {
    pub(super) fn edit_table(
        &self,
        mut table: Table,
        command: C,
        argument: Option<String>,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        let dimensions = table.dimensions();
        if command == C::TableToList {
            return self.table_list(table, cancel).map(Output::Plan);
        }
        let mut at = table.position(&self.text, self.point);
        let ending = table.rows.last().expect("admitted table").ending.clone();
        if command == C::TableSort {
            let Some(argument) = argument else {
                return Ok(Output::Prompt);
            };
            let scope = self.sort_scope(&table, at)?;
            table.sort(&mut at, &argument, scope, cancel)?;
        } else {
            table.edit(&mut at, command, cancel)?;
        }
        if let Some(last) = table.rows.last_mut() {
            last.ending = ending;
        }
        self.finish_edit(table, at, dimensions, cancel)
            .map(Output::Plan)
    }
    fn finish_edit(
        &self,
        mut table: Table,
        at: Position,
        dimensions: (usize, usize),
        cancel: &Cancellation,
    ) -> Result<Plan, String> {
        if table.rows.is_empty() {
            return self.empty_plan(&table);
        }
        table.finish()?;
        let layout = self.layout(&table, cancel)?;
        let mut plan = self.structural_plan(&table, layout, at)?;
        self.rewrite(&table, dimensions, &mut plan, cancel)?;
        Ok(plan)
    }
}
impl Table {
    fn edit(&mut self, at: &mut Position, command: C, cancel: &Cancellation) -> Result<(), String> {
        match command {
            C::TableKillRow => {
                self.rows.remove(at.row);
                at.row = at.row.min(self.rows.len().saturating_sub(1));
                at.prefix = false;
                at.offset = 0;
                at.column = 0;
                Ok(())
            }
            C::TableMoveRowUp | C::TableMoveRowDown => self.move_row(at, command),
            _ => self.columns(at, command, cancel),
        }
    }
    fn move_row(&mut self, at: &mut Position, command: C) -> Result<(), String> {
        let next = adjacent(at.row, self.rows.len(), command == C::TableMoveRowUp)?;
        self.rows.swap(at.row, next);
        at.row = next;
        Ok(())
    }
    fn columns(
        &mut self,
        at: &mut Position,
        command: C,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if self.rows[at.row].separator {
            return Err("Select a data field for column editing".into());
        }
        let next = self.column_target(at.column, command)?;
        for row in &mut self.rows {
            check(cancel)?;
            if !row.separator {
                row.resize(self.columns);
                row.column_edit(at.column, next, command);
            }
        }
        match command {
            C::TableDeleteColumn => {
                self.columns -= 1;
                self.column_order.remove(at.column);
            }
            C::TableInsertColumn => {
                self.columns += 1;
                self.column_order.insert(at.column, None);
            }
            _ => self.column_order.swap(at.column, next),
        }
        at.column = next.min(self.columns - 1);
        if at.prefix {
            at.offset = 0;
        }
        at.prefix = false;
        Ok(())
    }
    fn column_target(&self, column: usize, command: C) -> Result<usize, String> {
        match command {
            C::TableDeleteColumn if self.columns == 1 => {
                Err("Cannot delete the final column; kill table rows instead".into())
            }
            C::TableInsertColumn if self.columns >= COLUMNS => {
                Err("Table insertion exceeds 256 columns".into())
            }
            C::TableMoveColumnLeft | C::TableMoveColumnRight => {
                adjacent(column, self.columns, command == C::TableMoveColumnLeft)
            }
            _ => Ok(column),
        }
    }
}
impl Row {
    fn resize(&mut self, columns: usize) {
        self.cells.resize_with(columns, || Cell {
            source: self.body.end..self.body.end,
            value: String::new(),
        });
    }
    fn column_edit(&mut self, column: usize, next: usize, command: C) {
        match command {
            C::TableDeleteColumn => {
                self.cells.remove(column);
            }
            C::TableInsertColumn => self.cells.insert(
                column,
                Cell {
                    source: self.body.end..self.body.end,
                    value: String::new(),
                },
            ),
            _ => self.cells.swap(column, next),
        }
    }
}
fn adjacent(index: usize, length: usize, previous: bool) -> Result<usize, String> {
    let target = if previous {
        index.checked_sub(1)
    } else {
        index.checked_add(1)
    };
    target
        .filter(|i| *i < length)
        .ok_or_else(|| "No adjacent table row or column; source unchanged".into())
}
