use super::*;
use crate::org::{
    Limits, StructureKind, structure,
    time::{CivilDateTime, CivilInterval, CivilTime, RecordedClock},
};
pub(super) fn observe(
    result: &mut QueryResult,
    sources: &SourceSet,
    _policy: &Org,
    cancel: &Cancellation,
) -> Result<(), String> {
    let midnight = CivilTime::new(0, 0, 0)?;
    let interval = CivilInterval::new(
        CivilDateTime {
            date: result.context.start,
            time: midnight,
        },
        CivilDateTime {
            date: result.context.end()?.add_days(1)?,
            time: midnight,
        },
    )?;
    let window = super::super::scan::date_window(Some(interval), result.context.operation.zone())?;
    for (source, document) in sources.documents.iter().enumerate() {
        if cancel.is_cancelled() {
            break;
        }
        document_clocks(result, (source, document), window, cancel)?;
    }
    result
        .rows
        .retain(|r| r.fields.contains_key("CLOCKSUM") || r.fields.contains_key("CLOCKOPEN"));
    Ok(())
}
fn record(
    result: &mut QueryResult,
    origin: (usize, usize, &str),
    window: Option<RecordedClock>,
) -> Result<(), String> {
    let (source, byte, line) = origin;
    let Some(row) = result
        .rows
        .iter_mut()
        .rev()
        .find(|r| r.source == source && r.byte == byte)
    else {
        return Ok(());
    };
    match RecordedClock::observe(line, result.context.operation.zone()) {
        Ok(Some(record)) => {
            let minutes = record.clipped(window).map_or(0, |r| r.minutes());
            if minutes > 0 {
                increment(row, "CLOCKSUM", minutes)?;
            }
        }
        Ok(None) => increment(row, "CLOCKOPEN", 1)?,
        Err(error) => {
            result.truncated = true;
            if result.issues.len() < 4096 {
                result.issues.push(SourceIssue {
                    path: result.sources[source].path.clone(),
                    message: error,
                });
            }
        }
    }
    Ok(())
}
fn increment(row: &mut QueryRow, key: &str, amount: u64) -> Result<(), String> {
    let previous = row
        .fields
        .get(key)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    row.fields.insert(
        key.into(),
        previous
            .checked_add(amount)
            .ok_or("Clock total overflow")?
            .to_string(),
    );
    Ok(())
}

fn document_clocks(
    result: &mut QueryResult,
    input: (usize, &AgendaSource),
    window: Option<RecordedClock>,
    cancel: &Cancellation,
) -> Result<(), String> {
    let (source, document) = input;
    let text = document.snapshot();
    let structure = structure::observe(text.clone(), Limits::default(), cancel);
    let literals: Vec<_> = structure
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.kind,
                StructureKind::Block(_) | StructureKind::Comment | StructureKind::FixedWidth
            )
        })
        .map(|n| n.source.clone())
        .collect();
    let headings: Vec<_> = structure
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, StructureKind::Heading { .. }))
        .map(|n| n.source.start)
        .collect();
    let mut records = 0;
    let mut byte = 0;
    for line in text.lines() {
        if cancel.is_cancelled() {
            break;
        }
        let line = line.to_string();
        if line.trim_start().starts_with("CLOCK:") && !literals.iter().any(|r| r.contains(&byte)) {
            records += 1;
            if records > 4096 {
                return Err("Agenda clock block exceeds 4096 records per source".into());
            }
            let heading = headings.iter().rev().find(|h| **h <= byte).copied();
            if let Some(heading) = heading {
                record(result, (source, heading, &line), window)?;
            }
        }
        byte += line.len();
    }
    Ok(())
}
