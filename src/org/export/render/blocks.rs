use super::*;
impl Render<'_> {
    pub(super) fn table(&mut self, id: usize) -> Result<String, String> {
        let mut out = String::new();
        let mut row_count = 0;
        let mut columns = None;
        for row in self.children[id].clone() {
            check(self.cancel)?;
            let StructureKind::TableRow { separator } = self.nodes[row].kind else {
                continue;
            };
            let cells = self.cells(row)?;
            if separator {
                continue;
            }
            if columns.is_some_and(|c| c != cells.len()) {
                return Err("Ragged Org table cannot be rendered without inventing cells".into());
            }
            columns = Some(cells.len());
            let cells = cells
                .iter()
                .map(|c| c.replace('|', "\\|"))
                .collect::<Vec<_>>()
                .join(" | ");
            append(
                &mut out,
                &format!("| {cells} |\n"),
                self.snapshot.export_bytes,
            )?;
            self.rule(&mut out, columns.expect("row"), row_count)?;
            row_count += 1;
        }
        append(&mut out, "\n", self.snapshot.export_bytes)?;
        Ok(out)
    }
    fn rule(&self, out: &mut String, columns: usize, row: usize) -> Result<(), String> {
        if row == 0 && self.markdown() {
            append(
                out,
                &format!("| {} |\n", vec!["---"; columns].join(" | ")),
                self.snapshot.export_bytes,
            )?;
        }
        Ok(())
    }
    fn cells(&mut self, row: usize) -> Result<Vec<String>, String> {
        let mut cells = Vec::new();
        for cell in self.children[row].clone() {
            cells.push(self.inline.range(self.nodes[cell].content.clone(), 0)?);
        }
        Ok(cells)
    }
    pub(super) fn block(
        &mut self,
        id: usize,
        kind: BlockKind,
        depth: usize,
    ) -> Result<String, String> {
        let n = &self.nodes[id];
        let body = self.text.byte_slice(n.content.clone()).to_string();
        if kind == BlockKind::Greater {
            let mut changes = Vec::new();
            self.descend(id, depth, &mut changes)?;
            let text = self.compose(n.content.clone(), changes)?;
            let name = n
                .name
                .as_ref()
                .map(|r| {
                    self.text
                        .byte_slice(r.clone())
                        .to_string()
                        .to_ascii_lowercase()
                })
                .unwrap_or_default();
            if name == "quote" && self.markdown() {
                return Ok(format!(
                    "> {}\n",
                    text.trim_end_matches(['\r', '\n']).replace('\n', "\n> ")
                ));
            }
            return Ok(text);
        }
        if kind == BlockKind::Verse {
            return self.inline.range(n.content.clone(), 0);
        }
        if kind == BlockKind::Dynamic {
            return self.dynamic(id, &body);
        }
        let language = if kind == BlockKind::Source {
            language(self.text, n)
        } else {
            "text".into()
        };
        self.fenced(&body, &language)
    }
    fn dynamic(&mut self, id: usize, body: &str) -> Result<String, String> {
        let n = &self.nodes[id];
        let name = n
            .name
            .as_ref()
            .map(|r| {
                self.text
                    .byte_slice(r.clone())
                    .to_string()
                    .to_ascii_lowercase()
            })
            .unwrap_or_default();
        if matches!(name.as_str(), "clocktable" | "columnview") {
            return self.fragment(body);
        }
        self.diagnostic("Unknown dynamic block retained as literal source".into())?;
        self.fenced(&self.text.byte_slice(n.source.clone()).to_string(), "org")
    }
    fn fenced(&self, body: &str, language: &str) -> Result<String, String> {
        if !self.markdown() {
            return Ok(body.lines().map(|l| format!("    {l}\n")).collect());
        }
        let fence = inline::fence(body, 3, self.cancel)?;
        Ok(format!(
            "{fence}{language}\n{}{ending}{fence}\n",
            body,
            ending = if body.ends_with('\n') { "" } else { "\n" }
        ))
    }
}
fn language(text: &Rope, node: &structure::Raw) -> String {
    let Some(open) = &node.opening else {
        return String::new();
    };
    let line = text.byte_slice(open.clone()).to_string();
    let value = line.split_ascii_whitespace().nth(1).unwrap_or("");
    if value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_+.-".contains(&b))
    {
        value.into()
    } else {
        String::new()
    }
}
