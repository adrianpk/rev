use super::Zone;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Local honors the captured TZ value; System explicitly uses localtime.
/// Named zones are validated relative names under the observed zoneinfo root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZoneSelection {
    Local,
    System,
    Utc,
    Iana(String),
}
impl ZoneSelection {
    pub fn parse(text: &str) -> Result<Self, String> {
        match text {
            "local" => Ok(Self::Local),
            "system" => Ok(Self::System),
            "UTC" => Ok(Self::Utc),
            _ => {
                checked_name(text)?;
                Ok(Self::Iana(text.into()))
            }
        }
    }
}
/// Explicit observation inputs permit owned fixtures without changing global TZ.
/// Native callers capture these paths and TZ once when selecting an operation.
#[derive(Clone, Debug)]
pub struct ZoneEnvironment {
    pub localtime: PathBuf,
    pub zoneinfo: PathBuf,
    pub tz: Option<OsString>,
}
impl ZoneEnvironment {
    pub fn system() -> Self {
        Self {
            localtime: "/etc/localtime".into(),
            zoneinfo: "/usr/share/zoneinfo".into(),
            tz: std::env::var_os("TZ"),
        }
    }
}
impl Zone {
    /// Read one bounded immutable observation. Subsequent operations read again,
    /// so system file replacement and environment changes need no idle watcher.
    pub fn observe(selection: &ZoneSelection, env: &ZoneEnvironment) -> Result<Self, String> {
        match selection {
            ZoneSelection::Utc => Ok(Self::utc()),
            ZoneSelection::System => read(&env.localtime),
            ZoneSelection::Local => local(env),
            ZoneSelection::Iana(name) => {
                checked_name(name)?;
                read(&env.zoneinfo.join(name))
            }
        }
    }
}
fn local(env: &ZoneEnvironment) -> Result<Zone, String> {
    let Some(tz) = &env.tz else {
        return read(&env.localtime);
    };
    let tz = tz.to_str().ok_or("Local TZ must be UTF-8")?;
    if tz.len() > 1024 {
        return Err("Local TZ exceeds 1024 bytes".into());
    }
    let tz = tz.strip_prefix(':').unwrap_or(tz);
    if matches!(tz, "" | "UTC" | "UTC0" | "GMT" | "GMT0" | "Etc/UTC") {
        return Ok(Zone::utc());
    }
    if Path::new(tz).is_absolute() {
        return read(Path::new(tz));
    }
    if checked_name(tz).is_ok() {
        let path = env.zoneinfo.join(tz);
        match std::fs::metadata(&path) {
            Ok(_) => return read(&path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(format!("Cannot observe local zone: {e}")),
        }
    }
    Zone::from_posix(tz)
}
fn read(path: &Path) -> Result<Zone, String> {
    // Preflight rejects directories and special files before opening them.
    // The opened descriptor is checked again; reads are capped even if it grows.
    let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot observe Org zone: {e}"))?;
    checked_file(&metadata)?;
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open Org zone: {e}"))?;
    checked_file(&file.metadata().map_err(|e| e.to_string())?)?;
    let mut bytes = Vec::new();
    file.take(1048577)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Zone::from_tzif(&bytes)
}
fn checked_file(metadata: &std::fs::Metadata) -> Result<(), String> {
    if !metadata.is_file() || metadata.len() > 1048576 {
        return Err("Org zone requires a regular TZif file of at most 1 MiB".into());
    }
    Ok(())
}
fn checked_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 255 || !name.bytes().all(name_byte) {
        return Err("Org zone requires local, system, UTC or a relative IANA name".into());
    }
    if name.split('/').any(|p| matches!(p, "" | "." | "..")) {
        return Err("Org IANA zone cannot contain empty, . or .. components".into());
    }
    Ok(())
}
fn name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"/_-+.".contains(&b)
}
