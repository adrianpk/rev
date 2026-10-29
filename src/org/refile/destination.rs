use crate::{
    buffer::{Buffer, Buffers, Visit},
    org::{
        agenda::{AgendaSource, SourceVersion},
        io::{DestinationSource, DestinationVersion},
    },
};
/// Actual immutable observations from the two existing explicit source readers.
/// A missing selected file is distinct from an unnamed canonical buffer.
pub(in crate::org) enum Destination {
    Agenda(AgendaSource),
    File(DestinationSource),
}
impl Destination {
    pub fn check(&self, buffer: &Buffer) -> Result<(), String> {
        let span = buffer
            .span(0..buffer.len_chars())
            .map_err(|e| e.to_string())?;
        let (stamp, path, bom, text) = match self {
            Self::Agenda(s) => (
                match s.version {
                    SourceVersion::Buffer(v) => Some(v),
                    _ => None,
                },
                s.path.as_deref(),
                Some(s.bom),
                s.text(),
            ),
            Self::File(s) => (
                match s.version {
                    DestinationVersion::Buffer(v) => Some(v),
                    _ => None,
                },
                Some(s.path.as_path()),
                (!matches!(s.version, DestinationVersion::Missing)).then_some(s.bom),
                s.text(),
            ),
        };
        if stamp.is_some_and(|v| v != buffer.stamp())
            || buffer.file().map(|f| f.path()) != path
            || bom.is_some_and(|v| v != buffer.has_bom())
            || buffer.slice(&span).map_err(|e| e.to_string())? != text
        {
            return Err("Move destination changed; nothing moved".into());
        }
        Ok(())
    }
    pub fn visit(&self, buffers: &Buffers) -> Result<Visit, String> {
        match self {
            Self::File(s) => {
                if matches!(s.version, DestinationVersion::Missing) {
                    missing_parent(&s.path)?;
                }
                s.prepare_visit(buffers)
            }
            Self::Agenda(s) => match &s.version {
                SourceVersion::Buffer(stamp) => {
                    self.check(buffers.get(stamp.buffer).map_err(|e| e.to_string())?)?;
                    Ok(Visit::Existing(stamp.buffer))
                }
                SourceVersion::Disk(observed) => buffers
                    .prepare_observed_visit(
                        s.path.as_deref().ok_or("Move disk target has no path")?,
                        crate::file::Loaded::from_snapshot(s.snapshot(), s.bom),
                        observed,
                    )
                    .map_err(|e| e.to_string()),
            },
        }
    }
}

fn missing_parent(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let parent = path
        .parent()
        .ok_or("Archive destination has no parent directory")?;
    let metadata = std::fs::metadata(parent).map_err(|e| format!("Archive directory: {e}"))?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o222 == 0 {
        return Err("Archive destination directory is unavailable or read-only".into());
    }
    Ok(())
}
