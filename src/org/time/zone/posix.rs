use super::super::{CivilDateTime, Date};
mod parse;
use parse::{Parser, Rule};

#[derive(Clone, Debug)]
pub(super) struct Posix {
    pub standard: i32,
    pub daylight: Option<Daylight>,
}
#[derive(Clone, Debug)]
pub(super) struct Daylight {
    pub offset: i32,
    start: Rule,
    end: Rule,
}
impl Posix {
    pub fn parse(text: &str, extended: bool) -> Result<Self, String> {
        if text.len() > 1024 || !text.is_ascii() {
            return Err("POSIX zone requires at most 1024 ASCII bytes".into());
        }
        let mut parser = Parser { rest: text };
        parser.name()?;
        let standard = -parser.clock(24, true)?;
        checked_offset(standard)?;
        let daylight = daylight(&mut parser, standard, extended)?;
        Ok(Self { standard, daylight })
    }
    pub fn offset_at(&self, seconds: i64) -> Result<i32, String> {
        let Some(daylight) = &self.daylight else {
            return Ok(self.standard);
        };
        let year = civil_year(seconds)?;
        let mut latest = (i64::MIN, 0, self.standard);
        // Four years suffice even when signed rule times spill a week into an
        // adjacent year. No recurrence expansion is retained between operations.
        for year in year - 2..=year + 1 {
            for event in self.events(daylight, year)? {
                if event.0 <= seconds && (event.0, event.1) > (latest.0, latest.1) {
                    latest = event;
                }
            }
        }
        Ok(latest.2)
    }
    fn events(&self, dst: &Daylight, year: i32) -> Result<[(i64, u8, i32); 2], String> {
        Ok([
            (dst.start.instant(year, self.standard)?, 1, dst.offset),
            (dst.end.instant(year, dst.offset)?, 0, self.standard),
        ])
    }
}
fn civil_year(seconds: i64) -> Result<i32, String> {
    if !(-62135683200..=253402387199).contains(&seconds) {
        return Err("POSIX zone instant is outside the Org civil profile".into());
    }
    let civil = CivilDateTime::from_seconds(seconds.clamp(-62135596800, 253402300799))?;
    Ok(i32::from(civil.date.year()))
}
fn daylight(
    parser: &mut Parser<'_>,
    standard: i32,
    extended: bool,
) -> Result<Option<Daylight>, String> {
    if parser.rest.is_empty() {
        return Ok(None);
    }
    parser.name()?;
    let offset = if parser.rest.starts_with(',') {
        standard + 3600
    } else {
        -parser.clock(24, true)?
    };
    checked_offset(offset)?;
    parser.expect(',')?;
    let (start, end) = parser
        .rest
        .split_once(',')
        .ok_or("POSIX daylight zone requires start and end rules")?;
    Ok(Some(Daylight {
        offset,
        start: Rule::parse(start, extended)?,
        end: Rule::parse(end, extended)?,
    }))
}
pub(super) fn checked_offset(offset: i32) -> Result<(), String> {
    if !(-86400..=86400).contains(&offset) {
        return Err("Org zone offset exceeds 24 hours".into());
    }
    Ok(())
}
