use super::*;
impl Document<'_> {
    pub(super) fn table(&self, html: &mut Html<'_>, id: usize) -> Result<(), String> {
        let props = self.properties(html, id)?;
        props.open(html, "table")?;
        html.emit("\n")?;
        self.caption(html, props.caption.as_ref(), "caption")?;
        self.table_rows(html, id)?;
        html.emit("</table>\n")
    }
    fn table_rows(&self, html: &mut Html<'_>, id: usize) -> Result<(), String> {
        let header = self.children[id].iter().position(|row| {
            matches!(
                self.nodes[*row].kind,
                StructureKind::TableRow { separator: true }
            )
        });
        let mut columns = None;
        for (position, row) in self.children[id].iter().enumerate() {
            document::check(html.cancel)?;
            if !matches!(
                self.nodes[*row].kind,
                StructureKind::TableRow { separator: false }
            ) {
                continue;
            }
            self.table_row(
                html,
                &self.children[*row],
                &mut columns,
                header.is_some_and(|end| position < end),
            )?;
        }
        Ok(())
    }
    fn table_row(
        &self,
        html: &mut Html<'_>,
        cells: &[usize],
        columns: &mut Option<usize>,
        header: bool,
    ) -> Result<(), String> {
        if columns.is_some_and(|n| n != cells.len()) {
            return Err("Ragged HTML table cannot invent cells".into());
        }
        *columns = Some(cells.len());
        html.emit("<tr>")?;
        self.cells(html, cells, header)?;
        html.emit("</tr>\n")
    }
    fn cells(&self, html: &mut Html<'_>, cells: &[usize], header: bool) -> Result<(), String> {
        let (open, close) = if header {
            ("<th scope=\"col\">", "</th>")
        } else {
            ("<td>", "</td>")
        };
        for cell in cells {
            html.emit(open)?;
            self.inline(html, self.nodes[*cell].content.clone(), 0)?;
            html.emit(close)?;
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
        match kind {
            BlockKind::Comment => Ok(()),
            BlockKind::Greater => self.container(html, id, "blockquote", depth),
            BlockKind::Verse => self.verse(html, id),
            BlockKind::Dynamic => self.report(html, id, depth),
            _ => self.literal_block(html, id),
        }
    }
    fn verse(&self, html: &mut Html<'_>, id: usize) -> Result<(), String> {
        let props = self.properties(html, id)?;
        props.open(html, "pre")?;
        self.inline(html, self.nodes[id].content.clone(), 0)?;
        html.emit("</pre>\n")
    }
    fn report(&self, html: &mut Html<'_>, id: usize, depth: usize) -> Result<(), String> {
        let props = self.properties(html, id)?;
        props.open(html, "div")?;
        self.dynamic(html, &self.nodes[id], depth)?;
        html.emit("</div>\n")
    }
    fn literal_block(&self, html: &mut Html<'_>, id: usize) -> Result<(), String> {
        let n = &self.nodes[id];
        let props = self.properties(html, id)?;
        props.figure(html)?;
        props.open(html, "pre")?;
        html.emit("<code>")?;
        html.text(&self.text.byte_slice(n.content.clone()).to_string())?;
        html.emit("</code></pre>\n")?;
        self.caption(html, props.caption.as_ref(), "figcaption")?;
        props.end_figure(html)
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
        let doc = Document::new(&text, &nodes, html.snapshot, html.cancel, html.observed)?;
        doc.body(html, depth + 1)
    }
}
