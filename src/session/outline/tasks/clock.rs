//! The session captures one local operation timestamp only when logging is selected.
//! TZif reads are bounded and never execute a shell or a source-language form.
mod tzif;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

pub(in crate::session) fn now() -> Result<String, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let seconds = i64::try_from(seconds).map_err(|e| e.to_string())?;
    let offset = local_offset(seconds)?;
    format(seconds + i64::from(offset))
}
fn local_offset(seconds: i64) -> Result<i32, String> {
    let zone = std::env::var("TZ").ok();
    if matches!(
        zone.as_deref(),
        Some("UTC" | "UTC0" | "GMT" | "GMT0" | "Etc/UTC")
    ) {
        return Ok(0);
    }
    let path = zone_path(zone.as_deref())?;
    let file =
        std::fs::File::open(path).map_err(|e| format!("Cannot read local task time zone: {e}"))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Task time zone must be a regular TZif file".into());
    }
    let mut bytes = Vec::new();
    file.take(1048577)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1048576 {
        return Err("Task time zone exceeds 1 MiB".into());
    }
    tzif::offset(&bytes, seconds)
}
fn zone_path(zone: Option<&str>) -> Result<std::path::PathBuf, String> {
    let Some(zone) = zone else {
        return Ok("/etc/localtime".into());
    };
    let zone = zone.strip_prefix(':').unwrap_or(zone);
    let path = std::path::Path::new(zone);
    if path.is_absolute() {
        return Ok(path.into());
    }
    if path
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
        || zone.is_empty()
    {
        return Err("Task logging requires a TZif zone name, absolute TZif path or UTC".into());
    }
    Ok(std::path::Path::new("/usr/share/zoneinfo").join(path))
}
fn format(seconds: i64) -> Result<String, String> {
    let days = seconds.div_euclid(86400);
    let civil = days + 719468;
    let era = civil.div_euclid(146097);
    let day = civil - era * 146097;
    let year = (day - day / 1460 + day / 36524 - day / 146096) / 365;
    let day_year = day - (365 * year + year / 4 - year / 100);
    let month = (5 * day_year + 2) / 153;
    let d = day_year - (153 * month + 2) / 5 + 1;
    let m = month + if month < 10 { 3 } else { -9 };
    let y = year + era * 400 + i64::from(m <= 2);
    if !(1..=9999).contains(&y) {
        return Err("Task timestamp year is outside 0001..9999".into());
    }
    let weekday = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"][days.rem_euclid(7) as usize];
    let time = seconds.rem_euclid(86400);
    Ok(format!(
        "[{y:04}-{m:02}-{d:02} {weekday} {:02}:{:02}]",
        time / 3600,
        time / 60 % 60
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Calendar conversion covers the epoch, leap day and day rollover using
    // fixed instants; it neither reads the host clock nor changes TZ globally.
    #[test]
    fn civil_stamp() {
        for (seconds, expected) in [
            (0, "[1970-01-01 Thu 00:00]"),
            (-1, "[1969-12-31 Wed 23:59]"),
            (951782400, "[2000-02-29 Tue 00:00]"),
            (1793207100, "[2026-10-28 Wed 17:05]"),
        ] {
            assert_eq!(format(seconds).unwrap(), expected);
        }
    }
}
