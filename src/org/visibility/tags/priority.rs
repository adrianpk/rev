use super::*;
use crate::config::Org;
use crate::org::priority::Priority;

pub(super) fn declaration(value: &str) -> Result<Priority, String> {
    let words: Vec<_> = value.split_whitespace().take(4).collect();
    if words.len() != 3 {
        return Err("Org PRIORITIES requires highest lowest default".into());
    }
    Priority::new(words[0], words[1], words[2])
}
struct Slot {
    range: Range<usize>,
    explicit: Option<u8>,
    task_only: bool,
}
impl Outline {
    pub(crate) fn priority_policy(&self, policy: &Org) -> Result<Priority, String> {
        match &self.tags.priorities {
            Some(value) => value.clone(),
            None => Priority::configured(&policy.priority),
        }
    }
    pub(crate) fn priority_value(&self, index: usize, policy: &Org) -> Result<Option<u8>, String> {
        let slot = self.priority_slot(index, policy)?;
        Ok(slot.explicit)
    }
    fn priority_slot(&self, index: usize, policy: &Org) -> Result<Slot, String> {
        let header = &self.tags.headers[index];
        let workflow = self.task_workflow(&policy.todo)?;
        let task = workflow.state(&header.word).is_some();
        let word = header.words.get(usize::from(task));
        if let Some((range, word)) = word.filter(|(_, word)| word.starts_with("[#")) {
            let letter = word
                .strip_prefix("[#")
                .and_then(|s| s.strip_suffix(']'))
                .ok_or("Malformed Org priority cookie")?;
            let explicit = self.priority_policy(policy)?.admit(letter)?;
            return Ok(Slot {
                range: range.clone(),
                explicit,
                task_only: false,
            });
        }
        let at = word.map_or_else(
            || header.title.start + if task { header.word.len() } else { 0 },
            |(range, _)| range.start,
        );
        Ok(Slot {
            range: at..at,
            explicit: None,
            task_only: task && word.is_none(),
        })
    }
    pub(crate) fn priority_edit(
        &self,
        buffer: &Buffer,
        index: usize,
        value: Option<u8>,
        policy: &Org,
    ) -> Result<Vec<Replacement>, String> {
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        let mut slot = self.priority_slot(index, policy)?;
        if slot.explicit == value {
            return Ok(Vec::new());
        }
        let text = priority_text(&slot, value, self.headings[index].title.end);
        if value.is_none() {
            remove_space(buffer, &mut slot.range)?;
        }
        Ok(vec![Replacement {
            source: buffer.byte_span(slot.range).map_err(|e| e.to_string())?,
            text,
        }])
    }
    pub(crate) fn heading_property_entry(
        &self,
        buffer: &Buffer,
        index: usize,
    ) -> Result<usize, String> {
        let point = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .from_byte(crate::coordinates::Byte(self.headings[index].start))
            .map_err(|e| e.to_string())?;
        self.property_target(buffer, point)
    }
}
fn priority_text(slot: &Slot, value: Option<u8>, end: usize) -> String {
    let Some(value) = value else {
        return String::new();
    };
    let prefix = if slot.task_only { " " } else { "" };
    let suffix = if slot.range.is_empty() && !slot.task_only && slot.range.start < end {
        " "
    } else {
        ""
    };
    format!("{prefix}[#{}]{suffix}", char::from(value))
}
fn remove_space(buffer: &Buffer, range: &mut Range<usize>) -> Result<(), String> {
    let end = (range.end + 1).min(buffer.len_bytes());
    let span = buffer
        .byte_span(range.end..end)
        .map_err(|e| e.to_string())?;
    if buffer.slice(&span).map_err(|e| e.to_string())? == " " {
        range.end = end;
    }
    Ok(())
}
