use super::*;
impl Snapshot {
    pub(in crate::org::table) fn structural_plan(
        &self,
        table: &Table,
        layout: Layout,
        at: Position,
    ) -> Result<Plan, String> {
        let cell = layout
            .positions
            .get(at.row)
            .and_then(|r| r.get(at.column))
            .ok_or("Table destination has no cell")?;
        let start = if at.prefix {
            layout.starts[at.row]
        } else {
            cell.start
        };
        let end = cell.end;
        let target = start
            + layout.text[start..end]
                .char_indices()
                .nth(at.offset)
                .map_or(end - start, |(b, _)| b);
        let cursor =
            Scalar(self.text.byte_to_char(table.source.start) + layout.cursor(at.row, target)?);
        let mut anchors = layout.anchors;
        for (row, start) in table.rows.iter().zip(&layout.starts) {
            if !row.source.is_empty() {
                let indent = table.rows[0].indent.len();
                let source = row.body.start + row.indent.len();
                if self.text.byte_slice(source..source + 1) == "|" {
                    anchors.push(Anchor {
                        source: source..source + 1,
                        output: start + indent..start + indent + 1,
                    });
                }
            }
        }
        let retained = self.retained(&layout.text, table.source.start, anchors);
        let edits = self.table_replacement(table.source.clone(), layout.text)?;
        Ok(Plan {
            edits,
            retained,
            cursor,
        })
    }
    pub(super) fn retained(
        &self,
        output: &str,
        base: usize,
        mut anchors: Vec<Anchor>,
    ) -> Vec<Retained> {
        anchors.sort_by_key(|a| a.output.start);
        let mut byte = 0;
        let mut scalar = self.text.byte_to_char(base);
        let mut retained = Vec::with_capacity(anchors.len());
        for anchor in anchors {
            scalar += output[byte..anchor.output.start].chars().count();
            let end = scalar + output[anchor.output.clone()].chars().count();
            retained.push(Retained {
                source: Span {
                    stamp: self.stamp,
                    range: self.text.byte_to_char(anchor.source.start)
                        ..self.text.byte_to_char(anchor.source.end),
                },
                output: scalar..end,
            });
            scalar = end;
            byte = anchor.output.end;
        }
        retained
    }
    pub(in crate::org::table) fn table_replacement(
        &self,
        source: Range<usize>,
        value: String,
    ) -> Result<Vec<Replacement>, String> {
        let mut edits = Vec::new();
        self.gap(&mut edits, source, &value)?;
        Ok(edits)
    }
    pub(in crate::org::table) fn empty_plan(&self, table: &Table) -> Result<Plan, String> {
        Ok(Plan {
            edits: self.table_replacement(table.source.clone(), String::new())?,
            cursor: Scalar(self.text.byte_to_char(table.source.start)),
            retained: Vec::new(),
        })
    }
}
