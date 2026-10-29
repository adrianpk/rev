use super::*;
#[derive(Default)]
pub(super) struct Notes {
    bodies: BTreeMap<String, String>,
    used: BTreeMap<String, usize>,
    bytes: usize,
    anonymous: usize,
    references: usize,
}
impl Html<'_> {
    fn define(&mut self, label: String, body: String) -> Result<(), String> {
        self.admit_note(&label)?;
        self.notes.bytes = self
            .notes
            .bytes
            .checked_add(body.len())
            .ok_or("HTML notes overflow")?;
        if self.notes.bytes > self.snapshot.export_bytes {
            return Err("HTML footnote byte limit reached".into());
        }
        if self.notes.bodies.insert(label, body).is_some() {
            return Err("Duplicate HTML footnote definition".into());
        }
        Ok(())
    }
    fn admit_note(&self, label: &str) -> Result<(), String> {
        if label.is_empty()
            || label.len() > 1024
            || self.notes.bodies.len() + self.notes.used.len() > 4096
        {
            return Err("HTML footnotes exceed native labels or count limits".into());
        }
        Ok(())
    }
    fn reference(&mut self, label: String) -> Result<(), String> {
        self.admit_note(&label)?;
        self.notes.references += 1;
        if self.notes.references > 100000 {
            return Err("HTML footnote reference limit reached".into());
        }
        let count = self.notes.used.entry(label.clone()).or_default();
        *count += 1;
        let anchor = format!("org-note-ref-{}-{count}", encoded(&label));
        self.identity(&anchor)?;
        self.emit(&format!(
            "<sup id=\"{anchor}\"><a href=\"#org-note-{}\">",
            encoded(&label)
        ))?;
        self.text(&label)?;
        self.emit("</a></sup>")
    }
    pub(super) fn footnotes(&mut self) -> Result<(), String> {
        if self
            .notes
            .used
            .keys()
            .any(|k| !self.notes.bodies.contains_key(k))
        {
            return Err("Unresolved HTML footnote; include its definition".into());
        }
        if self.notes.bodies.is_empty() {
            return Ok(());
        }
        self.emit("<section aria-label=\"Footnotes\"><ol>\n")?;
        for (label, body) in self.notes.bodies.clone() {
            let anchor = format!("org-note-{}", encoded(&label));
            self.identity(&anchor)?;
            self.emit(&format!("<li id=\"{anchor}\">{body}"))?;
            self.backlinks(&label)?;
            self.emit("</li>\n")?;
        }
        self.emit("</ol></section>\n")
    }
    fn backlinks(&mut self, label: &str) -> Result<(), String> {
        for n in 1..=self.notes.used.get(label).copied().unwrap_or(0) {
            self.emit(&format!(
                " <a href=\"#org-note-ref-{}-{n}\" aria-label=\"Back to reference\">↩</a>",
                encoded(label)
            ))?;
        }
        Ok(())
    }
}
impl Document<'_> {
    pub(super) fn definition(
        &self,
        html: &mut Html<'_>,
        n: &structure::Raw,
    ) -> Result<bool, String> {
        let nodes = crate::org::inline::objects(
            self.text,
            n.content.clone(),
            InlineContext::Standard,
            html.cancel,
        )?;
        let Some(first) = nodes.first().filter(|first| {
            first.kind == InlineKind::FootnoteReference && first.source.start == n.content.start
        }) else {
            return Ok(false);
        };
        let label = self.text.byte_slice(first.content.clone()).to_string();
        let body = self.capture(html, first.source.end..n.content.end, 0)?;
        html.define(label, body.trim().into())?;
        Ok(true)
    }
    pub(super) fn note(
        &self,
        html: &mut Html<'_>,
        n: &crate::org::inline::Raw,
        depth: usize,
    ) -> Result<(), String> {
        let mut label = self.text.byte_slice(n.content.clone()).to_string();
        if let Some(body) = &n.detail {
            if label.is_empty() {
                html.notes.anonymous += 1;
                label = format!("inline-{}", html.notes.anonymous);
            }
            let body = self.capture(html, body.clone(), depth)?;
            html.define(label.clone(), body)?;
        }
        html.reference(label)
    }
}
