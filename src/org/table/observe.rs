use super::*;
use std::collections::BTreeMap;
impl Table {
    pub(super) fn observe(
        text: &Rope,
        nodes: &[structure::Raw],
        at: usize,
        cancel: &Cancellation,
    ) -> Result<Option<Self>, String> {
        let Some((id, node)) = nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.kind == K::Table && contains(&n.content, at, text.len_bytes()))
        else {
            return Ok(None);
        };
        if !admitted(nodes, id, text) {
            return Ok(None);
        }
        let mut table = Self {
            source: node.content.clone(),
            rows: Vec::new(),
            columns: 0,
            formulas: Vec::new(),
            column_order: Vec::new(),
        };
        let mut rows = BTreeMap::new();
        for (index, node) in nodes.iter().enumerate() {
            check(cancel)?;
            table.node(text, node, (id, index), &mut rows)?;
        }
        table.finish()?;
        table.identify();
        Ok(Some(table))
    }
    fn node(
        &mut self,
        text: &Rope,
        node: &structure::Raw,
        ids: (usize, usize),
        rows: &mut BTreeMap<usize, usize>,
    ) -> Result<(), String> {
        if node.parent == Some(ids.0) {
            if let K::TableRow { separator } = node.kind {
                rows.insert(ids.1, self.rows.len());
                self.rows
                    .push(Row::new(text, node.source.clone(), separator)?);
            }
            if node.kind == K::TableFormula {
                self.formulas.push(node.content.clone());
            }
        }
        if node.kind == K::TableCell
            && let Some(row) = node.parent.and_then(|p| rows.get(&p))
        {
            self.rows[*row].cells.push(Cell {
                source: node.content.clone(),
                value: text.byte_slice(node.content.clone()).to_string(),
            });
        }
        Ok(())
    }
    fn identify(&mut self) {
        self.column_order = (1..=self.columns).map(Some).collect();
        let mut number = 0;
        for row in self.rows.iter_mut().filter(|r| !r.separator) {
            number += 1;
            row.old_row = Some(number);
        }
    }
    pub(super) fn finish(&mut self) -> Result<(), String> {
        self.columns = self
            .columns
            .max(self.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0));
        if self.columns == 0 {
            self.columns = self
                .rows
                .iter()
                .map(Row::separator_columns)
                .max()
                .unwrap_or(1);
        }
        let count: usize = self.rows.iter().map(|r| r.cells.len()).sum();
        if self.rows.len() > ROWS
            || self.columns > COLUMNS
            || count > CELLS
            || self
                .rows
                .iter()
                .filter(|r| !r.separator)
                .count()
                .saturating_mul(self.columns)
                > CELLS
        {
            return Err("Table exceeds 10000 rows, 256 columns or 100000 cells".into());
        }
        if self.rows.is_empty() {
            return Err("Native table has no complete rows".into());
        }
        Ok(())
    }
    pub(super) fn position(&self, text: &Rope, at: usize) -> Position {
        let row = self
            .rows
            .iter()
            .position(|r| contains(&r.source, at, text.len_bytes()))
            .unwrap_or(self.rows.len() - 1);
        let cells = &self.rows[row].cells;
        let column = text
            .byte_slice(self.rows[row].body.start..at.min(self.rows[row].body.end))
            .bytes()
            .filter(|b| *b == b'|')
            .count()
            .saturating_sub(1)
            .min(cells.len().saturating_sub(1));
        let offset = cells.get(column).map_or(0, |c| {
            text.byte_slice(c.source.start..at.clamp(c.source.start, c.source.end))
                .len_chars()
        });
        let prefix = cells.first().is_some_and(|c| at < c.source.start);
        let offset = if prefix {
            text.byte_slice(self.rows[row].body.start..at).len_chars()
        } else {
            offset
        };
        Position {
            row,
            column,
            offset,
            prefix,
        }
    }
}
impl Row {
    fn new(text: &Rope, source: Range<usize>, separator: bool) -> Result<Self, String> {
        let line = text.byte_slice(source.clone()).to_string();
        let body = line.trim_end_matches(['\r', '\n']);
        let pipe = body.find('|').ok_or("Table row has no opening delimiter")?;
        if pipe > WIDTH {
            return Err("Table indentation exceeds 4096 bytes".into());
        }
        if separator && !body[pipe..].bytes().all(|c| b"-|+ \t".contains(&c)) {
            return Err("Malformed table separator; source unchanged".into());
        }
        Ok(Self {
            old_row: None,
            body: source.start..source.start + body.len(),
            indent: body[..pipe].into(),
            ending: line[body.len()..].into(),
            source,
            cells: Vec::new(),
            separator,
            rule_columns: body[pipe + 1..]
                .trim_end_matches('|')
                .split(['+', '|'])
                .count()
                .max(1),
        })
    }
    fn separator_columns(&self) -> usize {
        self.rule_columns
    }
}
fn contains(range: &Range<usize>, at: usize, end: usize) -> bool {
    range.contains(&at) || at == end && range.end == end
}
pub(super) fn admitted(nodes: &[structure::Raw], mut index: usize, text: &Rope) -> bool {
    loop {
        let node = &nodes[index];
        if blocked(node, text) {
            return false;
        }
        let Some(parent) = node.parent else {
            return true;
        };
        index = parent;
    }
}
fn blocked(node: &structure::Raw, text: &Rope) -> bool {
    use super::super::BlockKind as B;
    match node.kind {
        K::Block(B::Source | B::Example | B::Export | B::Comment | B::Dynamic) => true,
        K::Block(B::Greater) => !node.name.as_ref().is_some_and(|r| {
            matches!(
                text.byte_slice(r.clone())
                    .to_string()
                    .to_ascii_uppercase()
                    .as_str(),
                "QUOTE" | "CENTER"
            )
        }),
        _ => false,
    }
}
