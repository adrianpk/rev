use super::{
    AgendaSource, SourceSelection, SourceSet, SourceVersion,
    read::{self, Budget},
    walk::{Step, Walk},
};
use crate::{
    buffer::{Buffers, Stamp},
    config::OrgLimits,
    org::{Cancellation, Source},
};
use ropey::Rope;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
struct Open {
    path: Option<PathBuf>,
    stamp: Stamp,
    text: Rope,
    identity: Option<(u64, u64)>,
}
/// One immutable request for the session's existing owned analysis/IO job.
/// Construction snapshots shared rope nodes and performs no filesystem IO.
pub struct SourceScan {
    selection: SourceSelection,
    open: Vec<Open>,
    limits: OrgLimits,
    seen_files: BTreeSet<(u64, u64)>,
    seen_buffers: BTreeSet<crate::buffer::BufferId>,
    spent: usize,
    result: SourceSet,
}
impl SourceScan {
    pub fn new(
        buffers: &Buffers,
        selection: SourceSelection,
        limits: &OrgLimits,
    ) -> Result<Self, String> {
        validate(&selection, limits)?;
        let mut open = Vec::new();
        for buffer in buffers
            .iter()
            .filter(|b| b.file().is_some() || selection.buffers.contains(&b.id()))
        {
            if open.len() == 4096 {
                return Err("Agenda snapshot exceeds 4096 open buffers".into());
            }
            open.push(Open {
                path: buffer.file().map(|f| f.path().to_owned()),
                stamp: buffer.stamp(),
                text: Source::new(buffer).snapshot().map_err(|e| e.to_string())?,
                identity: buffer
                    .file()
                    .and_then(|f| f.observation())
                    .map(|o| (o.device, o.inode)),
            });
        }
        for id in &selection.buffers {
            if !buffers.get(*id).is_ok_and(|b| b.is_org()) {
                return Err("Select a live native Org buffer".into());
            }
        }
        selected_identities(&selection, &open)?;
        Ok(Self {
            selection,
            open,
            limits: limits.clone(),
            seen_files: BTreeSet::new(),
            seen_buffers: BTreeSet::new(),
            spent: 0,
            result: SourceSet::default(),
        })
    }
    pub fn run(mut self, cancel: &Cancellation) -> SourceSet {
        let budget = Budget {
            cancel,
            deadline: Instant::now() + Duration::from_secs(self.limits.job_seconds as u64),
        };
        let mut walk = Walk::new(std::mem::take(&mut self.selection.paths));
        let ids = std::mem::take(&mut self.selection.buffers);
        for id in ids {
            if budget.check().is_err() {
                break;
            }
            self.selected_buffer(id);
        }
        loop {
            if let Err(error) = budget.check() {
                self.result.cancelled = cancel.is_cancelled();
                self.result.truncated = !self.result.cancelled;
                self.result.issue(None, error);
                break;
            }
            match walk.next() {
                Ok(Step::End) => break,
                Ok(Step::Progress) => {}
                Ok(Step::Path(path)) => {
                    if !self.path(&path, &budget) {
                        break;
                    }
                }
                Err((path, error)) => {
                    self.result.truncated |= path.is_none();
                    self.result.issue(path, error);
                }
            }
            std::thread::yield_now();
        }
        self.result
    }
    fn selected_buffer(&mut self, id: crate::buffer::BufferId) {
        if let Some(index) = self.open.iter().position(|b| b.stamp.buffer == id) {
            if let Some(identity) = self.open[index].identity {
                self.seen_files.insert(identity);
            }
            self.buffer(index);
        }
    }
    fn path(&mut self, path: &Path, budget: &Budget<'_>) -> bool {
        match self.observe_path(path, budget) {
            Ok(keep_going) => keep_going,
            Err(error) => {
                self.result.issue(Some(path.to_owned()), error);
                true
            }
        }
    }
    fn observe_path(&mut self, path: &Path, budget: &Budget<'_>) -> Result<bool, String> {
        let resolved = crate::file::resolve(path).map_err(|e| e.to_string())?;
        let identity = resolved.observation().map(|o| (o.device, o.inode));
        if identity.is_some_and(|i| self.seen_files.contains(&i)) {
            return Ok(true);
        }
        let matches: Vec<_> = self
            .open
            .iter()
            .enumerate()
            .filter(|(_, b)| b.matches(resolved.path(), identity))
            .map(|(i, _)| i)
            .collect();
        if matches.len() > 1 {
            return Err(
                "Multiple open buffers share this file identity; select one buffer explicitly"
                    .into(),
            );
        }
        if matches
            .first()
            .is_some_and(|i| self.seen_buffers.contains(&self.open[*i].stamp.buffer))
        {
            return Ok(true);
        }
        if !self.admit(Some(path.to_owned())) {
            return Ok(false);
        }
        if let Some(i) = matches.first() {
            self.buffer(*i);
        } else {
            self.disk(&resolved, budget)?;
        }
        if let Some(identity) = identity {
            self.seen_files.insert(identity);
        }
        Ok(true)
    }
    fn disk(
        &mut self,
        resolved: &crate::file::Location,
        budget: &Budget<'_>,
    ) -> Result<(), String> {
        let remaining = self.limits.agenda_total_bytes.saturating_sub(self.spent);
        if resolved
            .observation()
            .is_some_and(|o| o.length > self.limits.agenda_file_bytes.min(remaining) as u64)
        {
            self.result.truncated = true;
            return Err("Agenda byte limit reached".into());
        }
        let (result, used) = read::source(
            resolved.path(),
            self.limits.agenda_file_bytes.min(remaining),
            budget,
        );
        self.result.truncated |= used > self.limits.agenda_file_bytes.min(remaining);
        self.spent = self.spent.saturating_add(used);
        let (text, observed) = result?;
        self.result.bytes += text.len_bytes();
        self.result.documents.push(AgendaSource {
            path: Some(resolved.path().to_owned()),
            version: SourceVersion::Disk(observed),
            text,
        });
        Ok(())
    }
    fn admit(&mut self, path: Option<PathBuf>) -> bool {
        if self.result.documents.len() < self.limits.agenda_files {
            return true;
        }
        self.result.truncated = true;
        self.result
            .issue(path, "Agenda file count limit reached".into());
        false
    }
    fn buffer(&mut self, index: usize) {
        if self.seen_buffers.contains(&self.open[index].stamp.buffer) {
            return;
        }
        if !self.admit(self.open[index].path.clone()) {
            return;
        }
        let open = &self.open[index];
        if !self.seen_buffers.insert(open.stamp.buffer) {
            return;
        }
        let bytes = open.text.len_bytes();
        if bytes > self.limits.agenda_file_bytes
            || bytes > self.limits.agenda_total_bytes.saturating_sub(self.spent)
        {
            self.result.truncated = true;
            self.result.issue(
                open.path.clone(),
                "Agenda byte limit reached for open buffer".into(),
            );
            return;
        }
        self.spent += bytes;
        self.result.bytes += bytes;
        self.result.documents.push(AgendaSource {
            path: open.path.clone(),
            version: SourceVersion::Buffer(open.stamp),
            text: open.text.clone(),
        });
    }
}
fn validate(selection: &SourceSelection, limits: &OrgLimits) -> Result<(), String> {
    if selection.paths.len() + selection.buffers.len() > 4096 {
        return Err("Agenda selection exceeds 4096 roots".into());
    }
    if selection
        .paths
        .iter()
        .any(|p| p.as_os_str().is_empty() || p.as_os_str().len() > 4096)
    {
        return Err("Invalid agenda source path".into());
    }
    if !(1..=4096).contains(&limits.agenda_files)
        || !(1..=33554432).contains(&limits.agenda_file_bytes)
        || !(1..=268435456).contains(&limits.agenda_total_bytes)
        || !(1..=600).contains(&limits.job_seconds)
    {
        return Err("Invalid agenda resource limits".into());
    }
    Ok(())
}

impl Open {
    fn matches(&self, path: &Path, identity: Option<(u64, u64)>) -> bool {
        self.path.as_deref() == Some(path) || identity.is_some() && self.identity == identity
    }
}

fn selected_identities(selection: &SourceSelection, open: &[Open]) -> Result<(), String> {
    let mut identities = BTreeSet::new();
    let mut buffers = BTreeSet::new();
    for source in open
        .iter()
        .filter(|b| selection.buffers.contains(&b.stamp.buffer))
    {
        if buffers.insert(source.stamp.buffer)
            && source.identity.is_some_and(|id| !identities.insert(id))
        {
            return Err(
                "Multiple selected live buffers share a file identity; select one owner".into(),
            );
        }
    }
    Ok(())
}
