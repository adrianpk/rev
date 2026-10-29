use super::*;
impl Document<'_> {
    pub(super) fn table(&self, html: &mut Html<'_>, id: usize) -> Result<(), String> {
        html.emit("<table>\n")?;
        let mut columns = None;
        for row in &self.children[id] {
            document::check(html.cancel)?;
            if !matches!(
                self.nodes[*row].kind,
                StructureKind::TableRow { separator: false }
            ) {
                continue;
            }
            let cells = &self.children[*row];
            if columns.is_some_and(|n| n != cells.len()) {
                return Err("Ragged HTML table cannot invent cells".into());
            }
            columns = Some(cells.len());
            html.emit("<tr>")?;
            self.cells(html, cells)?;
            html.emit("</tr>\n")?;
        }
        html.emit("</table>\n")
    }
    fn cells(&self, html: &mut Html<'_>, cells: &[usize]) -> Result<(), String> {
        for cell in cells {
            html.emit("<td>")?;
            self.inline(html, self.nodes[*cell].content.clone(), 0)?;
            html.emit("</td>")?;
        }
        Ok(())
    }
    pub(super) fn block(
        &self,
        html: &mut Html<'_>,
        id: usize,
        kind: BlockKind,
        depth: usize,
    ) -> Result<(), String> {
        let n = &self.nodes[id];
        match kind {
            BlockKind::Comment => Ok(()),
            BlockKind::Greater => self.container(html, id, "blockquote", depth),
            BlockKind::Verse => {
                html.emit("<pre>")?;
                self.inline(html, n.content.clone(), 0)?;
                html.emit("</pre>\n")
            }
            BlockKind::Dynamic => self.dynamic(html, n, depth),
            _ => {
                html.emit("<pre><code>")?;
                html.text(&self.text.byte_slice(n.content.clone()).to_string())?;
                html.emit("</code></pre>\n")
            }
        }
    }
    fn dynamic(&self, html: &mut Html<'_>, n: &structure::Raw, depth: usize) -> Result<(), String> {
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
        if !matches!(name.as_str(), "clocktable" | "columnview") {
            html.diagnostic("Unknown dynamic block retained literally in HTML")?;
            html.emit("<pre>")?;
            html.text(&self.text.byte_slice(n.source.clone()).to_string())?;
            return html.emit("</pre>\n");
        }
        let text = Rope::from_str(&self.text.byte_slice(n.content.clone()).to_string());
        let nodes = document::nodes(&text, html.cancel)?;
        html.observed = html
            .observed
            .checked_add(nodes.len())
            .ok_or("HTML node count overflows")?;
        if html.observed > super::super::super::Limits::default().nodes {
            return Err("HTML report node limit reached".into());
        }
        let doc = Document::new(&text, &nodes, html.snapshot)?;
        doc.body(html, depth + 1)
    }
}
