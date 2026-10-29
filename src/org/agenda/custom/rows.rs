use super::*;
use crate::config::AgendaSort as S;
use std::{cmp::Ordering, collections::BTreeMap};
pub(super) fn calendar(result: &mut QueryResult, calendar: Calendar) {
    let metadata: BTreeMap<_, _> = result
        .rows
        .drain(..)
        .map(|r| ((r.source, r.byte), r))
        .collect();
    for event in calendar.rows {
        if let Some(row) = metadata.get(&(event.source, event.byte)) {
            let mut row = row.clone();
            row.fields.insert("DATE".into(), event.date.to_string());
            row.fields.insert(
                "TIME".into(),
                event
                    .time
                    .map(|t| format!("{:02}:{:02}", t.hour(), t.minute()))
                    .unwrap_or_default(),
            );
            row.title = format!("{} {} {}", event.date, event.kind.label(), row.title).into();
            result.rows.push(row);
        }
    }
    result.truncated |= calendar.truncated;
    result.cancelled |= calendar.cancelled;
    result.issues.extend(calendar.issues);
}
pub(super) fn append(
    output: &mut QueryResult,
    mut result: QueryResult,
    selected: (usize, &AgendaBlock),
    policy: &Org,
) -> Result<(), String> {
    let (index, block) = selected;
    let mapping = sources(output, std::mem::take(&mut result.sources))?;
    output.truncated |= result.truncated;
    output.cancelled |= result.cancelled;
    let remaining = policy.limits.agenda_rows.saturating_sub(output.rows.len());
    let limit = remaining.min(block.row_limit);
    output.truncated |= result.rows.len() > limit;
    let title = if block.title.is_empty() {
        format!("Block {} {:?}", index + 1, block.kind)
    } else {
        block.title.clone()
    };
    for mut row in result.rows.into_iter().take(limit) {
        row.source = mapping[row.source];
        row.block = Some(title.clone());
        row.block_index = index;
        output.rows.push(row);
    }
    let remaining = 4096_usize.saturating_sub(output.issues.len());
    output.truncated |= result.issues.len() > remaining;
    output
        .issues
        .extend(result.issues.into_iter().take(remaining));
    // Bound retained property text across all blocks, not independently per row.
    let bytes: usize = output
        .rows
        .iter()
        .map(|r| {
            r.fields
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>()
        })
        .sum();
    if bytes > 8388608 {
        return Err("Agenda property projection exceeds 8 MiB".into());
    }
    Ok(())
}
pub(super) fn sort(rows: &mut [QueryRow], keys: &[S], units: &crate::config::DurationUnits) {
    if keys.is_empty() {
        return;
    }
    rows.sort_by(|a, b| {
        keys.iter()
            .map(|k| compare(a, b, *k, units))
            .find(|o| *o != Ordering::Equal)
            .unwrap_or_else(|| (a.source, a.byte).cmp(&(b.source, b.byte)))
    });
}
fn compare(a: &QueryRow, b: &QueryRow, key: S, units: &crate::config::DurationUnits) -> Ordering {
    match key {
        S::TitleUp => a.title.cmp(&b.title),
        S::TitleDown => b.title.cmp(&a.title),
        S::TodoUp => a.task.cmp(&b.task),
        S::PriorityDown => field(a, "PRIORITY").cmp(field(b, "PRIORITY")),
        S::EffortUp => effort(a, units).total_cmp(&effort(b, units)),
        S::EffortDown => effort(b, units).total_cmp(&effort(a, units)),
        S::DateUp => field(a, "DATE").cmp(field(b, "DATE")),
        S::ClockDown => field(b, "CLOCKSUM")
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&field(a, "CLOCKSUM").parse::<u64>().unwrap_or(0)),
        S::SourceUp => (a.source, a.byte).cmp(&(b.source, b.byte)),
    }
}
fn field<'a>(row: &'a QueryRow, key: &str) -> &'a str {
    row.fields.get(key).map_or("", String::as_str)
}
fn effort(row: &QueryRow, units: &crate::config::DurationUnits) -> f64 {
    crate::org::time::Duration::parse(field(row, "EFFORT"), units)
        .map_or(f64::INFINITY, |v| v.minutes())
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum SourceKey {
    Buffer(crate::buffer::BufferId, u64),
    Disk(std::path::PathBuf, u64, u64),
}
fn key(source: &CalendarSource) -> SourceKey {
    match &source.version {
        SourceVersion::Buffer(stamp) => SourceKey::Buffer(stamp.buffer, stamp.revision),
        SourceVersion::Disk(observed) => SourceKey::Disk(
            source.path.clone().unwrap_or_default(),
            observed.device,
            observed.inode,
        ),
    }
}
fn sources(output: &mut QueryResult, sources: Vec<CalendarSource>) -> Result<Vec<usize>, String> {
    let mut known: BTreeMap<_, _> = output
        .sources
        .iter()
        .enumerate()
        .map(|(i, s)| (key(s), i))
        .collect();
    let mut mapping = Vec::new();
    for source in sources {
        let key = key(&source);
        let index = match known.get(&key) {
            Some(index) => {
                if output.sources[*index].version != source.version {
                    return Err("Custom agenda source observations disagree".into());
                }
                *index
            }
            None => {
                let index = output.sources.len();
                known.insert(key, index);
                output.sources.push(source);
                index
            }
        };
        mapping.push(index);
    }
    Ok(mapping)
}
