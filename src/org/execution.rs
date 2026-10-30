//! Confirmed external source jobs. Profiles carry host authority, not a sandbox.
mod headers;
mod input;
mod results;
mod variables;
pub(crate) use input::{Input, Prepare};
mod run;
pub(in crate::org) mod scratch;
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
    pub source_path: Option<PathBuf>,
    pub result: config::SourceResult,
    pub state: State,
    pub status: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub detail: String,
    pub(crate) site: results::Site,
    pub(crate) related: Vec<crate::org::io::DestinationSource>,
    pub(crate) value: Vec<u8>,
    pub(crate) proposed: Option<String>,
}
pub(crate) struct Work {
    scan: Scan,
    input: Input,
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
        input: Input,
        profile: SourceProgram,
        snapshot: &config::Snapshot,
    ) -> Result<Self, String> {
        crate::org::source::current(buffer, input.stamp).map_err(|e| e.to_string())?;
        let definition = input.definition.clone();
        headers::admit(&definition)?;
        input.site.admit(profile.result)?;
        results::Site::collection(&definition.headers, profile.result)?;
        variables::body(&definition, "", &profile)?;
        profile.validate()?;
        let mut profile = profile;
        resolve(&mut profile, snapshot)?;
        let policy = &snapshot.settings().org.limits;
        Ok(Self {
            scan: input.scan.clone(),
            directory: source_directory(input.source_path.as_deref(), profile.cwd)?,
            input,
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
        self.input.query || self.definition.headers.get("eval") == Some("query")
    }
}
fn source_directory(
    path: Option<&std::path::Path>,
    cwd: config::SourceCwd,
) -> Result<Option<PathBuf>, String> {
    if cwd == config::SourceCwd::Workspace {
        return Ok(None);
    }
    path.and_then(std::path::Path::parent)
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

impl Outcome {
    pub(crate) fn current(&self, buffers: &crate::buffer::Buffers) -> Result<(), String> {
        input::related_current(&self.related, buffers)
    }
    pub(crate) fn publication(
        &self,
        buffers: &crate::buffer::Buffers,
    ) -> Result<Option<crate::buffer::edit::Replacement>, String> {
        let buffer = buffers
            .get(self.site.stamp().buffer)
            .map_err(|e| e.to_string())?;
        self.site.edit(buffer, self)
    }
}
pub(crate) fn call_at(buffer: &Buffer, at: crate::coordinates::Scalar) -> bool {
    let Ok(coordinates) = crate::coordinates::Coordinates::new(buffer) else {
        return false;
    };
    let Ok(byte) = coordinates.to_byte(at) else {
        return false;
    };
    Scan::new(buffer, byte).is_ok_and(|s| input::calls::at(&s))
}
