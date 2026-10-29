use super::*;
impl Document<'_> {
    pub(super) fn list(
        &self,
        odt: &mut Odt<'_>,
        list: (usize, bool),
        depth: usize,
    ) -> Result<(), String> {
        let (id, ordered) = list;
        let style = if ordered { "NumberList" } else { "BulletList" };
        odt.xml
            .emit(&format!("<text:list text:style-name=\"{style}\">"))?;
        odt.lists += 1;
        self.children(odt, id, depth)?;
        odt.lists -= 1;
        odt.xml.emit("</text:list>")
    }
    pub(super) fn item(&self, odt: &mut Odt<'_>, id: usize, depth: usize) -> Result<(), String> {
        let n = &self.nodes[id];
        let ordered = n
            .parent
            .is_some_and(|p| matches!(self.nodes[p].kind, StructureKind::List { ordered: true }));
        let start = if ordered {
            self.start(n)?
        } else {
            String::new()
        };
        odt.xml.emit(&format!("<text:list-item{start}>"))?;
        self.children(odt, id, depth)?;
        odt.xml.emit("</text:list-item>")
    }
    fn start(&self, n: &structure::Raw) -> Result<String, String> {
        let range = n.opening.as_ref().ok_or("ODT list item has no marker")?;
        let value = self.text.byte_slice(range.clone()).to_string();
        let value = value
            .trim()
            .trim_end_matches(['.', ')'])
            .parse::<u32>()
            .map_err(|_| "ODT list marker is not a finite integer")?;
        Ok(format!(" text:start-value=\"{value}\""))
    }
    pub(super) fn table(&self, odt: &mut Odt<'_>, id: usize) -> Result<(), String> {
        if odt.lists > 0 {
            return odt.unsupported(
                &self
                    .text
                    .byte_slice(self.nodes[id].source.clone())
                    .to_string(),
            );
        }
        let rows: Vec<_> = self.children[id]
            .iter()
            .copied()
            .filter(|r| {
                matches!(
                    self.nodes[*r].kind,
                    StructureKind::TableRow { separator: false }
                )
            })
            .collect();
        let columns = rows.first().map_or(0, |r| self.children[*r].len());
        if rows.iter().any(|r| self.children[*r].len() != columns) {
            return Err("Ragged ODT table cannot invent cells".into());
        }
        if columns == 0 {
            return odt.unsupported(
                &self
                    .text
                    .byte_slice(self.nodes[id].source.clone())
                    .to_string(),
            );
        }
        odt.tables += 1;
        odt.xml.emit(&format!("<table:table table:name=\"Table{}\"><table:table-column table:number-columns-repeated=\"{columns}\"/>", odt.tables))?;
        for row in rows {
            self.row(odt, row)?;
        }
        odt.xml.emit("</table:table>")
    }
    fn row(&self, odt: &mut Odt<'_>, row: usize) -> Result<(), String> {
        odt.xml.emit("<table:table-row>")?;
        for cell in &self.children[row] {
            odt.xml
                .emit("<table:table-cell office:value-type=\"string\"><text:p>")?;
            self.inline(
                odt,
                self.nodes[*cell].content.clone(),
                0,
                InlineContext::Standard,
            )?;
            odt.xml.emit("</text:p></table:table-cell>")?;
        }
        odt.xml.emit("</table:table-row>")
    }
    pub(super) fn block(
        &self,
        odt: &mut Odt<'_>,
        block: (usize, BlockKind),
        depth: usize,
    ) -> Result<(), String> {
        let (id, kind) = block;
        let n = &self.nodes[id];
        match kind {
            BlockKind::Comment => Ok(()),
            BlockKind::Greater => self.children(odt, id, depth),
            BlockKind::Dynamic => self.dynamic(odt, n, depth),
            BlockKind::Source | BlockKind::Example | BlockKind::Verse => {
                odt.literal(&self.text.byte_slice(n.content.clone()).to_string())
            }
            _ => odt.unsupported(&self.text.byte_slice(n.source.clone()).to_string()),
        }
    }
    fn dynamic(&self, odt: &mut Odt<'_>, n: &structure::Raw, depth: usize) -> Result<(), String> {
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
        if !["columnview", "clocktable"].contains(&name.as_str()) {
            return odt.unsupported(&self.text.byte_slice(n.source.clone()).to_string());
        }
        let text = Rope::from_str(&self.text.byte_slice(n.content.clone()).to_string());
        let nodes = document::nodes(&text, odt.cancel)?;
        odt.observed = odt
            .observed
            .checked_add(nodes.len())
            .ok_or("ODT node count overflows")?;
        if odt.observed > super::super::super::super::Limits::default().nodes {
            return Err("ODT report node limit reached".into());
        }
        Document::new(&text, &nodes, odt.snapshot, odt.cancel, odt.observed)?.body(odt, depth + 1)
    }
}
