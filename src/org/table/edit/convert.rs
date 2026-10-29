use super::*;
impl Snapshot {
    pub(in crate::org::table) fn list_table(
        &self,
        nodes: &[structure::Raw],
        cancel: &Cancellation,
    ) -> Result<Plan, String> {
        let id = list_at(nodes, self.point)?;
        if !super::super::observe::admitted(nodes, id, &self.text) {
            return Err("Literal or unsupported list conversion context".into());
        }
        let rows = self.list_rows(nodes, id, cancel)?;
        let first = rows
            .first()
            .ok_or("List has no complete items")?
            .source
            .start;
        let end = rows.last().expect("nonempty rows").source.end;
        let selected = rows
            .iter()
            .position(|r| r.source.contains(&self.point))
            .unwrap_or(0);
        let mut table = Table {
            source: first..end,
            rows,
            columns: 1,
            formulas: Vec::new(),
            column_order: Vec::new(),
        };
        table.finish()?;
        let mut layout = self.layout(&table, cancel)?;
        let before = super::super::action::adjacent_row(nodes, first, false);
        let after = super::super::action::adjacent_row(nodes, end, true);
        layout.separate(
            if before { self.newline } else { "" },
            if after { self.newline } else { "" },
        )?;
        self.structural_plan(
            &table,
            layout,
            Position {
                row: selected,
                column: 0,
                offset: 0,
                prefix: false,
            },
        )
    }
    fn list_rows(
        &self,
        nodes: &[structure::Raw],
        id: usize,
        cancel: &Cancellation,
    ) -> Result<Vec<Row>, String> {
        let mut rows = Vec::new();
        for node in nodes
            .iter()
            .filter(|n| n.parent == Some(id) && matches!(n.kind, K::Item { .. }))
        {
            check(cancel)?;
            rows.push(self.list_row(node)?);
            if rows.len() > ROWS {
                return Err("List conversion exceeds 10000 rows".into());
            }
        }
        if rows
            .windows(2)
            .any(|p| p[0].source.end != p[1].source.start)
        {
            return Err("List conversion requires contiguous single-line items".into());
        }
        Ok(rows)
    }
    fn list_row(&self, item: &structure::Raw) -> Result<Row, String> {
        let line = self.text.byte_to_line(item.source.start);
        let physical = self.text.line(line).to_string();
        let body = physical.trim_end_matches(['\r', '\n']);
        let end = item.source.start + body.len();
        let tail = item.source.start + physical.len();
        if !self
            .text
            .byte_slice(tail..item.source.end)
            .chars()
            .all(char::is_whitespace)
        {
            return Err("List conversion requires flat single-line items; source unchanged".into());
        }
        let content = self.text.byte_slice(item.content.start..end).to_string();
        let value = content.trim_end_matches([' ', '\t']);
        if value.contains('|') {
            return Err("List item contains a literal table delimiter; source unchanged".into());
        }
        let indent: String = body
            .chars()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .collect();
        Ok(Row {
            old_row: None,
            source: item.source.start..tail,
            body: item.source.start..end,
            indent,
            ending: physical[body.len()..].into(),
            separator: false,
            rule_columns: 1,
            cells: vec![Cell {
                source: item.content.start..item.content.start + value.len(),
                value: value.into(),
            }],
        })
    }
    pub(super) fn table_list(&self, table: Table, cancel: &Cancellation) -> Result<Plan, String> {
        if !table.formulas.is_empty() {
            return Err(
                "Conversion with stored formula source is unsupported; source unchanged".into(),
            );
        }
        if table.columns != 1 || table.rows.iter().any(|r| r.separator) {
            return Err(
                "Table-to-list requires one column without separators; source unchanged".into(),
            );
        }
        let selected = table.position(&self.text, self.point).row;
        let mut output = String::new();
        let mut anchors = Vec::new();
        let mut cursor = self.text.byte_to_char(table.source.start);
        for (index, row) in table.rows.iter().enumerate() {
            check(cancel)?;
            output.push_str(&row.indent);
            output.push_str("- ");
            if index == selected {
                cursor = self.text.byte_to_char(table.source.start) + output.chars().count();
            }
            if let Some(cell) = row.cells.first() {
                let start = output.len();
                output.push_str(&cell.value);
                if !cell.value.is_empty() {
                    anchors.push(Anchor {
                        source: cell.source.clone(),
                        output: start..output.len(),
                    });
                }
            }
            output.push_str(&row.ending);
        }
        let retained = self.retained(&output, table.source.start, anchors);
        Ok(Plan {
            edits: self.table_replacement(table.source, output, false)?,
            cursor: Some(Scalar(cursor)),
            retained,
        })
    }
}

fn list_at(nodes: &[structure::Raw], point: usize) -> Result<usize, String> {
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| matches!(n.kind, K::List { .. }) && n.source.contains(&point))
        .min_by_key(|(_, n)| n.source.len())
        .map(|(id, _)| id)
        .ok_or_else(|| "No native Org list at point".into())
}
