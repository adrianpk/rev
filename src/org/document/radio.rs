use super::*;
impl Document {
    pub(super) fn radio_unique(&self, name: &str) -> Result<usize, String> {
        let mut matches = self.objects.iter().filter(|o| {
            o.kind == InlineKind::RadioTarget && folded(&self.slice(o.content.clone()), name)
        });
        let first = matches.next().ok_or("Unresolved radio target")?;
        if matches.next().is_some() {
            return Err("Ambiguous radio target; source unchanged".into());
        }
        Ok(first.source.start)
    }
    pub(super) fn radio_at(
        &self,
        at: usize,
        cancel: &Cancellation,
    ) -> Result<Option<usize>, String> {
        let mut found = None;
        let mut records = 0;
        for target in self
            .objects
            .iter()
            .filter(|o| o.kind == InlineKind::RadioTarget)
        {
            check(cancel)?;
            records += 1;
            if records > 4096 || target.content.len() > 1024 {
                return Err("Radio target profile exceeds 4096 records or 1024-byte names".into());
            }
            let name = self.slice(target.content.clone());
            if self.radio_match(at, &name, cancel)? {
                let byte = self.radio_unique(&name)?;
                if found.replace(byte).is_some_and(|old| old != byte) {
                    return Err("Ambiguous overlapping radio links".into());
                }
            }
        }
        Ok(found)
    }
    fn radio_match(&self, at: usize, name: &str, cancel: &Cancellation) -> Result<bool, String> {
        let point = self.text.byte_to_char(at);
        let width = name.chars().count();
        for start in point.saturating_sub(width)..=point {
            check(cancel)?;
            let end = start + width;
            if end > self.text.len_chars() || point >= end {
                continue;
            }
            let range = self.text.char_to_byte(start)..self.text.char_to_byte(end);
            if self.standard(&range).is_ok()
                && self.word_edges(start..end)
                && folded(&self.slice(range), name)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn word_edges(&self, range: Range<usize>) -> bool {
        let before = range.start.checked_sub(1).map(|p| self.text.char(p));
        let after = self.text.get_char(range.end);
        !before.is_some_and(word) && !after.is_some_and(word)
    }
}
fn folded(a: &str, b: &str) -> bool {
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}
fn word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
