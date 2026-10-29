//! Explicit bounded source observations. These values never own edit authority.
use crate::org::io::source as read;
mod scan;
mod walk;
use crate::{
    buffer::{BufferId, Stamp},
    config::Snapshot,
    file::Observation,
};
use ropey::Rope;
pub use scan::SourceScan;
use std::path::PathBuf;

/// User-selected paths and/or live Org buffer identities. Directories admit
/// immediate .org children only; no recursive or implicit discovery occurs.
#[derive(Clone, Debug, Default)]
pub struct SourceSelection {
    pub paths: Vec<PathBuf>,
    pub buffers: Vec<BufferId>,
}
impl SourceSelection {
    pub fn paths(paths: &[String]) -> Result<Self, String> {
        if paths.len() > 4096 {
            return Err("Agenda selection exceeds 4096 roots".into());
        }
        if paths
            .iter()
            .any(|p| p.is_empty() || p.len() > 4096 || p.contains('\0') || p.contains("://"))
        {
            return Err("Agenda sources require nonempty local paths of at most 4096 bytes".into());
        }
        Ok(Self {
            paths: paths.iter().map(PathBuf::from).collect(),
            buffers: Vec::new(),
        })
    }
    /// Resolve against the assigning configuration source, without IO,
    /// environment expansion, or observing the selected paths at startup.
    pub fn configured(snapshot: &Snapshot) -> Result<Self, String> {
        let mut selection = Self::paths(&snapshot.settings().org.agenda.sources)?;
        if let Some(root) = snapshot
            .origin(&["org", "agenda", "sources"])
            .and_then(|p| p.parent())
        {
            for path in &mut selection.paths {
                if path.is_relative() {
                    *path = root.join(&*path);
                }
            }
        }
        Ok(selection)
    }
}
/// Provenance is evidence for refresh/navigation; it is never permission to edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceVersion {
    Buffer(Stamp),
    Disk(Observation),
}
#[derive(Clone, Debug)]
pub struct AgendaSource {
    pub(crate) bom: bool,
    pub path: Option<PathBuf>,
    pub version: SourceVersion,
    text: Rope,
    pub(in crate::org::agenda) aliases: Vec<PathBuf>,
}
impl AgendaSource {
    pub(in crate::org) fn snapshot(&self) -> Rope {
        self.text.clone()
    }

    pub fn text(&self) -> ropey::RopeSlice<'_> {
        self.text.slice(..)
    }
}
#[derive(Clone, Debug)]
pub struct SourceIssue {
    pub path: Option<PathBuf>,
    pub message: String,
}
/// Every admitted document is whole, even when other sources fail or truncate.
#[derive(Clone, Debug, Default)]
pub struct SourceSet {
    pub documents: Vec<AgendaSource>,
    pub issues: Vec<SourceIssue>,
    pub truncated: bool,
    pub cancelled: bool,
    pub bytes: usize,
}
impl SourceSet {
    pub fn complete(&self) -> bool {
        self.issues.is_empty() && !self.truncated && !self.cancelled
    }
    pub(super) fn issue(&mut self, path: Option<PathBuf>, message: String) {
        if self.issues.len() < 4096 {
            self.issues.push(SourceIssue { path, message });
        } else {
            self.truncated = true;
        }
    }
    /// Drop stale buffer observations before display. Other admitted sources stay
    /// usable, and stale evidence becomes an explicit incomplete-source issue.
    pub fn revalidate(&mut self, buffers: &crate::buffer::Buffers) {
        let mut stale = Vec::new();
        self.documents.retain(|source| {
            let current = match source.version {
                SourceVersion::Buffer(stamp) => {
                    buffers.get(stamp.buffer).is_ok_and(|b| b.stamp() == stamp)
                }
                SourceVersion::Disk(_) => true,
            };
            if !current {
                stale.push(source.path.clone());
            }
            current
        });
        for path in stale {
            self.issue(path, "Open buffer changed; refresh required".into());
        }
        self.bytes = self.documents.iter().map(|s| s.text.len_bytes()).sum();
    }
    pub fn feedback(&self) -> String {
        let state = if self.complete() {
            "complete"
        } else {
            "incomplete"
        };
        let mut message = format!(
            "Org agenda sources: {state}; {} documents, {} bytes",
            self.documents.len(),
            self.bytes
        );
        if self.cancelled {
            message.push_str("; cancelled");
        }
        if self.truncated {
            message.push_str("; limit reached");
        }
        for issue in self.issues.iter().take(8) {
            use std::fmt::Write;
            let _ = write!(message, "; {:?}: {}", issue.path, issue.message);
        }
        if self.issues.len() > 8 {
            message.push_str("; more failures retained in source results");
        }
        message
    }
}
