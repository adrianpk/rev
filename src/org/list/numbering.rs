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

/// Retain the selected list's indentation and marker family for every root
/// item, without interpreting list-looking text inside literal source blocks.
pub(in crate::org) fn capture_style(
    text: &ropey::Rope,
    marker: &str,
    cancel: &crate::org::Cancellation,
) -> Result<String, String> {
    use crate::org::{Coverage, Limits, StructureKind, structure};
    let document = structure::observe(text.clone(), Limits::default(), cancel);
    if document.coverage != Coverage::Complete {
        return Err("Capture list draft is incomplete".into());
    }
    let (id, list) = document
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| matches!(n.kind, StructureKind::List { .. }))
        .ok_or("Capture draft must contain a native list item")?;
    if text
        .byte_slice(list.source.end..)
        .chars()
        .any(|c| !c.is_whitespace())
    {
        return Err("Capture list draft contains text outside its list".into());
    }
    let mut value = String::new();
    let mut offset = 0;
    for (count, node) in document
        .nodes
        .iter()
        .filter(|n| n.parent == Some(id) && matches!(n.kind, StructureKind::Item { .. }))
        .enumerate()
    {
        if cancel.is_cancelled() || count == 8192 {
            return Err("Capture list cancelled or exceeds 8192 items".into());
        }
        let item = super::model::read(text, node, 0, 0);
        value.push_str(&text.byte_slice(offset..item.bullet.start).to_string());
        value.push_str(marker.trim_start());
        offset = item.bullet.end;
    }
    value.push_str(&text.byte_slice(offset..).to_string());
    let indent = &marker[..marker.len() - marker.trim_start().len()];
    Ok(value
        .split_inclusive('\n')
        .map(|line| format!("{indent}{line}"))
        .collect())
}

type MarkerEdits = Vec<(std::ops::Range<usize>, String)>;
/// Existing source markers change only when prepending requires renumbering.
pub(in crate::org) fn capture_markers(
    old: &ropey::Rope,
    new: &ropey::Rope,
    prepend: bool,
    cancel: &crate::org::Cancellation,
) -> Result<(String, MarkerEdits), String> {
    let mut counter = Counter::default();
    let (old_edits, new_edits) = if prepend {
        let new_edits = capture_count(new, &mut counter, cancel)?;
        (capture_count(old, &mut counter, cancel)?, new_edits)
    } else {
        capture_count(old, &mut counter, cancel)?;
        (Vec::new(), capture_count(new, &mut counter, cancel)?)
    };
    let mut text = new.to_string();
    for (range, marker) in new_edits.iter().rev() {
        text.replace_range(range.clone(), marker)
    }
    Ok((text, old_edits))
}
fn capture_count(
    text: &ropey::Rope,
    counter: &mut Counter,
    cancel: &crate::org::Cancellation,
) -> Result<MarkerEdits, String> {
    use crate::org::{Coverage, Limits, StructureKind, structure};
    let document = structure::observe(text.clone(), Limits::default(), cancel);
    if document.coverage != Coverage::Complete {
        return Err("Capture list observation is incomplete".into());
    }
    let (id, _) = document
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| matches!(n.kind, StructureKind::List { .. }))
        .ok_or("Capture list is unavailable")?;
    let mut edits = Vec::new();
    for (count, node) in document
        .nodes
        .iter()
        .filter(|n| n.parent == Some(id) && matches!(n.kind, StructureKind::Item { .. }))
        .enumerate()
    {
        if cancel.is_cancelled() || count == 8192 {
            return Err("Capture list cancelled or exceeds 8192 items".into());
        }
        let item = super::model::read(text, node, 0, 0);
        let marker = text.byte_slice(item.bullet.clone()).to_string();
        if marker.ends_with(['.', ')']) {
            let number = counter.advance(item.reset.as_deref())?;
            let next = format!("{number}{}", marker.chars().last().expect("native marker"));
            if marker != next {
                edits.push((item.bullet, next))
            }
        }
    }
    Ok(edits)
}
