use super::*;
impl Document<'_> {
    pub(super) fn definition(
        &self,
        n: &structure::Raw,
        cancel: &Cancellation,
    ) -> Result<Option<(String, Range<usize>)>, String> {
        if self
            .text
            .line_to_byte(self.text.byte_to_line(n.content.start))
            != n.content.start
        {
            return Ok(None);
        }
        let nodes = crate::org::inline::objects(
            self.text,
            n.content.clone(),
            InlineContext::Standard,
            cancel,
        )?;
        let Some(first) = nodes.first().filter(|first| {
            first.kind == InlineKind::FootnoteReference && first.source.start == n.content.start
        }) else {
            return Ok(None);
        };
        Ok(Some((
            self.text.byte_slice(first.content.clone()).to_string(),
            first.source.end..n.content.end,
        )))
    }
    pub(super) fn definitions(&mut self, cancel: &Cancellation) -> Result<(), String> {
        for n in
            document::active(self.nodes, cancel)?.filter(|n| n.kind == StructureKind::Paragraph)
        {
            if let Some((label, body)) = self.definition(n, cancel)? {
                admit(&label, self.notes.len())?;
                if self.notes.insert(label, body).is_some() {
                    return Err("Duplicate TeX footnote definition".into());
                }
            }
        }
        Ok(())
    }
    pub(super) fn note(
        &self,
        tex: &mut Tex<'_>,
        n: &crate::org::inline::Raw,
        depth: usize,
    ) -> Result<(), String> {
        let label = self.text.byte_slice(n.content.clone()).to_string();
        if let Some(body) = &n.detail {
            return self.inline_note(tex, body.clone(), depth);
        }
        admit(&label, tex.used.len())?;
        if let Some(number) = tex.used.get(&label) {
            return tex.emit(&format!("\\footnotemark[{number}]"));
        }
        let body = self
            .notes
            .get(&label)
            .ok_or("Unresolved TeX footnote; include its definition")?;
        if !tex.active.insert(label.clone()) {
            return Err("Cyclic TeX footnote".into());
        }
        let number = tex.note_count + 1;
        self.inline_note(tex, body.clone(), depth)?;
        tex.used.insert(label.clone(), number);
        tex.active.remove(&label);
        Ok(())
    }
    fn inline_note(
        &self,
        tex: &mut Tex<'_>,
        body: Range<usize>,
        depth: usize,
    ) -> Result<(), String> {
        tex.note_count += 1;
        if tex.note_count > 4096 {
            return Err("TeX footnote count limit reached".into());
        }
        tex.emit("\\footnote{")?;
        self.inline(tex, body, depth)?;
        tex.emit("}")
    }
}
fn admit(label: &str, count: usize) -> Result<(), String> {
    if label.is_empty() || label.len() > 1024 || count >= 4096 {
        return Err("TeX footnotes exceed native label or count limits".into());
    }
    Ok(())
}
