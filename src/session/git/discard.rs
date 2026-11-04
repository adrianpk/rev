use super::super::Pending;
use super::{Action, Page, Session, Target};
use crate::{
    buffer::{Buffer, Stamp},
    git::{Diff, Effects, Outcome, Update},
};
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::Arc,
};

pub(in crate::session) struct Consent {
    diff: Arc<Diff>,
    sources: Sources,
    pub message: String,
}
struct Sources {
    path: PathBuf,
    canonical: PathBuf,
    destination: Option<fs::Metadata>,
    referent: Option<fs::Metadata>,
    stamps: Vec<Stamp>,
    _admission: crate::git::budget::Lease,
}

impl Sources {
    fn related(&self, buffer: &Buffer) -> bool {
        buffer.file().is_some_and(|f| {
            f.path() == self.path
                || f.path() == self.canonical
                || f.observation()
                    .zip(self.referent.as_ref())
                    .is_some_and(|(a, b)| a.device == b.dev() && a.inode == b.ino())
        })
    }
    fn current(&self, session: &Session) -> Result<(), String> {
        if canonical(&self.path)? != self.canonical
            || !same_metadata(self.destination.as_ref(), metadata(&self.path)?.as_ref())
        {
            return Err("Discard destination changed while confirmation was open".into());
        }
        let mut expected = self.stamps.iter();
        for buffer in session.buffers.iter().filter(|b| self.related(b)) {
            if buffer.is_modified() || expected.next() != Some(&buffer.stamp()) {
                return Err(
                    "Discard source changed or has unsaved text; confirmation invalidated".into(),
                );
            }
        }
        if expected.next().is_some() {
            return Err("Discard source owners changed; confirmation invalidated".into());
        }
        Ok(())
    }
}

impl Session {
    pub(in crate::session) fn cancel_git_discard(&mut self) -> Option<Action> {
        if !matches!(self.pending, Some(Pending::GitDiscard(_))) {
            return None;
        }
        self.pending = None;
        self.input.cancel();
        Some(self.message("Discard canceled; source buffers and working files retained"))
    }
    pub(super) fn discard_git(&mut self) -> Action {
        if !self.git.marks.targets.is_empty() {
            return self.message("Discard requires one whole path; clear marked groups first");
        }
        if !matches!(self.git_index_selection(), Some(Target::Path(_, _))) {
            return self.message(
                "Discard requires one whole tracked path; hunks and sections are unsupported",
            );
        }
        self.prepare_git_index(super::index::Intent::Discard)
    }
    pub(super) fn confirm_git_discard(&mut self, diff: Arc<Diff>) -> Action {
        if !self.git.visible
            || !matches!(self.git.page, Page::Status | Page::Diff)
            || self.input.focus() != crate::input::Focus::Editing
            || self.pending.is_some()
        {
            return self
                .message("Discard comparison retained; request confirmation again from Git");
        }
        let result = (|| {
            self.git.engine.check_discard(&diff)?;
            let path = diff
                .observation
                .identity
                .worktree
                .as_ref()
                .ok_or("No working tree")?
                .join(&diff.change.path);
            let canonical = canonical(&path)?;
            let destination = metadata(&path)?;
            let referent = match fs::metadata(&path) {
                Ok(m) => Some(m),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.to_string()),
            };
            let mut sources = Sources {
                path,
                canonical,
                destination,
                referent,
                stamps: Vec::new(),
                _admission: self.git.engine.source_admission(0)?,
            };
            let count = self.buffers.iter().filter(|b| sources.related(b)).count();
            let bytes = count
                .checked_mul(std::mem::size_of::<Stamp>())
                .and_then(|n| n.checked_add(sources.path.capacity()))
                .and_then(|n| n.checked_add(sources.canonical.capacity()))
                .and_then(|n| n.checked_add(24 * diff.change.path.as_os_str().len() + 4096))
                .ok_or("Discard source/confirmation admission overflow")?;
            sources._admission.grow(bytes)?;
            sources.stamps = Vec::with_capacity(count);
            for buffer in self.buffers.iter() {
                if !sources.related(buffer) {
                    continue;
                }
                if buffer.is_modified() {
                    return Err(
                        "Discard rejected: modified visiting buffer has unsaved text".into(),
                    );
                }
                sources.stamps.push(buffer.stamp());
            }
            Ok(sources)
        })();
        let sources = match result {
            Ok(s) => s,
            Err(e) => return self.error(&e),
        };
        let message = format!(
            "Discard working-tree changes in {} from its index? y=discard, n=retain, C-g=cancel",
            crate::display::git::label(diff.change.path.as_os_str().as_encoded_bytes())
        );
        if let Err(e) = self
            .input
            .decision("Discard working-tree path from index? [y/n]")
        {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::GitDiscard(Box::new(Consent {
            diff,
            sources,
            message: message.clone(),
        })));
        self.message(&message)
    }
    pub(in crate::session) fn git_discard_answer(
        &mut self,
        consent: Consent,
        accepted: bool,
    ) -> Action {
        self.input.cancel();
        if !accepted {
            return self.message("Discard canceled; working files retained");
        }
        if let Err(e) = consent.sources.current(self) {
            return self.error(&e);
        }
        if self.git.state.context != Some(consent.diff.observation.context)
            || !self.git.marks.targets.is_empty()
        {
            return self
                .message("Discard context or selection changed; request confirmation again");
        }
        match self.git.engine.discard(consent.diff) {
            Ok(operation) => {
                self.git.discard_request = Some(operation);
                self.message(
                    "Checking confirmed working-tree discard; source buffers remain protected",
                )
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn publish_git_discard(&mut self, update: &Update) {
        self.git.discard_request = None;
        if update.published {
            self.git.state.context = update.context;
            self.git.reconcile(self.git_extent());
        }
        // Disk changes enter the existing observation/automatic-or-ask reload
        // owner; this command never publishes a replacement source buffer.
        self.observe_files();
        if !self.git.visible
            || self.input.focus() != crate::input::Focus::Editing
            || self.pending.is_some()
        {
            return;
        }
        let Some(record) = self.git.engine.record(update.operation) else {
            return;
        };
        if record.outcome == Outcome::Verified && record.effects == Effects::Verified {
            self.message("Working path restored and verified; index entries unchanged; source reload follows file policy");
        } else {
            self.message(&format!(
                "Discard {:?}; effects {:?}; inspect git-processes and actual files",
                record.outcome, record.effects
            ));
        }
    }
}
fn canonical(path: &Path) -> Result<PathBuf, String> {
    match fs::canonicalize(path) {
        Ok(p) => Ok(p),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => crate::file::resolve(path)
            .map(|f| f.path().to_owned())
            .map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
fn metadata(path: &Path) -> Result<Option<fs::Metadata>, String> {
    match fs::symlink_metadata(path) {
        Ok(m) => Ok(Some(m)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
fn same_metadata(a: Option<&fs::Metadata>, b: Option<&fs::Metadata>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.dev() == b.dev()
                && a.ino() == b.ino()
                && a.len() == b.len()
                && a.mode() == b.mode()
                && a.mtime() == b.mtime()
                && a.mtime_nsec() == b.mtime_nsec()
                && a.ctime() == b.ctime()
                && a.ctime_nsec() == b.ctime_nsec()
        }
        _ => false,
    }
}
