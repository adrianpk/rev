use super::*;
impl Snapshot {
    pub(super) fn layout(&self, table: &Table, cancel: &Cancellation) -> Result<Layout, String> {
        let indent = &table.rows[0].indent;
        let base = measure(indent, 0, self.tab_width, cancel)?;
        let alignments = table.alignments(cancel)?;
        let widths = self.widths(table, &alignments, base, cancel)?;
        let mut layout = Layout {
            text: String::new(),
            anchors: Vec::new(),
            positions: Vec::new(),
            starts: Vec::new(),
        };
        for (index, row) in table.rows.iter().enumerate() {
            check(cancel)?;
            layout.starts.push(layout.text.len());
            let rendered = self.render(row, (&widths, &alignments), indent, cancel)?;
            let ending = row.output_ending(index + 1 < table.rows.len(), self.newline);
            layout.append(row, rendered, ending)?;
        }
        Ok(layout)
    }
    fn widths(
        &self,
        table: &Table,
        alignments: &[Alignment],
        indent: usize,
        cancel: &Cancellation,
    ) -> Result<Vec<usize>, String> {
        let mut widths = Vec::new();
        let mut column = indent + 2;
        for (index, _) in alignments.iter().enumerate() {
            let mut width = 1;
            for cell in table.rows.iter().filter_map(|r| r.cells.get(index)) {
                check(cancel)?;
                width = width.max(measure(&cell.value, column, self.tab_width, cancel)?);
            }
            if width > WIDTH {
                return Err("Table cell exceeds 4096 displayed columns".into());
            }
            widths.push(width);
            column += width + 3;
        }
        Ok(widths)
    }
    fn render(
        &self,
        row: &Row,
        columns: (&[usize], &[Alignment]),
        indent: &str,
        cancel: &Cancellation,
    ) -> Result<Rendered, String> {
        let mut rendered = Rendered {
            text: format!("{indent}|"),
            cells: Vec::new(),
        };
        let mut column = measure(indent, 0, self.tab_width, cancel)? + 2;
        for (index, width) in columns.0.iter().copied().enumerate() {
            check(cancel)?;
            if row.separator {
                rendered.rule(width, index);
                continue;
            }
            let value = row.cells.get(index).map_or("", |c| c.value.as_str());
            let (left, right) = self.padding(value, column, (width, columns.1[index]), cancel)?;
            rendered.cell(value, (left, right));
            column += width + 3;
        }
        Ok(rendered)
    }
    fn padding(
        &self,
        value: &str,
        column: usize,
        spec: (usize, Alignment),
        cancel: &Cancellation,
    ) -> Result<(usize, usize), String> {
        if value.is_empty() {
            return Ok((0, spec.0));
        }
        let width = measure(value, column, self.tab_width, cancel)?;
        let mut left = match spec.1 {
            Alignment::Left => 0,
            Alignment::Center => (spec.0 - width) / 2,
            Alignment::Right => spec.0 - width,
        };
        while left + measure(value, column + left, self.tab_width, cancel)? > spec.0 {
            left -= 1;
        }
        let right = spec.0 - left - measure(value, column + left, self.tab_width, cancel)?;
        Ok((left, right))
    }
}
struct Rendered {
    text: String,
    cells: Vec<Range<usize>>,
}
impl Rendered {
    fn cell(&mut self, value: &str, padding: (usize, usize)) {
        self.text.push(' ');
        self.text.push_str(&" ".repeat(padding.0));
        let start = self.text.len();
        self.text.push_str(value);
        self.cells.push(start..self.text.len());
        self.text.push_str(&" ".repeat(padding.1));
        self.text.push_str(" |");
    }
    fn rule(&mut self, width: usize, index: usize) {
        if index > 0 {
            self.text.pop();
            self.text.push('+');
        }
        self.text.push_str(&"-".repeat(width + 2));
        self.text.push('|');
    }
}
impl Layout {
    fn append(&mut self, row: &Row, rendered: Rendered, ending: &str) -> Result<(), String> {
        if self.text.len() + rendered.text.len() + ending.len() > Limits::default().bytes {
            return Err("Aligned table exceeds native 8 MiB output limit".into());
        }
        let start = self.text.len();
        let cells: Vec<_> = rendered
            .cells
            .into_iter()
            .map(|r| start + r.start..start + r.end)
            .collect();
        for (original, output) in row
            .cells
            .iter()
            .zip(&cells)
            .filter(|(c, _)| !c.value.is_empty())
        {
            self.anchors.push(Anchor {
                source: original.source.clone(),
                output: output.clone(),
            });
        }
        self.positions.push(if row.separator {
            // A rule has one caret destination and no data field.
            let at = start + rendered.text.find('|').expect("rendered rule delimiter");
            std::iter::once(at..at).collect()
        } else {
            cells
        });
        self.text.push_str(&rendered.text);
        self.text.push_str(ending);
        Ok(())
    }
}
fn measure(value: &str, column: usize, tab: usize, cancel: &Cancellation) -> Result<usize, String> {
    let glyphs = Glyphs::for_columns(value, column, tab).map_err(|e| e.to_string())?;
    let mut end = column;
    for (index, glyph) in glyphs.enumerate() {
        if index % 64 == 0 {
            check(cancel)?;
        }
        end = glyph.map_err(|e| e.to_string())?.columns.end;
        if end - column > WIDTH {
            return Err("Table fragment exceeds 4096 displayed columns".into());
        }
    }
    Ok(end - column)
}
impl Table {
    fn alignments(&self, cancel: &Cancellation) -> Result<Vec<Alignment>, String> {
        let mut alignments = Vec::new();
        for index in 0..self.columns {
            check(cancel)?;
            let values: Vec<_> = self
                .rows
                .iter()
                .filter_map(|r| r.cells.get(index))
                .map(|c| c.value.as_str())
                .filter(|s| !s.is_empty())
                .collect();
            let cookies: Vec<_> = values.iter().filter_map(|s| cookie(s)).collect();
            if cookies.windows(2).any(|p| p[0] != p[1]) {
                return Err("Conflicting table alignment cookies; source unchanged".into());
            }
            let numeric = values.iter().filter(|s| number(s)).count();
            let alignment =
                cookies
                    .first()
                    .copied()
                    .unwrap_or(if numeric > 0 && numeric * 2 >= values.len() {
                        Alignment::Right
                    } else {
                        Alignment::Left
                    });
            alignments.push(alignment);
        }
        Ok(alignments)
    }
}
fn cookie(value: &str) -> Option<Alignment> {
    let body = value.strip_prefix('<')?.strip_suffix('>')?;
    let (head, tail) = body.split_at_checked(1)?;
    if !tail.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    match head {
        "l" => Some(Alignment::Left),
        "c" => Some(Alignment::Center),
        "r" => Some(Alignment::Right),
        _ => None,
    }
}
fn number(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

impl Row {
    fn output_ending<'a>(&'a self, has_next: bool, newline: &'a str) -> &'a str {
        if self.ending.is_empty() && has_next {
            newline
        } else {
            &self.ending
        }
    }
}
