mod format;
use crate::file::{Files, Observation, Output, observe, temporary::Temporary};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const LIMIT: u64 = 65536;
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockState {
    pub path: PathBuf,
    pub file: Observation,
    pub heading: usize,
    pub heading_source: String,
    pub line_start: usize,
    pub line_end: usize,
    pub source: String,
    pub start_seconds: i64,
    pub acknowledged_seconds: i64,
}
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    version: u8,
    clock: Option<ClockState>,
}
/// One explicit state destination with conservative replacement admission.
/// Opening never creates a directory or visits a source document.
pub struct ClockStore {
    path: PathBuf,
    expected: Option<Observation>,
    clock: Option<ClockState>,
    blocked: bool,
}
impl ClockStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if !path.is_absolute() {
            return Err("Org clock state requires an absolute path".into());
        }
        let expected = observe(path).map_err(|e| e.to_string())?;
        let clock = read(path, expected.as_ref())?;
        if observe(path).map_err(|e| e.to_string())? != expected {
            return Err("Org clock state changed during reading".into());
        }
        Ok(Self {
            path: path.into(),
            expected,
            clock,
            blocked: false,
        })
    }
    pub fn clock(&self) -> Option<&ClockState> {
        self.clock.as_ref()
    }
    pub fn write(&mut self, clock: Option<ClockState>) -> Result<(), String> {
        self.write_with(clock, &mut Files)
    }
    /// Overrides retain the existing native output boundary's failure meaning.
    /// A failed replacement must mean no replacement; later errors are reported
    /// explicitly as published with completion unresolved.
    pub fn write_with(
        &mut self,
        clock: Option<ClockState>,
        output: &mut impl Output,
    ) -> Result<(), String> {
        let bytes = prepare_state(clock.as_ref())?;
        self.guard()?;
        let parent = self.path.parent().ok_or("Org clock state has no parent")?;
        create_directory(parent)?;
        let mut temporary = Temporary::new(parent).map_err(|e| e.to_string())?;
        if let Err(error) = stage(&mut temporary, bytes.as_bytes(), output) {
            return Err(temporary.failed(error, output).to_string());
        }
        let replaced = self
            .guard()
            .map_err(std::io::Error::other)
            .and_then(|()| output.replace(&temporary.path, &self.path));
        if let Err(error) = replaced {
            return Err(temporary.failed(error, output).to_string());
        }
        temporary.disarm();
        self.clock = clock;
        self.complete(output)
    }
    fn guard(&self) -> Result<(), String> {
        if self.blocked || observe(&self.path).map_err(|e| e.to_string())? != self.expected {
            return Err("Org clock state changed externally; retained without replacement".into());
        }
        Ok(())
    }
    fn complete(&mut self, output: &mut impl Output) -> Result<(), String> {
        self.blocked = true;
        let parent = self.path.parent().expect("admitted state parent");
        let directory = File::open(parent)
            .map_err(|e| format!("Clock state published; completion unresolved: {e}"))?;
        let synced = output.sync_directory(&directory);
        let observed = output
            .observe(&self.path)
            .map_err(|e| format!("Clock state published; completion unresolved: {e}"))?;
        self.expected = Some(observed);
        self.blocked = false;
        synced.map_err(|e| format!("Clock state published; completion unresolved: {e}"))
    }
}
impl ClockState {
    pub fn validate(&self) -> Result<(), String> {
        if !self.path.is_absolute() || self.path.as_os_str().len() > 4096 {
            return Err("Org clock source path must be absolute and at most 4096 bytes".into());
        }
        if self.line_start >= self.line_end
            || self.heading >= self.line_start
            || self.acknowledged_seconds < self.start_seconds
        {
            return Err("Org clock state has invalid source or time bounds".into());
        }
        self.validate_text()
    }
    fn validate_text(&self) -> Result<(), String> {
        if self.heading_source.len() > 4096 || self.source.len() > 4096 {
            return Err("Org clock state source fields exceed 4096 bytes".into());
        }
        if !self.heading_source.starts_with('*')
            || self.heading_source.contains(['\r', '\n'])
            || self.source.contains(['\r', '\n'])
        {
            return Err("Org clock state requires one heading and one CLOCK line".into());
        }
        Ok(())
    }
}
fn read(path: &Path, expected: Option<&Observation>) -> Result<Option<ClockState>, String> {
    if expected.is_none() {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > LIMIT as usize {
        return Err("Org clock state exceeds 65536 bytes".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let state: State = toml::from_str(text).map_err(|e| format!("Invalid Org clock state: {e}"))?;
    if state.version != 1 {
        return Err("Unsupported Org clock state version".into());
    }
    if let Some(clock) = &state.clock {
        clock.validate()?;
    }
    Ok(state.clock)
}
fn create_directory(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(|e| e.to_string())
}
fn stage(temporary: &mut Temporary, bytes: &[u8], output: &mut impl Output) -> std::io::Result<()> {
    temporary
        .file
        .write_all(bytes)
        .and_then(|()| output.flush(&mut temporary.file))
        .and_then(|()| output.sync_file(&temporary.file))
}

fn prepare_state(clock: Option<&ClockState>) -> Result<String, String> {
    if let Some(value) = clock {
        value.validate()?;
    }
    let bytes = format::encode(clock)?;
    if bytes.len() > LIMIT as usize {
        return Err("Org clock state exceeds 65536 bytes".into());
    }
    Ok(bytes)
}
