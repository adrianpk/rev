use super::*;
impl Snapshot {
    pub(super) fn operate(
        &self,
        mut table: Table,
        command: C,
        cancel: &Cancellation,
    ) -> Result<Plan, String> {
        let position = table.position(&self.text, self.point);
        let target = table.destination(position, command, self.newline)?;
        table.finish()?;
        let layout = self.layout(&table, cancel)?;
        self.plan(&table, layout, target)
    }
    pub(super) fn create(
        &self,
        nodes: &[structure::Raw],
        argument: Option<String>,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        let line = self.text.byte_to_line(self.point);
        let start = self.text.line_to_byte(line);
        let source = self.text.line(line).to_string();
        let body = source.trim_end_matches(['\r', '\n']);
        if !body.bytes().all(|b| b" \t".contains(&b)) {
            return Err("Create an Org table on a blank line".into());
        }
        if !creation_context(nodes, self.point, &self.text) {
            return Err("Literal or unsupported table creation context".into());
        }
        let Some(argument) = argument else {
            return Ok(Output::Prompt);
        };
        let (columns, rows) = dimensions(&argument)?;
        let mut table =
            self.empty_table(start..start + body.len(), body, (columns, rows), cancel)?;
        table.finish()?;
        let layout = self.layout(&table, cancel)?;
        self.plan(
            &table,
            layout,
            Position {
                row: 0,
                column: 0,
                offset: 0,
                prefix: false,
            },
        )
        .map(Output::Plan)
    }
}
impl Snapshot {
    fn empty_table(
        &self,
        source: Range<usize>,
        indent: &str,
        size: (usize, usize),
        cancel: &Cancellation,
    ) -> Result<Table, String> {
        let start = source.start;
        let (columns, rows) = size;
        let mut table = Table {
            source,
            rows: Vec::new(),
            columns,
            formulas: false,
        };
        for row in 0..rows {
            check(cancel)?;
            table.rows.push(Row::empty(
                start,
                indent,
                if row + 1 == rows { "" } else { self.newline },
                false,
                columns,
            ));
            if row == 0 {
                table.rows.push(Row::empty(
                    start,
                    indent,
                    if rows == 1 { "" } else { self.newline },
                    true,
                    columns,
                ));
            }
        }
        // The header separator needs a preceding line break even for a one-row table.
        table.rows[0].ending = self.newline.into();
        Ok(table)
    }
}

impl Table {
    fn destination(
        &mut self,
        mut at: Position,
        command: C,
        newline: &str,
    ) -> Result<Position, String> {
        if let Some(result) = self.movement(at, command, newline) {
            return result;
        }
        match command {
            C::MetaReturn => return self.insert(at.row + 1, false, at.column, newline),
            C::TableInsertRow => return self.insert(at.row, false, at.column, newline),
            C::TableInsertHline => {
                self.insert(at.row + 1, true, 0, newline)?;
            }
            C::TableBeginningField | C::TableEndField => {
                self.field_edge(&mut at, command)?;
            }
            _ => (),
        }
        Ok(at)
    }
    fn movement(
        &mut self,
        at: Position,
        command: C,
        newline: &str,
    ) -> Option<Result<Position, String>> {
        Some(match command {
            C::Cycle | C::TableNextField => self.next_field(at, newline),
            C::GlobalCycle | C::TablePreviousField => self.previous_field(at),
            C::Return | C::TableNextRow => self.next_row(at, newline),
            _ => return None,
        })
    }
    fn field_edge(&self, at: &mut Position, command: C) -> Result<(), String> {
        let cell = self.rows[at.row]
            .cells
            .get(at.column)
            .ok_or("Separator row has no field")?;
        at.offset = if command == C::TableBeginningField {
            0
        } else {
            cell.value.chars().count()
        };
        at.prefix = false;
        Ok(())
    }
    fn next_field(&mut self, at: Position, newline: &str) -> Result<Position, String> {
        if !self.rows[at.row].separator && at.column + 1 < self.columns {
            return Ok(field(at.row, at.column + 1));
        }
        if let Some(row) = (at.row + 1..self.rows.len()).find(|i| !self.rows[*i].separator) {
            return Ok(field(row, 0));
        }
        self.insert(self.rows.len(), false, 0, newline)
    }
    fn previous_field(&self, at: Position) -> Result<Position, String> {
        if !self.rows[at.row].separator && at.column > 0 {
            return Ok(field(at.row, at.column - 1));
        }
        let row = (0..at.row)
            .rev()
            .find(|i| !self.rows[*i].separator)
            .ok_or("Beginning of Org table; no previous field")?;
        Ok(field(row, self.columns - 1))
    }
    fn next_row(&mut self, at: Position, newline: &str) -> Result<Position, String> {
        if at.row + 1 < self.rows.len() && !self.rows[at.row + 1].separator {
            return Ok(field(at.row + 1, at.column));
        }
        self.insert(at.row + 1, false, at.column, newline)
    }
    fn insert(
        &mut self,
        row: usize,
        rule: bool,
        column: usize,
        newline: &str,
    ) -> Result<Position, String> {
        if self.formulas {
            return Err(
                "Row insertion requires checked formula reference rewriting; source unchanged"
                    .into(),
            );
        }
        if self.rows.len() >= ROWS || (self.rows.len() + 1) * self.columns > CELLS {
            return Err("Table insertion exceeds native row/cell limits".into());
        }
        let at = self
            .rows
            .get(row)
            .map_or(self.source.end, |r| r.source.start);
        let ending =
            if row == self.rows.len() && self.rows.last().is_some_and(|r| r.ending.is_empty()) {
                ""
            } else {
                newline
            };
        let inserted = Row::empty(at, &self.rows[0].indent, ending, rule, self.columns);
        self.rows.insert(row, inserted);
        Ok(field(row, column))
    }
}
impl Row {
    fn empty(at: usize, indent: &str, ending: &str, separator: bool, columns: usize) -> Self {
        Self {
            source: at..at,
            body: at..at,
            indent: indent.into(),
            ending: ending.into(),
            cells: Vec::new(),
            separator,
            rule_columns: columns,
        }
    }
}
fn field(row: usize, column: usize) -> Position {
    Position {
        row,
        column,
        offset: 0,
        prefix: false,
    }
}
fn dimensions(value: &str) -> Result<(usize, usize), String> {
    let value = if value.is_empty() {
        "5x2"
    } else {
        value.trim()
    };
    if value.len() > 32 {
        return Err("Table dimensions exceed 32 bytes".into());
    }
    let (columns, rows) = value
        .split_once('x')
        .ok_or("Use columns x rows, for example 2x3")?;
    let parse = |v: &str| {
        v.trim()
            .parse::<usize>()
            .map_err(|_| "Table dimensions require positive integers".to_owned())
    };
    let (columns, rows) = (parse(columns)?, parse(rows)?);
    if !valid_dimensions(columns, rows) {
        return Err("Table dimensions exceed native row/column/cell limits".into());
    }
    Ok((columns, rows))
}
fn creation_context(nodes: &[structure::Raw], at: usize, text: &Rope) -> bool {
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            n.source.start <= at && at < n.closing.as_ref().map_or(n.source.end, |r| r.end)
        })
        .all(|(i, _)| super::observe::admitted(nodes, i, text))
}

fn valid_dimensions(columns: usize, rows: usize) -> bool {
    columns > 0
        && columns <= COLUMNS
        && rows > 0
        && rows < ROWS
        && rows.saturating_mul(columns) <= CELLS
}
