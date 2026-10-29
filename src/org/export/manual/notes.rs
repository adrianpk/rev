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
    pub(super) fn index_note(
        &mut self,
        n: &structure::Raw,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if let Some((label, body)) = self.definition(n, cancel)? {
            admit(&label, self.notes.len())?;
            if self.notes.insert(label, body).is_some() {
                return Err("Duplicate manual footnote definition".into());
            }
        }
        Ok(())
    }
    pub(super) fn note(
        &self,
        out: &mut Manual<'_>,
        n: &crate::org::inline::Raw,
        depth: usize,
    ) -> Result<(), String> {
        if let Some(body) = &n.detail {
            return self.inline_note(out, body.clone(), depth);
        }
        let label = self.text.byte_slice(n.content.clone()).to_string();
        admit(&label, out.used.len())?;
        if out.active.contains(&label) {
            return Err("Cyclic manual footnote".into());
        }
        if let Some(number) = out.used.get(&label) {
            return out.note_reference(*number);
        }
        self.named_note(out, &label, depth)
    }
    fn named_note(&self, out: &mut Manual<'_>, label: &str, depth: usize) -> Result<(), String> {
        let body = self
            .notes
            .get(label)
            .ok_or("Unresolved manual footnote; include its definition")?;
        out.active.insert(label.into());
        out.used.insert(label.into(), out.notes.len() + 1);
        self.inline_note(out, body.clone(), depth)?;
        out.active.remove(label);
        Ok(())
    }
    fn inline_note(
        &self,
        out: &mut Manual<'_>,
        body: Range<usize>,
        depth: usize,
    ) -> Result<(), String> {
        if out.notes.len() >= 4096 {
            return Err("Manual footnote count limit reached".into());
        }
        let index = out.notes.len();
        out.notes.push(String::new());
        let start = out.output.len();
        self.inline(out, body, depth, InlineContext::Standard)?;
        let body = out.output.split_off(start);
        out.retained += body.len();
        out.notes[index] = body;
        out.note_reference(index + 1)
    }
}
impl Manual<'_> {
    fn note_reference(&mut self, number: usize) -> Result<(), String> {
        self.emit(&if self.texinfo() {
            format!("@ref{{org-note-{number},,[{number}]}}")
        } else {
            format!("[{number}]")
        })
    }
    pub(super) fn endnotes(&mut self) -> Result<(), String> {
        if self.notes.is_empty() {
            return Ok(());
        }
        self.emit(if self.texinfo() {
            "@node org-notes\n@unnumbered Notes\n\n"
        } else {
            ".SH \"Notes\"\n"
        })?;
        for (i, body) in std::mem::take(&mut self.notes).into_iter().enumerate() {
            let number = i + 1;
            self.retained -= body.len();
            self.emit(&if self.texinfo() {
                format!("@anchor{{org-note-{number}}}\n[{number}] ")
            } else {
                format!(".IP \"[{number}]\" 4\n")
            })?;
            self.emit(&body)?;
            self.emit("\n\n")?;
        }
        Ok(())
    }
}
fn admit(label: &str, count: usize) -> Result<(), String> {
    if label.is_empty() || label.len() > 1024 || count >= 4096 {
        return Err("Manual footnote label or count limit reached".into());
    }
    Ok(())
}
