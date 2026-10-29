//! One explicit capture destination read. Preparation snapshots canonical ropes;
//! run performs bounded IO, without visits, directories, edits or source writes.
use super::source::{self, Budget};
use crate::{
    buffer::{Buffers, Stamp},
    config::OrgLimits,
    file::{self, Location, Observation},
    org::{Cancellation, Source},
};
use ropey::{Rope, RopeSlice};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DestinationVersion {
    Buffer(Stamp),
    Disk(Observation),
    Missing,
}
#[derive(Clone, Debug)]
pub struct DestinationSource {
    pub path: PathBuf,
    pub version: DestinationVersion,
    pub readonly: bool,
    pub bom: bool,
    text: Rope,
}
impl DestinationSource {
    pub fn text(&self) -> RopeSlice<'_> {
        self.text.slice(..)
    }
    pub(in crate::org) fn snapshot(&self) -> Rope {
        self.text.clone()
    }
}
struct Open {
    source: DestinationSource,
    identity: Option<(u64, u64)>,
}
pub struct CaptureRead {
    path: PathBuf,
    open: Vec<Open>,
    limits: OrgLimits,
}
impl CaptureRead {
    pub fn new(buffers: &Buffers, path: &Path, limits: &OrgLimits) -> Result<Self, String> {
        valid_limits(limits)?;
        let value = path
            .to_str()
            .ok_or("Capture destination path must be UTF-8")?;
        crate::config::capture::path(value)?;
        if value.is_empty() {
            return Err("Enter a capture destination file".into());
        }
        let mut open = Vec::new();
        for b in buffers.iter().filter(|b| b.file().is_some()) {
            if open.len() == 4096 {
                return Err("Capture snapshot exceeds 4096 open files".into());
            }
            let file = b.file().expect("associated file");
            open.push(Open {
                source: DestinationSource {
                    path: file.path().to_owned(),
                    version: DestinationVersion::Buffer(b.stamp()),
                    readonly: b.is_readonly(),
                    bom: b.has_bom(),
                    text: Source::new(b).snapshot().map_err(|e| e.to_string())?,
                },
                identity: file.observation().map(|o| (o.device, o.inode)),
            });
        }
        Ok(Self {
            path: path.to_owned(),
            open,
            limits: limits.clone(),
        })
    }
    pub fn run(self, cancel: &Cancellation) -> Result<DestinationSource, String> {
        let budget = Budget {
            cancel,
            deadline: Instant::now() + Duration::from_secs(self.limits.job_seconds as u64),
        };
        budget.check()?;
        let location = file::resolve(&self.path).map_err(|e| e.to_string())?;
        if let Some(source) = self.canonical(&location)? {
            return Ok(source);
        }
        if !location.is_present() {
            budget.check()?;
            return Ok(DestinationSource {
                path: location.path().to_owned(),
                version: DestinationVersion::Missing,
                readonly: false,
                bom: false,
                text: Rope::new(),
            });
        }
        self.disk(&location, &budget)
    }
    fn canonical(&self, location: &Location) -> Result<Option<DestinationSource>, String> {
        let identity = location.observation().map(|o| (o.device, o.inode));
        let mut matches = self.open.iter().filter(|s| {
            s.source.path == location.path() || identity.is_some() && s.identity == identity
        });
        let Some(open) = matches.next() else {
            return Ok(None);
        };
        if matches.next().is_some() {
            return Err(
                "Multiple canonical buffers share the capture destination; resolve ownership first"
                    .into(),
            );
        }
        self.bytes(open.source.text.len_bytes())?;
        Ok(Some(open.source.clone()))
    }
    fn disk(&self, location: &Location, budget: &Budget<'_>) -> Result<DestinationSource, String> {
        let limit = self
            .limits
            .agenda_file_bytes
            .min(self.limits.agenda_total_bytes);
        if location
            .observation()
            .is_some_and(|o| o.length > limit as u64)
        {
            return Err("Capture source byte limit reached".into());
        }
        let (read, _) = source::source(location.path(), limit, budget);
        let (loaded, observed) = read.map_err(|e| format!("Capture destination: {e}"))?;
        let (text, bom) = loaded.into_parts();
        Ok(DestinationSource {
            path: location.path().to_owned(),
            version: DestinationVersion::Disk(observed),
            readonly: false,
            bom,
            text,
        })
    }
    fn bytes(&self, size: usize) -> Result<(), String> {
        if size
            > self
                .limits
                .agenda_file_bytes
                .min(self.limits.agenda_total_bytes)
        {
            return Err("Capture source byte limit reached".into());
        }
        Ok(())
    }
}
fn valid_limits(limits: &OrgLimits) -> Result<(), String> {
    if !(1..=33554432).contains(&limits.agenda_file_bytes)
        || !(1..=268435456).contains(&limits.agenda_total_bytes)
        || !(1..=600).contains(&limits.job_seconds)
    {
        return Err("Capture requires bounded source bytes and job seconds".into());
    }
    Ok(())
}
