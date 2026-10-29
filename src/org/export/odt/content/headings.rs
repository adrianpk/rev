use super::*;
impl Document<'_> {
    pub(super) fn index(
        &mut self,
        snapshot: &Snapshot,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let options = Options {
            todo: false,
            tags: false,
            ..snapshot.options.clone()
        };
        for (id, n) in self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, StructureKind::Heading { .. }))
        {
            document::check(cancel)?;
            let target = format!("org-heading-{}-{id}", self.prefix);
            let title = super::super::super::render::headings::title_range(
                self.text,
                n,
                &options,
                &snapshot.todo,
            );
            for range in [n.content.clone(), title] {
                self.heading_links
                    .entry(self.text.byte_slice(range).to_string())
                    .and_modify(|v| *v = None)
                    .or_insert(Some(target.clone()));
            }
            let range = super::super::super::render::headings::title_range(
                self.text,
                n,
                &snapshot.options,
                &snapshot.todo,
            );
            self.headings.insert(id, (target, range));
        }
        Ok(())
    }
    pub(super) fn header(&self, odt: &mut Odt<'_>) -> Result<(), String> {
        if matches!(odt.snapshot.request.scope, Scope::Region(_)) {
            return Ok(());
        }
        let m = &odt.snapshot.metadata;
        for (value, style, enabled) in [
            (&m.title, "Title", odt.snapshot.options.title),
            (&m.author, "Caption", odt.snapshot.options.author),
            (&m.date, "Caption", true),
        ] {
            if let Some(value) = value.as_ref().filter(|_| enabled) {
                odt.xml
                    .emit(&format!("<text:p text:style-name=\"{style}\">"))?;
                odt.xml.prose(value)?;
                odt.xml.emit("</text:p>")?;
            }
        }
        self.contents(odt)
    }
    fn contents(&self, odt: &mut Odt<'_>) -> Result<(), String> {
        if !odt.snapshot.options.toc || self.headings.is_empty() {
            return Ok(());
        }
        odt.xml.emit("<text:p>Contents</text:p>")?;
        for (target, range) in self.headings.values() {
            odt.references.insert(target.clone());
            odt.xml.emit(&format!(
                "<text:p><text:a xlink:type=\"simple\" xlink:href=\"#{target}\">"
            ))?;
            odt.xml
                .prose(&self.text.byte_slice(range.clone()).to_string())?;
            odt.xml.emit("</text:a></text:p>")?;
        }
        Ok(())
    }
    pub(super) fn heading(
        &self,
        odt: &mut Odt<'_>,
        heading: (usize, usize),
        depth: usize,
    ) -> Result<(), String> {
        let (id, level) = heading;
        let (target, title) = &self.headings[&id];
        let level = level.saturating_sub(self.base).max(1);
        let unnumbered = !odt.snapshot.options.numbered;
        odt.xml.emit(&format!("<text:h text:style-name=\"Heading\" text:outline-level=\"{level}\" text:is-list-header=\"{unnumbered}\">"))?;
        odt.bookmark(target)?;
        self.inline(odt, title.clone(), 0, InlineContext::Standard)?;
        odt.xml.emit("</text:h>")?;
        self.children(odt, id, depth)
    }
}
