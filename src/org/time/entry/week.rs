use super::super::Date;

pub(super) fn read(text: &str, default: Date) -> Result<Date, String> {
    let (year, tail) = text.split_once('w').ok_or("Invalid Org ISO week")?;
    let year = year.trim_end_matches('-');
    let year = if year.is_empty() {
        default.year()
    } else {
        year.parse::<u16>()
            .map_err(|_| "Invalid Org ISO week year")?
    };
    let tail = tail.trim();
    let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
    let week: u8 = tail[..digits]
        .parse()
        .map_err(|_| "Invalid Org ISO week number")?;
    let day = tail[digits..].trim_start_matches('-').trim();
    let day = if day.is_empty() {
        1
    } else {
        super::date::weekday(day)
            .or_else(|| day.parse::<u8>().ok())
            .ok_or("Invalid Org ISO weekday")?
    };
    iso_date(year, week, day)
}
fn iso_date(year: u16, week: u8, day: u8) -> Result<Date, String> {
    if !(1..=53).contains(&week) || !(1..=7).contains(&day) {
        return Err("Org ISO week or weekday is out of range".into());
    }
    let jan4 = Date::new(year, 1, 4)?;
    let monday = jan4.add_days(1 - i64::from(jan4.weekday()))?;
    let result = monday.add_days(i64::from(week - 1) * 7 + i64::from(day - 1))?;
    let thursday = result.add_days(4 - i64::from(result.weekday()))?;
    if thursday.year() != year {
        return Err("Org ISO week does not exist in this year".into());
    }
    Ok(result)
}
