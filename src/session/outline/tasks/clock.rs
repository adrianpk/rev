//! The session captures one local operation timestamp only when logging is selected.
//! TZif reads are bounded and never execute a shell or a source-language form.
mod tzif;
use crate::org::time::OperationTime;
use std::io::Read;
use std::time::SystemTime;

pub(in crate::session) fn now() -> Result<String, String> {
    let time = OperationTime::capture(|| Ok(SystemTime::now()))?;
    let offset = local_offset(time.unix_seconds())?;
    Ok(time.civil_at_offset(offset)?.format_timestamp(false))
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
