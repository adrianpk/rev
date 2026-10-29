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
                    return Err("Duplicate ODT footnote definition".into());
                }
            }
        }
        Ok(())
    }
    pub(super) fn note(
        &self,
        odt: &mut Odt<'_>,
        n: &crate::org::inline::Raw,
        depth: usize,
    ) -> Result<(), String> {
        let label = self.text.byte_slice(n.content.clone()).to_string();
        if let Some(body) = &n.detail {
            return self.inline_note(odt, body.clone(), depth);
        }
        admit(&label, odt.used.len())?;
        let key = (self.prefix, label.clone());
        if let Some(number) = odt.used.get(&key) {
            return odt.xml.emit(&format!("<text:note-ref text:ref-name=\"note{number}\" text:note-class=\"footnote\" text:reference-format=\"text\">{number}</text:note-ref>"));
        }
        let body = self
            .notes
            .get(&label)
            .ok_or("Unresolved ODT footnote; include its definition")?;
        if !odt.active.insert(key.clone()) {
            return Err("Cyclic ODT footnote".into());
        }
        let number = odt.note_count + 1;
        self.inline_note(odt, body.clone(), depth)?;
        odt.used.insert(key.clone(), number);
        odt.active.remove(&key);
        Ok(())
    }
    fn inline_note(
        &self,
        odt: &mut Odt<'_>,
        body: Range<usize>,
        depth: usize,
    ) -> Result<(), String> {
        odt.note_count += 1;
        if odt.note_count > 4096 {
            return Err("ODT footnote count limit reached".into());
        }
        let number = odt.note_count;
        odt.xml.emit(&format!("<text:note text:id=\"note{number}\" text:note-class=\"footnote\"><text:note-citation>{number}</text:note-citation><text:note-body><text:p>"))?;
        self.inline(odt, body, depth, InlineContext::Standard)?;
        odt.xml.emit("</text:p></text:note-body></text:note>")
    }
}
fn admit(label: &str, count: usize) -> Result<(), String> {
    if label.is_empty() || label.len() > 1024 || count >= 4096 {
        return Err("ODT footnotes exceed native label or count limits".into());
    }
    Ok(())
}
