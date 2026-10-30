//! Confirmed external source jobs. Profiles carry host authority, not a sandbox.
mod headers;
mod run;
mod scratch;
use crate::{
    buffer::Buffer,
    config::{self, SourceProgram},
    coordinates::Byte,
    org::{
        Cancellation,
        block::{Definition, Scan},
    },
};
use std::path::PathBuf;
/// The process and cleanup outcome. Failed jobs never become successful results.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Stale,
}
/// Bounded actual output, retained with the exact input definition for inspection.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub definition: Definition,
    pub program: String,
    pub result: config::SourceResult,
    pub state: State,
    pub status: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub detail: String,
}
pub(crate) struct Work {
    scan: Scan,
    definition: Definition,
    profile: SourceProgram,
    directory: Option<PathBuf>,
    seconds: usize,
    output_bytes: usize,
    expansion_bytes: usize,
}
impl Work {
    pub fn new(
        buffer: &Buffer,
        definition: Definition,
        profile: SourceProgram,
        snapshot: &config::Snapshot,
    ) -> Result<Self, String> {
        definition.current(buffer)?;
        headers::admit(&definition)?;
        profile.validate()?;
        let mut profile = profile;
        resolve(&mut profile, snapshot)?;
        let policy = &snapshot.settings().org.limits;
        Ok(Self {
            scan: Scan::new(buffer, Byte(definition.body.start))?,
            directory: source_directory(buffer, profile.cwd)?,
            seconds: (profile.seconds as usize).min(policy.job_seconds),
            output_bytes: (profile.output_bytes as usize).min(policy.job_output_bytes),
            expansion_bytes: policy.expansion_bytes,
            definition,
            profile,
        })
    }
    pub fn label(&self) -> String {
        format!(
            "{} ({}) using {}",
            self.profile.name, self.profile.language, self.profile.executable
        )
    }
    pub fn force_query(&self) -> bool {
        self.definition.headers.get("eval") == Some("query")
    }
}
fn source_directory(buffer: &Buffer, cwd: config::SourceCwd) -> Result<Option<PathBuf>, String> {
    if cwd == config::SourceCwd::Workspace {
        return Ok(None);
    }
    buffer
        .file()
        .and_then(|f| f.path().parent())
        .map(PathBuf::from)
        .map(Some)
        .ok_or("Source cwd requires an associated Org file".into())
}
fn resolve(profile: &mut SourceProgram, snapshot: &config::Snapshot) -> Result<(), String> {
    let root = snapshot
        .origin(&["org", "source", "programs"])
        .and_then(std::path::Path::parent);
    executable(&mut profile.executable, root)?;
    for step in &mut profile.steps {
        executable(&mut step.executable, root)?;
    }
    Ok(())
}
fn executable(value: &mut String, root: Option<&std::path::Path>) -> Result<(), String> {
    if std::path::Path::new(value).is_relative()
        && value.contains('/')
        && let Some(root) = root
    {
        *value = root
            .join(&*value)
            .to_str()
            .ok_or("Source executable path is not UTF-8")?
            .into();
    }
    Ok(())
}
pub(super) fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Source execution cancelled or expired".into())
    } else {
        Ok(())
    }
}
