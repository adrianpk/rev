use super::{Item, Lists, replacement, source_text};
use crate::buffer::{Buffer, edit::Replacement};

impl Lists {
    pub(super) fn numbering(
        &self,
        buffer: &Buffer,
        items: &[usize],
        inserted: Option<(usize, bool)>,
    ) -> Result<Vec<Replacement>, String> {
        let mut counter = Counter::default();
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

    pub(super) fn marker_for(
        &self,
        buffer: &Buffer,
        index: usize,
        above: bool,
    ) -> Result<String, String> {
        let item = &self.items[index];
        let marker = source_text(buffer, item.bullet.clone())?;
        if !ordered(buffer, item)? {
            return Ok(marker);
        }
        let mut counter = self.counter_before(buffer, index)?;
        let value = if above {
            counter.advance(None)?
        } else {
            counter.advance(item.reset.as_deref())?;
            counter.advance(None)?
        };
        Ok(format!(
            "{value}{}",
            marker.chars().last().expect("list marker")
        ))
    }
    fn counter_before(&self, buffer: &Buffer, index: usize) -> Result<Counter, String> {
        let mut counter = Counter::default();
        for sibling in &self.lists[self.items[index].list].items {
            if *sibling == index {
                break;
            }
            if ordered(buffer, &self.items[*sibling])? {
                counter.advance(self.items[*sibling].reset.as_deref())?;
            }
        }
        Ok(counter)
    }
}

struct Counter {
    value: String,
    rendered: usize,
}
impl Default for Counter {
    fn default() -> Self {
        Self {
            value: "0".into(),
            rendered: 0,
        }
    }
}
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
        self.inserted(inserted == Some(true))?;
        let number = self.advance(item.reset.as_deref())?;
        let marker = source_text(buffer, item.bullet.clone())?;
        let suffix = marker.chars().last().ok_or("Missing Org list marker")?;
        let next = format!("{number}{suffix}");
        self.inserted(inserted == Some(false))?;
        if marker == next {
            return Ok(None);
        }
        replacement(buffer, item.bullet.clone(), next).map(Some)
    }
    fn inserted(&mut self, active: bool) -> Result<(), String> {
        if active {
            self.advance(None)?;
        }
        Ok(())
    }
    fn advance(&mut self, reset: Option<&str>) -> Result<String, String> {
        self.value = reset.map_or_else(|| increment(&self.value), str::to_owned);
        self.rendered = self
            .rendered
            .checked_add(self.value.len())
            .ok_or("Org list counter capacity is exhausted")?;
        if self.rendered > crate::org::Limits::default().bytes {
            return Err("Org list counters exceed the native source budget".into());
        }
        Ok(self.value.clone())
    }
}

fn increment(number: &str) -> String {
    let mut digits = number.as_bytes().to_vec();
    for digit in digits.iter_mut().rev() {
        if *digit < b'9' {
            *digit += 1;
            return String::from_utf8(digits).expect("decimal digits stay ASCII");
        }
        *digit = b'0';
    }
    digits.insert(0, b'1');
    String::from_utf8(digits).expect("decimal digits stay ASCII")
}

pub(super) fn ordered(buffer: &Buffer, item: &Item) -> Result<bool, String> {
    let marker = source_text(buffer, item.bullet.clone())?;
    Ok(marker.ends_with(['.', ')']))
}
