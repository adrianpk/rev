use super::*;
impl Context {
    pub(crate) fn body(
        &mut self,
        buffer: &Buffer,
        model: &Outline,
        time: String,
    ) -> Result<Rope, String> {
        self.subtree.current(buffer)?;
        self.values.insert("ARCHIVE_TIME".into(), time);
        if self
            .values
            .values()
            .any(|s| s.len() > 4096 || s.chars().any(char::is_control))
        {
            return Err("Archive origin metadata exceeds its one-line 4096-byte profile".into());
        }
        let edits = model.archive_properties(buffer, self.entry, &self.values)?;
        let mut body = self.subtree.text().clone();
        let start = self.subtree.source().range.start;
        let end = self.subtree.source().range.end;
        for edit in edits.iter().rev() {
            if edit.source.range.start < start || edit.source.range.end > end {
                return Err("Archive metadata is outside the source subtree".into());
            }
            let range = edit.source.range.start - start..edit.source.range.end - start;
            body.remove(range.clone());
            body.insert(range.start, &edit.text);
        }
        Ok(body)
    }
}
