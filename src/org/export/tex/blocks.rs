use super::*;
impl Document<'_> {
    pub(super) fn list(
        &self,
        tex: &mut Tex<'_>,
        id: usize,
        ordered: bool,
        depth: usize,
    ) -> Result<(), String> {
        let name = if ordered { "enumerate" } else { "itemize" };
        tex.emit(&format!("\\begin{{{name}}}\n"))?;
        if ordered {
            tex.ordered += 1;
            self.list_start(tex, id)?;
        }
        self.children(tex, id, depth)?;
        if ordered {
            tex.ordered -= 1;
        }
        tex.emit(&format!("\\end{{{name}}}\n"))
    }
    fn list_start(&self, tex: &mut Tex<'_>, id: usize) -> Result<(), String> {
        let counter = ["enumi", "enumii", "enumiii", "enumiv"]
            .get(tex.ordered.saturating_sub(1))
            .ok_or("TeX admits four nested ordered lists")?;
        let marker = self.children[id]
            .iter()
            .find_map(|i| self.nodes[*i].opening.as_ref())
            .ok_or("Ordered list has no marker")?;
        let value = self.text.byte_slice(marker.clone()).to_string();
        let start = value
            .trim()
            .trim_end_matches(['.', ')'])
            .parse::<u32>()
            .map_err(|_| "TeX list start is not a finite integer")?;
        if start > 2147483647 {
            return Err("TeX list start exceeds integer range".into());
        }
        tex.emit(&format!(
            "\\setcounter{{{counter}}}{{{}}}\n",
            i64::from(start) - 1
        ))
    }
    pub(super) fn table(&self, tex: &mut Tex<'_>, id: usize) -> Result<(), String> {
        let mut columns = None;
        for row in &self.children[id] {
            if !matches!(
                self.nodes[*row].kind,
                StructureKind::TableRow { separator: false }
            ) {
                continue;
            }
            let count = self.children[*row].len();
            if columns.is_some_and(|old| old != count) {
                return Err("Ragged TeX table cannot invent cells".into());
            }
            columns = Some(count);
        }
        let columns = columns.unwrap_or(0);
        if columns == 0 {
            return Ok(());
        }
        tex.emit(&format!("\\begin{{tabular}}{{{}}}\n", "l".repeat(columns)))?;
        for row in &self.children[id] {
            self.table_row(tex, *row)?;
        }
        tex.emit("\\end{tabular}\n")
    }
    fn table_row(&self, tex: &mut Tex<'_>, row: usize) -> Result<(), String> {
        match self.nodes[row].kind {
            StructureKind::TableRow { separator: true } => tex.emit("\\hline\n"),
            StructureKind::TableRow { separator: false } => {
                for (i, cell) in self.children[row].iter().enumerate() {
                    if i > 0 {
                        tex.emit(" & ")?;
                    }
                    self.inline(tex, self.nodes[*cell].content.clone(), 0)?;
                }
                tex.emit(" \\\\\n")
            }
            _ => Ok(()),
        }
    }
    pub(super) fn block(
        &self,
        tex: &mut Tex<'_>,
        id: usize,
        kind: BlockKind,
        depth: usize,
    ) -> Result<(), String> {
        let n = &self.nodes[id];
        match kind {
            BlockKind::Comment => Ok(()),
            BlockKind::Greater => {
                tex.emit("\\begin{quote}\n")?;
                self.children(tex, id, depth)?;
                tex.emit("\\end{quote}\n")
            }
            BlockKind::Verse => {
                tex.emit("\\begin{verse}\n")?;
                self.inline(tex, n.content.clone(), 0)?;
                tex.emit("\\end{verse}\n")
            }
            BlockKind::Dynamic => self.dynamic(tex, n, depth),
            _ => tex.literal(&self.text.byte_slice(n.content.clone()).to_string()),
        }
    }
    fn dynamic(&self, tex: &mut Tex<'_>, n: &structure::Raw, depth: usize) -> Result<(), String> {
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
            tex.diagnostic("Unknown dynamic block retained as escaped TeX data")?;
            return tex.literal(&self.text.byte_slice(n.source.clone()).to_string());
        }
        let text = Rope::from_str(&self.text.byte_slice(n.content.clone()).to_string());
        let nodes = document::nodes(&text, tex.cancel)?;
        tex.observed = tex
            .observed
            .checked_add(nodes.len())
            .ok_or("TeX node count overflows")?;
        if tex.observed > super::super::super::Limits::default().nodes {
            return Err("TeX report node limit reached".into());
        }
        Document::new(&text, &nodes, tex.snapshot, tex.cancel)?.body(tex, depth + 1)
    }
}
