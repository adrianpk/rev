use super::{
    Calendar, SourceSet, SourceVersion,
    sources::read::{self, Budget},
};
use crate::{
    file::{Loaded, Observation},
    org::Cancellation,
};
use ropey::Rope;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
pub(crate) struct Navigation {
    path: PathBuf,
    expected: Observation,
    text: Rope,
    byte: usize,
    limit: usize,
    seconds: usize,
}
pub(crate) struct NavigationCandidate {
    pub path: PathBuf,
    pub observed: Observation,
    pub loaded: Loaded,
    pub byte: usize,
}
impl Calendar {
    pub(crate) fn disk_navigation(
        &self,
        row: usize,
        sources: &SourceSet,
        limits: &crate::config::OrgLimits,
    ) -> Result<Navigation, String> {
        let row = self.rows.get(row).ok_or("Select an agenda entry")?;
        let origin = &self.sources[row.source];
        let SourceVersion::Disk(expected) = &origin.version else {
            return Err("Agenda row belongs to an open buffer".into());
        };
        let source = sources
            .documents
            .iter()
            .find(|s| s.path == origin.path && s.version == origin.version)
            .ok_or("Source observation changed; refresh required")?;
        Ok(Navigation {
            path: origin.path.clone().ok_or("Disk source has no path")?,
            expected: expected.clone(),
            text: source.snapshot(),
            byte: row.byte,
            limit: limits.agenda_file_bytes,
            seconds: limits.job_seconds,
        })
    }
}
impl Navigation {
    pub(crate) fn run(self, cancel: &Cancellation) -> Result<NavigationCandidate, String> {
        let budget = Budget {
            cancel,
            deadline: Instant::now() + Duration::from_secs(self.seconds as u64),
        };
        budget.check()?;
        let (read, _) = read::source(&self.path, self.limit, &budget);
        let (loaded, observed) = read?;
        if observed != self.expected || loaded.observed_text() != self.text.slice(..) {
            return Err("Agenda disk source changed; refresh required".into());
        }
        Ok(NavigationCandidate {
            path: self.path,
            observed,
            loaded,
            byte: self.byte,
        })
    }
}
