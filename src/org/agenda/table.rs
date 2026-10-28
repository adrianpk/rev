use super::clocks::{ClockQuery, ClockScope};
use crate::org::time::{CivilDateTime, CivilInterval, CivilTime, Date, Timestamp};

/// Finite data-only clock-table parameters. Unknown options never evaluate code
/// and reject an update before the original block body is replaced.
#[derive(Clone, Debug)]
pub struct ClockTable {
    pub query: ClockQuery,
    pub maxlevel: usize,
}
impl ClockTable {
    pub fn parse(source: &str, now: CivilDateTime) -> Result<Self, String> {
        let mut table = Self {
            query: ClockQuery {
                scope: ClockScope::File,
                window: None,
            },
            maxlevel: 2,
        };
        let words = words(source)?;
        let mut bounds = (None, None);
        let mut seen = std::collections::BTreeSet::new();
        for pair in words.chunks(2) {
            if pair.len() != 2 || !seen.insert(pair[0].as_str()) {
                return Err("Clocktable needs distinct option/value pairs".into());
            }
            option(&mut table, pair, &mut bounds, now)?;
        }
        table.bounds(bounds, seen.contains(":block"), now)?;
        Ok(table)
    }
    fn bounds(
        &mut self,
        bounds: (Option<CivilDateTime>, Option<CivilDateTime>),
        block: bool,
        now: CivilDateTime,
    ) -> Result<(), String> {
        match bounds {
            (None, None) => {
                if !block {
                    self.query.window = Some(year(now.date)?);
                }
                Ok(())
            }
            (Some(start), Some(end)) if !block && start < end => {
                self.query.window = Some(CivilInterval::new(start, end)?);
                Ok(())
            }
            _ => Err("Clocktable requires ordered :tstart/:tend, separate from :block".into()),
        }
    }
}
fn option(
    table: &mut ClockTable,
    pair: &[String],
    bounds: &mut (Option<CivilDateTime>, Option<CivilDateTime>),
    now: CivilDateTime,
) -> Result<(), String> {
    match pair[0].as_str() {
        ":scope" => table.query.scope = scope(&pair[1])?,
        ":maxlevel" => table.maxlevel = level(&pair[1])?,
        ":tstart" | ":tend" => set_boundary(bounds, pair)?,
        ":block" => table.query.window = block(&pair[1], now)?,
        _ => return Err(format!("Unsupported clocktable option: {}", pair[0])),
    }
    Ok(())
}
fn scope(value: &str) -> Result<ClockScope, String> {
    match value {
        "nil" | "file" => Ok(ClockScope::File),
        "subtree" => Ok(ClockScope::Subtree),
        _ => Err("Clocktable scope requires file, nil or subtree".into()),
    }
}
fn level(value: &str) -> Result<usize, String> {
    let value: usize = value
        .parse()
        .map_err(|_| "Clocktable maxlevel requires 1..256")?;
    if !(1..=256).contains(&value) {
        return Err("Clocktable maxlevel requires 1..256".into());
    }
    Ok(value)
}
fn boundary(value: &str) -> Result<CivilDateTime, String> {
    let value = if value.starts_with(['[', '<']) {
        value.to_owned()
    } else {
        format!("[{value}]")
    };
    let value = Timestamp::parse(&value)?;
    if value.repeater()?.is_some() || value.warning_delay()?.is_some() || value.end_time().is_some()
    {
        return Err("Clocktable boundary requires one date/time".into());
    }
    Ok(CivilDateTime {
        date: value.date(),
        time: value.time().unwrap_or(CivilTime::new(0, 0, 0)?),
    })
}
fn year(date: Date) -> Result<CivilInterval, String> {
    between(
        Date::new(date.year(), 1, 1)?,
        Date::new(date.year() + 1, 1, 1)?,
    )
}
fn between(start: Date, end: Date) -> Result<CivilInterval, String> {
    let time = CivilTime::new(0, 0, 0)?;
    CivilInterval::new(
        CivilDateTime { date: start, time },
        CivilDateTime { date: end, time },
    )
}
fn block(value: &str, now: CivilDateTime) -> Result<Option<CivilInterval>, String> {
    if value == "untilnow" {
        return Ok(None);
    }
    let today = now.date;
    let result = match value {
        "today" | "yesterday" => day(value, today),
        "thisyear" | "lastyear" => annual(value, today),
        "thismonth" | "lastmonth" => month(value, today),
        "thisweek" | "lastweek" => week(value, today),
        _ => super::clocks::date(value).and_then(|date| day("today", date)),
    };
    result.map(Some)
}
fn day(value: &str, date: Date) -> Result<CivilInterval, String> {
    let start = if value == "yesterday" {
        date.add_days(-1)?
    } else {
        date
    };
    between(start, start.add_days(1)?)
}
fn annual(value: &str, date: Date) -> Result<CivilInterval, String> {
    let date = if value == "lastyear" {
        date.add_years(-1)?
    } else {
        date
    };
    year(date)
}
fn set_boundary(
    bounds: &mut (Option<CivilDateTime>, Option<CivilDateTime>),
    pair: &[String],
) -> Result<(), String> {
    let value = Some(boundary(&pair[1])?);
    if pair[0] == ":tstart" {
        bounds.0 = value;
    } else {
        bounds.1 = value;
    }
    Ok(())
}
fn month(value: &str, date: Date) -> Result<CivilInterval, String> {
    let first = Date::new(date.year(), date.month(), 1)?;
    let end = if value == "lastmonth" {
        first
    } else {
        first.add_months(1)?
    };
    between(end.add_months(-1)?, end)
}
fn week(value: &str, date: Date) -> Result<CivilInterval, String> {
    let start = date.add_days(1 - i64::from(date.weekday()))?;
    let end = if value == "lastweek" {
        start
    } else {
        start.add_days(7)?
    };
    between(end.add_days(-7)?, end)
}
fn words(source: &str) -> Result<Vec<String>, String> {
    if source.len() > 4096 {
        return Err("Clocktable options exceed 4096 UTF-8 bytes".into());
    }
    let mut result = Vec::new();
    let mut rest = source.trim();
    while !rest.is_empty() {
        if result.len() == 32 {
            return Err("Clocktable exceeds 16 options".into());
        }
        let (value, tail) = word(rest)?;
        result.push(value.into());
        rest = tail.trim_start();
    }
    Ok(result)
}
fn word(source: &str) -> Result<(&str, &str), String> {
    if let Some(source) = source.strip_prefix('"') {
        let end = source
            .find('"')
            .ok_or("Unclosed clocktable option string")?;
        let tail = &source[end + 1..];
        if !tail.is_empty() && !tail.starts_with(char::is_whitespace) {
            return Err("Clocktable options require separators".into());
        }
        return Ok((&source[..end], tail));
    }
    let end = source.find(char::is_whitespace).unwrap_or(source.len());
    Ok((&source[..end], &source[end..]))
}
