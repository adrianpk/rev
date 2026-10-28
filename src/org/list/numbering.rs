use super::{Item, Lists, replacement, source_text};
use crate::buffer::{Buffer, edit::Replacement};

impl Lists {
    pub(super) fn numbering(
        &self,
        buffer: &Buffer,
        items: &[usize],
        inserted: Option<(usize, bool)>,
    ) -> Result<Vec<Replacement>, String> {
        let mut counter = Counter(0);
        let mut edits = Vec::new();
        for (position, index) in items.iter().copied().enumerate() {
            let insertion = inserted
                .filter(|entry| entry.0 == position)
                .map(|entry| entry.1);
            if let Some(edit) = counter.item(buffer, &self.items[index], insertion)? {
                edits.push(edit);
            }
        }
        edits.sort_by_key(|edit| edit.source.range.start);
        Ok(edits)
    }
}

struct Counter(u64);
impl Counter {
    fn item(
        &mut self,
        buffer: &Buffer,
        item: &Item,
        inserted: Option<bool>,
    ) -> Result<Option<Replacement>, String> {
        if !ordered(buffer, item)? {
            return Ok(None);
        }
        self.skip(inserted == Some(true))?;
        self.skip(true)?;
        self.0 = item.reset.unwrap_or(self.0);
        let marker = source_text(buffer, item.bullet.clone())?;
        let suffix = marker.chars().last().ok_or("Missing Org list marker")?;
        let next = format!("{}{suffix}", self.0);
        self.skip(inserted == Some(false))?;
        if marker == next {
            return Ok(None);
        }
        replacement(buffer, item.bullet.clone(), next).map(Some)
    }
    fn skip(&mut self, active: bool) -> Result<(), String> {
        if active {
            self.0 = self
                .0
                .checked_add(1)
                .ok_or("Org list counter is exhausted")?;
        }
        Ok(())
    }
}

pub(super) fn ordered(buffer: &Buffer, item: &Item) -> Result<bool, String> {
    let marker = source_text(buffer, item.bullet.clone())?;
    Ok(marker.ends_with(['.', ')']))
}
