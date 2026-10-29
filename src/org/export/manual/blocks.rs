use super::*;
impl Document<'_> {
    pub(super) fn list(
        &self,
        out: &mut Manual<'_>,
        id: usize,
        ordered: bool,
        depth: usize,
    ) -> Result<(), String> {
        let name = if ordered { "enumerate" } else { "itemize" };
        if out.texinfo() {
            self.list_open(out, id, ordered)?;
        } else {
            out.emit(".RS 2\n")?;
        }
        self.children(out, id, depth)?;
        out.emit(&if out.texinfo() {
            format!("@end {name}\n")
        } else {
            ".RE\n".into()
        })
    }
    fn list_open(&self, out: &mut Manual<'_>, id: usize, ordered: bool) -> Result<(), String> {
        if !ordered {
            return out.emit("@itemize @bullet\n");
        }
        let item = self.children[id]
            .first()
            .ok_or("Empty ordered manual list")?;
        let range = self.nodes[*item]
            .opening
            .clone()
            .ok_or("Ordered manual list has no marker")?;
        let marker = self.text.byte_slice(range).to_string();
        let number = marker
            .trim()
            .trim_end_matches(['.', ')'])
            .parse::<u32>()
            .map_err(|_| "Manual list start is not a finite integer")?;
        out.emit(&format!("@enumerate {number}\n"))
    }
    pub(super) fn item(&self, out: &mut Manual<'_>, id: usize, depth: usize) -> Result<(), String> {
        if out.texinfo() {
            out.emit("@item\n")?;
        } else {
            self.man_item(out, id)?;
        }
        self.children(out, id, depth)
    }
    fn man_item(&self, out: &mut Manual<'_>, id: usize) -> Result<(), String> {
        let marker = self.nodes[id]
            .opening
            .as_ref()
            .map(|r| self.text.byte_slice(r.clone()).to_string())
            .unwrap_or_default();
        out.emit(".IP \"")?;
        out.line(marker.trim())?;
        out.emit("\" 4\n")
    }
    pub(super) fn block(
        &self,
        out: &mut Manual<'_>,
        id: usize,
        kind: BlockKind,
        depth: usize,
    ) -> Result<(), String> {
        let n = &self.nodes[id];
        match kind {
            BlockKind::Comment => Ok(()),
            BlockKind::Greater => self.quote(out, id, depth),
            _ => {
                if kind == BlockKind::Dynamic {
                    out.diagnostic(
                        "Recorded dynamic block retained literally; no refresh performed",
                    )?;
                }
                out.literal(&self.text.byte_slice(n.content.clone()).to_string())
            }
        }
    }
    fn quote(&self, out: &mut Manual<'_>, id: usize, depth: usize) -> Result<(), String> {
        let (start, end) = if out.texinfo() {
            ("@quotation\n", "@end quotation\n")
        } else {
            (".RS 2\n", ".RE\n")
        };
        out.emit(start)?;
        self.children(out, id, depth)?;
        out.emit(end)
    }
}
