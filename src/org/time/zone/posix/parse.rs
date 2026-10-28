use super::Date;
use crate::org::time::date::{leap_year, month_days};

pub(super) struct Parser<'a> {
    pub rest: &'a str,
}
impl Parser<'_> {
    pub fn expect(&mut self, prefix: char) -> Result<(), String> {
        self.rest = self
            .rest
            .strip_prefix(prefix)
            .ok_or("Invalid POSIX zone separator")?;
        Ok(())
    }
    pub fn number(&mut self, min: i32, max: i32) -> Result<i32, String> {
        let len = self.rest.bytes().take_while(u8::is_ascii_digit).count();
        if len == 0 || len > 3 {
            return Err("Invalid POSIX zone number".into());
        }
        let n: i32 = self.rest[..len]
            .parse()
            .map_err(|_| "Invalid POSIX zone number")?;
        if !(min..=max).contains(&n) {
            return Err("POSIX zone number is out of range".into());
        }
        self.rest = &self.rest[len..];
        Ok(n)
    }
    pub fn name(&mut self) -> Result<(), String> {
        let name = if let Some(rest) = self.rest.strip_prefix('<') {
            let (name, tail) = rest.split_once('>').ok_or("Unclosed POSIX zone name")?;
            if !name.bytes().all(quoted_name) {
                return Err("Invalid quoted POSIX zone name".into());
            }
            self.rest = tail;
            name
        } else {
            let len = self
                .rest
                .bytes()
                .take_while(u8::is_ascii_alphabetic)
                .count();
            let name = &self.rest[..len];
            self.rest = &self.rest[len..];
            name
        };
        if !(3..=64).contains(&name.len()) {
            return Err("POSIX zone name requires 3..64 ASCII bytes".into());
        }
        Ok(())
    }
    pub fn clock(&mut self, hours: i32, signed: bool) -> Result<i32, String> {
        let sign = self.sign(signed)?;
        let h = self.number(0, hours)?;
        let m = self.component()?;
        let s = self.component()?;
        Ok(sign * (h * 3600 + m * 60 + s))
    }
    fn sign(&mut self, signed: bool) -> Result<i32, String> {
        let first = self.rest.as_bytes().first().copied();
        if !matches!(first, Some(b'+' | b'-')) {
            return Ok(1);
        }
        if !signed {
            return Err("Signed POSIX rule times require TZif version 3 or 4".into());
        }
        self.rest = &self.rest[1..];
        Ok(if first == Some(b'-') { -1 } else { 1 })
    }
    fn component(&mut self) -> Result<i32, String> {
        if !self.rest.starts_with(':') {
            return Ok(0);
        }
        self.expect(':')?;
        self.number(0, 59)
    }
}
fn quoted_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'-'
}

#[derive(Clone, Debug)]
pub(super) struct Rule {
    day: Day,
    seconds: i32,
}
#[derive(Clone, Debug)]
enum Day {
    Julian(i32),
    Ordinal(i32),
    Month(u8, u8, u8),
}
impl Rule {
    pub fn parse(text: &str, extended: bool) -> Result<Self, String> {
        let mut p = Parser { rest: text };
        let day = day(&mut p)?;
        let seconds = if p.rest.is_empty() {
            7200
        } else {
            p.expect('/')?;
            p.clock(if extended { 167 } else { 24 }, extended)?
        };
        if !p.rest.is_empty() {
            return Err("Trailing POSIX zone rule data".into());
        }
        Ok(Self { day, seconds })
    }
    pub fn instant(&self, year: i32, before: i32) -> Result<i64, String> {
        // Gregorian rules repeat each 400 years. A representative admitted year
        // also handles transition spill beyond the first/last source year.
        let representative = 2000 + year.rem_euclid(400);
        let cycles = i64::from((year - representative) / 400);
        let first = Date::new(representative as u16, 1, 1)?;
        let days = first.days() + self.day.days(representative as u16)? + cycles * 146097;
        Ok(days * 86400 + i64::from(self.seconds) - i64::from(before))
    }
}
fn day(p: &mut Parser<'_>) -> Result<Day, String> {
    if p.rest.starts_with('M') {
        return month_rule(p);
    }
    if p.rest.starts_with('J') {
        p.expect('J')?;
        return Ok(Day::Julian(p.number(1, 365)?));
    }
    Ok(Day::Ordinal(p.number(0, 365)?))
}
fn month_rule(p: &mut Parser<'_>) -> Result<Day, String> {
    p.expect('M')?;
    let month = p.number(1, 12)? as u8;
    p.expect('.')?;
    let week = p.number(1, 5)? as u8;
    p.expect('.')?;
    let weekday = p.number(0, 6)? as u8;
    Ok(Day::Month(month, week, weekday))
}
impl Day {
    fn days(&self, year: u16) -> Result<i64, String> {
        match *self {
            Self::Julian(n) => Ok(i64::from(n - 1 + i32::from(n >= 60 && leap_year(year)))),
            Self::Ordinal(n) => Ok(i64::from(n)),
            Self::Month(m, w, d) => month_day(year, m, w, d),
        }
    }
}
fn month_day(year: u16, month: u8, week: u8, weekday: u8) -> Result<i64, String> {
    let first = Date::new(year, month, 1)?;
    let delta = (7 + i32::from(weekday) - i32::from(first.weekday() % 7)) % 7;
    let day = 1 + delta + i32::from(week - 1) * 7;
    let limit = month_days(year, month);
    let day = if day > i32::from(limit) { day - 7 } else { day };
    Ok(first.days() - Date::new(year, 1, 1)?.days() + i64::from(day - 1))
}
