use super::*;
use crate::coordinates::Scalar;
use crate::org::visibility::tags::timestamps;
use std::ops::Range;

pub(super) fn edit(
    model: &Outline,
    buffer: &Buffer,
    change: &TaskChange,
    plan: &mut edit::Plan,
) -> Result<(), String> {
    let repeat = change.repeat.as_ref().ok_or("Missing repeat change")?;
    let planning = &model.tags.metadata.entries[change.index].planning;
    let mut edits = Vec::new();
    for value in &repeat.values {
        let range = bytes(buffer, &value.source)?;
        if planning
            .as_ref()
            .is_some_and(|p| p.start <= range.start && range.end <= p.end)
        {
            edits.push((range, value.value.format()));
        } else {
            plan.add(range, value.value.format())?;
        }
    }
    if let Some(range) = planning {
        planning_row(buffer, range.clone(), edits, plan)?;
    }
    Ok(())
}
fn planning_row(
    buffer: &Buffer,
    range: Range<usize>,
    mut edits: Vec<(Range<usize>, String)>,
    plan: &mut edit::Plan,
) -> Result<(), String> {
    let mut text = edit::source(buffer, range.clone())?;
    if text.len() > 4096 {
        return Err("Org planning line exceeds 4096 bytes".into());
    }
    obsolete_fields(buffer, (&range, &text), &mut edits)?;
    edits.sort_by_key(|(r, _)| r.start);
    if edits.windows(2).any(|pair| pair[0].0.end > pair[1].0.start) {
        return Err("Overlapping Org repeat planning facts".into());
    }
    for (source, value) in edits.into_iter().rev() {
        text.replace_range(source.start - range.start..source.end - range.start, &value);
    }
    if text.trim().is_empty() {
        text.clear();
    }
    plan.add(range, text)
}
fn obsolete_fields(
    buffer: &Buffer,
    row: (&Range<usize>, &str),
    edits: &mut Vec<(Range<usize>, String)>,
) -> Result<(), String> {
    let (range, text) = row;
    remove_field(buffer, (range, text), "CLOSED", edits)?;
    let scheduled = timestamps::in_planning(buffer, range.clone(), "SCHEDULED")?;
    let scheduled_range = bytes(buffer, &scheduled.source)?;
    if scheduled.exists() && !edits.iter().any(|(range, _)| *range == scheduled_range) {
        remove_field(buffer, (range, text), "SCHEDULED", edits)?;
    }
    Ok(())
}
fn remove_field(
    buffer: &Buffer,
    row: (&Range<usize>, &str),
    name: &str,
    edits: &mut Vec<(Range<usize>, String)>,
) -> Result<(), String> {
    let (range, text) = row;
    let target = timestamps::in_planning(buffer, range.clone(), name)?;
    if !target.exists() {
        return Ok(());
    }
    let marker = format!("{name}:");
    let start = text.find(&marker).ok_or("Missing Org planning marker")?;
    let start = text[..start].trim_end().len();
    let end = bytes(buffer, &target.source)?.end;
    edits.push((range.start + start..end, String::new()));
    Ok(())
}
pub(super) fn bytes(buffer: &Buffer, source: &Span) -> Result<Range<usize>, String> {
    let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let start = c
        .to_byte(Scalar(source.range.start))
        .map_err(|e| e.to_string())?;
    let end = c
        .to_byte(Scalar(source.range.end))
        .map_err(|e| e.to_string())?;
    Ok(start.0..end.0)
}
