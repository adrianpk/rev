use super::{Format, format::Summary};
use crate::{
    config::Org,
    org::{
        Cancellation, Duration,
        agenda::{ClockRow, QueryResult, QueryRow},
    },
};
use std::collections::BTreeMap;
pub(super) fn level(row: &QueryRow) -> usize {
    row.fields
        .get("LEVEL")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}
pub(super) fn clocks(
    result: &mut QueryResult,
    rows: &[ClockRow],
    cancel: &Cancellation,
) -> Result<(), String> {
    let mut totals: Vec<_> = rows
        .iter()
        .map(|r| (r.minutes, r.open as u64, r.unresolved as u64))
        .collect();
    for i in (0..rows.len()).rev() {
        if cancel.is_cancelled() {
            return Err("Column report cancelled".into());
        }
        if let Some(parent) = rows[i].parent {
            totals[parent] = add(totals[parent], totals[i])?;
        }
    }
    let totals: BTreeMap<_, _> = rows.iter().zip(totals).map(|(r, v)| (r.byte, v)).collect();
    for row in &mut result.rows {
        if let Some((minutes, open, unresolved)) = totals.get(&row.byte) {
            row.fields.insert(
                "CLOCKSUM".into(),
                Duration::from_minutes(*minutes as f64)?.format_clock(false)?,
            );
            row.fields.insert("CLOCKOPEN".into(), open.to_string());
            row.fields
                .insert("CLOCKUNRESOLVED".into(), unresolved.to_string());
        }
    }
    Ok(())
}
fn add(a: (u64, u64, u64), b: (u64, u64, u64)) -> Result<(u64, u64, u64), String> {
    let sum = |a: u64, b: u64| {
        a.checked_add(b)
            .ok_or_else(|| "Column clock total overflow".to_owned())
    };
    Ok((sum(a.0, b.0)?, sum(a.1, b.1)?, sum(a.2, b.2)?))
}
pub(super) fn properties(
    result: &mut QueryResult,
    format: &Format,
    policy: &Org,
    cancel: &Cancellation,
) -> Result<(), String> {
    let parents = parents(&result.rows, cancel)?;
    for column in &format.columns {
        if let Some(kind) = column.summary {
            summarize(
                &mut result.rows,
                &parents,
                (&column.key, kind),
                (policy, cancel),
            )?;
        }
    }
    Ok(())
}
fn parents(rows: &[QueryRow], cancel: &Cancellation) -> Result<Vec<Option<usize>>, String> {
    let mut stack = Vec::<usize>::new();
    let mut parents = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err("Column report cancelled".into());
        }
        while stack.last().is_some_and(|p| level(&rows[*p]) >= level(row)) {
            stack.pop();
        }
        parents.push(stack.last().copied());
        stack.push(i);
    }
    Ok(parents)
}
fn summarize(
    rows: &mut [QueryRow],
    parents: &[Option<usize>],
    column: (&str, Summary),
    data: (&Org, &Cancellation),
) -> Result<(), String> {
    let mut totals = vec![0.0; rows.len()];
    let mut children = vec![false; rows.len()];
    for i in (0..rows.len()).rev() {
        if data.1.is_cancelled() {
            return Err("Column report cancelled".into());
        }
        let value = if children[i] {
            totals[i]
        } else {
            number(
                rows[i].fields.get(column.0).map_or("", String::as_str),
                column.1,
                data.0,
            )?
        };
        if !value.is_finite() {
            return Err("Column sum is not finite".into());
        }
        let text = match column.1 {
            Summary::Number => value.to_string(),
            Summary::Duration => Duration::from_minutes(value)?.format_clock(false)?,
        };
        rows[i].fields.insert(column.0.into(), text);
        if let Some(parent) = parents[i] {
            totals[parent] += value;
            children[parent] = true;
        }
    }
    Ok(())
}
fn number(value: &str, kind: Summary, policy: &Org) -> Result<f64, String> {
    if value.is_empty() {
        return Ok(0.0);
    }
    match kind {
        Summary::Number => value
            .parse::<f64>()
            .map_err(|_| "Column summary requires finite numbers".into()),
        Summary::Duration => Duration::parse(value, &policy.duration.units).map(|v| v.minutes()),
    }
}
