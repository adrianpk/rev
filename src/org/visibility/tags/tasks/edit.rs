use super::*;
use crate::buffer::edit::Replacement;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Plan {
    edits: BTreeMap<usize, (usize, String)>,
}
impl Plan {
    pub(super) fn add(&mut self, range: Range<usize>, text: String) -> Result<(), String> {
        if let Some((end, value)) = self.edits.get_mut(&range.start) {
            if *end != range.end || !range.is_empty() {
                return Err("Overlapping Org task metadata".into());
            }
            value.push_str(&text);
        } else {
            self.edits.insert(range.start, (range.end, text));
        }
        Ok(())
    }
    pub(super) fn import(&mut self, buffer: &Buffer, value: Replacement) -> Result<(), String> {
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let start = c
            .to_byte(crate::coordinates::Scalar(value.source.range.start))
            .map_err(|e| e.to_string())?
            .0;
        let end = c
            .to_byte(crate::coordinates::Scalar(value.source.range.end))
            .map_err(|e| e.to_string())?
            .0;
        self.add(start..end, value.text)
    }
    fn finish(self, buffer: &Buffer) -> Result<Vec<Replacement>, String> {
        self.edits
            .into_iter()
            .map(|(start, (end, text))| {
                Ok(Replacement {
                    source: buffer.byte_span(start..end).map_err(|e| e.to_string())?,
                    text,
                })
            })
            .collect()
    }
}
impl Outline {
    pub(crate) fn task_edits(
        &self,
        buffer: &Buffer,
        change: &TaskChange,
        stamp: &str,
        note: &str,
    ) -> Result<Vec<Replacement>, String> {
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        if change.old == change.next {
            return Ok(Vec::new());
        }
        let mut plan = Plan::default();
        self.task_keyword_edit(buffer, change, &mut plan)?;
        if change.repeat.is_none() {
            self.task_closed_edit(buffer, change, stamp, &mut plan)?;
        }
        self.repeat_edits(buffer, change, stamp, &mut plan)?;
        self.task_log_edit(buffer, change, (stamp, note), &mut plan)?;
        let mut edits = plan.finish(buffer)?;
        edits.extend(self.lists.task_statistics(
            buffer,
            change.index,
            change.result_state(),
            &change.workflow,
        )?);
        edits.sort_by_key(|e| e.source.range.start);
        Ok(edits)
    }
    fn task_keyword_edit(
        &self,
        buffer: &Buffer,
        change: &TaskChange,
        plan: &mut Plan,
    ) -> Result<(), String> {
        let start = self.tags.headers[change.index].title.start;
        let mut end = start + change.old.len();
        let text = match (change.old.is_empty(), change.result_state().is_empty()) {
            (true, false) => format!("{} ", change.result_state()),
            (false, true) => {
                if source(buffer, end..(end + 1).min(buffer.len_bytes()))? == " " {
                    end += 1;
                }
                String::new()
            }
            _ => change.result_state().to_owned(),
        };
        plan.add(start..end, text)
    }
    fn task_closed_edit(
        &self,
        buffer: &Buffer,
        change: &TaskChange,
        stamp: &str,
        plan: &mut Plan,
    ) -> Result<(), String> {
        let entry = &self.tags.metadata.entries[change.index];
        if !change.logging.closed && change.workflow.state(&change.next).is_some_and(|s| s.done) {
            return Ok(());
        }
        if let Some(range) = &entry.planning {
            let text = source(buffer, range.clone())?;
            return planning(
                plan,
                range.clone(),
                &text,
                change.logging.closed.then_some(stamp),
            );
        }
        if change.logging.closed {
            let at = self.headings[change.index].body;
            let prefix = prefix(buffer, at)?;
            plan.add(
                at..at,
                format!("{prefix}CLOSED: {stamp}{}", newline(buffer)),
            )?;
        }
        Ok(())
    }
    fn task_log_edit(
        &self,
        buffer: &Buffer,
        change: &TaskChange,
        data: (&str, &str),
        plan: &mut Plan,
    ) -> Result<(), String> {
        if change.logging.state == Log::None && !change.logging.note {
            return Ok(());
        }
        let entry = &self.tags.metadata.entries[change.index];
        let text = record(change, data, newline(buffer));
        let drawer = &change.logging.drawer;
        if drawer.is_empty() {
            let at = entry.anchor;
            return plan.add(at..at, format!("{}{text}", prefix(buffer, at)?));
        }
        let matches: Vec<_> = entry
            .drawers
            .iter()
            .filter(|d| {
                source(buffer, d.name.clone()).is_ok_and(|s| s.eq_ignore_ascii_case(drawer))
            })
            .collect();
        if matches.len() > 1 {
            return Err("Ambiguous Org task log drawer".into());
        }
        if let Some(found) = matches.first() {
            return plan.add(found.body.start..found.body.start, text);
        }
        let at = entry.anchor;
        let nl = newline(buffer);
        plan.add(
            at..at,
            format!("{}:{drawer}:{nl}{text}:END:{nl}", prefix(buffer, at)?),
        )
    }
}
fn planning(
    plan: &mut Plan,
    range: Range<usize>,
    text: &str,
    closed: Option<&str>,
) -> Result<(), String> {
    if let Some(start) = text.find("CLOSED:") {
        let tail = &text[start + 7..];
        let spaces = tail.len() - tail.trim_start().len();
        let tail = tail.trim_start();
        if !tail.starts_with('[') {
            return Err("Malformed Org CLOSED timestamp".into());
        }
        let end = start + 7 + spaces + tail.find(']').ok_or("Malformed Org CLOSED timestamp")? + 1;
        if text[end..].contains("CLOSED:") {
            return Err("Duplicate Org CLOSED timestamps".into());
        }
        if closed.is_none() && text[..start].trim().is_empty() && text[end..].trim().is_empty() {
            return plan.add(range, String::new());
        }
        return plan.add(
            range.start + start..range.start + end,
            closed.map_or_else(String::new, |s| format!("CLOSED: {s}")),
        );
    }
    if let Some(stamp) = closed {
        plan.add(range.start..range.start, format!("CLOSED: {stamp} "))?;
    }
    Ok(())
}
fn record(change: &TaskChange, data: (&str, &str), nl: &str) -> String {
    let (stamp, note) = data;
    let head = if change.logging.state != Log::None {
        format!(
            "- State \"{}\" from \"{}\" {stamp}",
            change.next, change.old
        )
    } else {
        format!("- CLOSING NOTE {stamp}")
    };
    if note.is_empty() {
        format!("{head}{nl}")
    } else {
        format!("{head} \\\\{nl}  {note}{nl}")
    }
}
pub(super) fn source(buffer: &Buffer, range: Range<usize>) -> Result<String, String> {
    let span = buffer.byte_span(range).map_err(|e| e.to_string())?;
    buffer
        .slice(&span)
        .map(|s| s.to_string())
        .map_err(|e| e.to_string())
}
fn prefix(buffer: &Buffer, at: usize) -> Result<&'static str, String> {
    if at == 0 {
        return Ok("");
    }
    let position = buffer
        .byte_span(at..at)
        .map_err(|e| e.to_string())?
        .range
        .start;
    let span = buffer
        .span(position - 1..position)
        .map_err(|e| e.to_string())?;
    let previous = buffer.slice(&span).map_err(|e| e.to_string())?;
    Ok(if previous == "\n" {
        ""
    } else {
        newline(buffer)
    })
}
fn newline(buffer: &Buffer) -> &'static str {
    buffer.newline()
}
