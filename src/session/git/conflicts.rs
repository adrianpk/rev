use super::*;
use crate::git::{Conflict, ConflictRole, Update};

pub(super) struct Inspection {
    pub request: Option<OperationId>,
    pub preparing: Option<OperationId>,
    pub explaining: Option<OperationId>,
    pub resolving: Option<OperationId>,
    pub detail: Option<Arc<Conflict>>,
    pub selected: Option<PathBuf>,
    pub top: usize,
    _admission: crate::git::budget::Lease,
}
impl Inspection {
    pub(super) fn new(engine: &Repositories) -> std::io::Result<Self> {
        Ok(Self {
            request: None,
            preparing: None,
            explaining: None,
            resolving: None,
            detail: None,
            selected: None,
            top: 0,
            _admission: engine
                .source_admission(8192)
                .map_err(std::io::Error::other)?,
        })
    }
}
impl Work {
    pub(super) fn conflict_path(&self) -> Option<&std::path::Path> {
        let sequence = self
            .state
            .context
            .and_then(|id| self.engine.context(id))?
            .sequence
            .as_ref()?;
        self.conflicts
            .selected
            .as_deref()
            .filter(|p| sequence.conflicts.iter().any(|c| c == p))
            .or_else(|| sequence.conflicts.first().map(PathBuf::as_path))
    }
    fn conflict_lines(&self, width: usize) -> impl Iterator<Item = String> + '_ {
        let detail = self.conflicts.detail.as_ref();
        let header = (0..5).map(move |row| {
            let Some(c) = detail else { return if row == 0 { "Git Conflict: checking exact index sides".into() } else { String::new() }; };
            match row {
                0 => format!("Git Conflict: {}", label(c.change.path.as_os_str().as_encoded_bytes())),
                1 => format!("Kind: {} ({}/{}) / {:?}", c.kind(), char::from(c.change.index), char::from(c.change.worktree), c.completeness),
                2 => format!("Actual operation: {:?}; index stages retain their exact Git meanings", c.sequence.family),
                3 => "Read-only captured sides; Return visits the canonical editable saved-result file".into(),
                _ => "Editing or removing markers never stages or continues the sequence".into(),
            }
        });
        let sides = ConflictRole::ALL.into_iter().flat_map(move |role| {
            let side = detail.and_then(|c| c.side(role));
            (0..side.map_or(2, |s| 3 + if s.text { s.lines.len() } else { 0 })).map(move |row| {
                if row == 0 { return format!("{} / index stage {} / {}", role.label(), role.stage(), role.meaning(detail.and_then(|c| c.sequence.family))); }
                let Some(s) = side else { return "Missing stage (deletion or structural conflict); no bytes inferred".into(); };
                match row {
                    1 => format!("Object {} / mode {:06o}", s.object.hex(), s.mode),
                    2 => format!("{} captured bytes / {:?} / {}", s.bytes.len(), s.completeness, if s.binary { "binary; text actions unavailable" } else if s.text { "complete UTF-8 text" } else { "unsupported mode, encoding or partial content; text actions unavailable" }),
                    _ => label(&s.bytes[s.lines[row-3].clone()]),
                }
            })
        });
        header.chain(sides).flat_map(move |text| Wrapped {
            text,
            offset: Some(0),
            width: width.max(1),
        })
    }
    pub(super) fn conflict_row(&self, row: usize, width: usize) -> (String, bool) {
        (
            self.conflict_lines(width)
                .nth(self.conflicts.top + row)
                .unwrap_or_default(),
            false,
        )
    }
}
impl Session {
    pub(super) fn open_git_conflict(&mut self) -> Action {
        if self.git.visible && self.git.page == Page::Conflict {
            return self.visit_git_conflict_result();
        }
        if !self.git.visible || self.git.page != Page::Sequence {
            return self
                .message("Open git-conflicts and select one exact conflict before inspection");
        }
        let Some(sequence) = self
            .git
            .state
            .context
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.sequence.clone())
        else {
            return self.message("Git-owned sequence state is still checking");
        };
        let Some(path) = self.git.conflict_path().map(std::path::Path::to_path_buf) else {
            return self.message("No exact unmerged path is available");
        };
        match self.git.engine.inspect_conflict(sequence, &path) {
            Ok(id) => {
                self.git.conflicts.request = Some(id);
                self.git.painted = None;
                self.message(
                    "Checking exact conflict stages, operation roles and bounded side content",
                )
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn publish_git_conflict(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.conflicts.request = None;
        if update.published
            && let Some(detail) = update
                .context
                .and_then(|id| self.git.engine.context(id))
                .and_then(|c| c.conflict.clone())
        {
            self.reveal_git_conflict(detail);
        } else if self.git.visible && self.pending.is_none() && self.input.focus() == Focus::Editing
        {
            self.message(
                "Conflict inspection failed or became stale; refresh actual Git sequence state",
            );
        }
        self.git.painted = None;
    }
    fn reveal_git_conflict(&mut self, detail: Arc<Conflict>) {
        self.git.conflicts.detail = Some(detail);
        self.git.conflicts.top = 0;
        if self.git.visible
            && self.git.page == Page::Sequence
            && self.pending.is_none()
            && self.input.focus() == Focus::Editing
        {
            self.git.page = Page::Conflict;
            self.message("Checked conflict sides retained as read-only inspection; Return visits editable result");
        }
    }
    pub(super) fn move_git_conflict(&mut self, distance: usize, backward: bool) -> Action {
        let count = self
            .git
            .conflict_lines(usize::from(self.size.width))
            .count();
        self.git.conflicts.top = if backward {
            self.git.conflicts.top.saturating_sub(distance)
        } else {
            self.git
                .conflicts
                .top
                .saturating_add(distance)
                .min(count.saturating_sub(self.git_extent()))
        };
        self.git.painted = None;
        Action::Redraw
    }
    pub(super) fn visit_git_conflict_result(&mut self) -> Action {
        let Some(detail) = self
            .git
            .conflicts
            .detail
            .as_ref()
            .filter(|_| self.git.visible && self.git.page == Page::Conflict)
        else {
            return self.message("Inspect one exact conflict first");
        };
        let Some(root) = detail.sequence.refs.observation.identity.worktree.as_ref() else {
            return self.message("Conflict has no working-tree result");
        };
        let path = root.join(&detail.change.path);
        let Ok(actual) = std::fs::canonicalize(&path) else {
            return self.message("Conflict result is missing; no file is created by inspection");
        };
        if !actual.starts_with(root)
            || !std::fs::symlink_metadata(&path)
                .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
        {
            return self
                .message("Conflict result is not a canonical ordinary file inside this worktree");
        }
        if let Err(e) = self.load_file(&actual) {
            return self.error(&e.to_string());
        }
        self.git.visible = false;
        self.git.painted = None;
        self.input.set_git(false);
        self.terminal_source_focus();
        self.sync_input_context();
        Action::Redraw
    }
}

impl Session {
    pub(super) fn prepare_git_resolution(
        &mut self,
        action: crate::git::ResolutionAction,
    ) -> Action {
        let Some(conflict) = self
            .git
            .conflicts
            .detail
            .clone()
            .filter(|_| self.git.visible && self.git.page == Page::Conflict)
        else {
            return self.message("Inspect one exact current conflict before requesting resolution");
        };
        if let Err(e) = self.git_resolution_sources(&conflict) {
            return self.error(&e);
        }
        match self.git.engine.prepare_resolution(conflict, action) {
            Ok(id) => {
                self.git.conflicts.preparing = Some(id);
                self.git.painted = None;
                self.message("Preparing checked conflict resolution; no change has started")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn publish_git_resolution_preparation(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.conflicts.preparing = None;
        if !self.git.visible
            || self.git.page != Page::Conflict
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let Some(plan) = update
            .context
            .filter(|_| update.published)
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.resolution.clone())
        else {
            self.message("Conflict resolution preparation failed; refresh the actual sequence and saved result");
            return;
        };
        let sources = match self.git_resolution_sources(&plan.conflict) {
            Ok(s) => s,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        let c = &plan.conflict;
        let review = resolution_review(&plan);
        self.confirm_git_ref(
            refs::Proposal {
                refs: std::sync::Arc::clone(&c.sequence.refs),
                action: refs::RefAction::Resolution(std::sync::Arc::clone(&plan)),
                sources: Some(sources),
            },
            format!(
                "Confirm {} for this exact conflict? y=accept, n=cancel",
                plan.action.label()
            ),
            review,
        );
    }
    pub(super) fn publish_git_resolution(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.conflicts.resolving = None;
        self.git.painted = None;
        if update.published {
            self.git.state.context = update.context;
        }
        self.observe_files();
        if self.git.visible && self.pending.is_none() && self.input.focus() == Focus::Editing {
            self.git.page = Page::Sequence;
            self.git.refs.sequence_top = 0;
            if update.published
                && self
                    .git
                    .engine
                    .record(update.operation)
                    .is_some_and(|r| r.outcome == crate::git::Outcome::Verified)
            {
                self.message("Conflict action verified against exact index and disk; continuing remains separate");
            } else {
                self.message("Conflict action failed or effects remain uncertain; inspect git-processes and refresh actual sequence state");
            }
        }
    }
}

impl Session {
    pub(super) fn explain_git_conflict(&mut self) -> Action {
        let Some(conflict) = self
            .git
            .conflicts
            .detail
            .clone()
            .filter(|_| self.git.visible && self.git.page == Page::Conflict)
        else {
            return self
                .message("Inspect one exact conflict before requesting its optional explanation");
        };
        match self.git.engine.explain_conflict(conflict) {
            Ok(id) => {
                self.git.conflicts.explaining = Some(id);
                self.message(
                    "Capturing checked conflict context for optional read-only explanation",
                )
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn publish_git_explanation(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.conflicts.explaining = None;
        if !self.git.visible
            || self.git.page != Page::Conflict
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let Some(capture) = update
            .context
            .filter(|_| update.published)
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.conflict_explanation.clone())
        else {
            self.message("Conflict explanation capture failed or became stale; manual inspection and resolution remain available");
            return;
        };
        if let Err(e) = self.ai_explain_conflict(capture) {
            self.error(&e);
        }
    }
    pub(in crate::session) fn git_explanation_origin(
        &self,
        capture: &crate::git::ConflictExplanation,
    ) -> bool {
        !self.git.visible
            || (self.git.page == Page::Conflict
                && self
                    .git
                    .conflicts
                    .detail
                    .as_ref()
                    .is_some_and(|c| Arc::ptr_eq(c, &capture.conflict)))
    }
    pub(in crate::session) fn release_git_explanation(&mut self) {
        self.git.visible = false;
        self.git.painted = None;
        self.input.set_git(false);
        self.sync_input_context();
    }
}

fn resolution_review(plan: &crate::git::Resolution) -> String {
    let c = &plan.conflict;
    let effect = match plan.action {
        crate::git::ResolutionAction::Use(_) => {
            "Replace only this working file with the complete selected side under Git checkout conversion; preserve all index stages."
        }
        crate::git::ResolutionAction::Delete => {
            "Remove only this exact result path from disk and index, staging its deletion; preserve every other index entry."
        }
        crate::git::ResolutionAction::Stage => {
            "Stage only the captured saved bytes or saved absence; preserve disk and every other index entry."
        }
    };
    let mut review = format!(
        "Checked conflict resolution review\nAction: {}\nExact path: {}\nKind: {}\nActual family: {:?}\nChecked HEAD:\n{}\nCompatible mode: {:06o}\nSaved bytes: {}\nExpected staged object: {}\n{}\n",
        plan.action.label(),
        label(c.change.path.as_os_str().as_encoded_bytes()),
        c.kind(),
        c.sequence.family,
        c.sequence.refs.head.map_or("unborn".into(), |id| id.hex()),
        plan.mode,
        plan.saved
            .as_ref()
            .map_or("absent".into(), |s| s.bytes.len().to_string()),
        plan.expected_object
            .map_or("none / index unchanged or deletion".into(), |id| id.hex()),
        effect
    );
    for role in ConflictRole::ALL {
        if let Some(side) = c.side(role) {
            review.push_str(&format!(
                "{} / stage {} / {}\n{} mode {:06o}\n",
                role.label(),
                role.stage(),
                role.meaning(c.sequence.family),
                side.object.hex(),
                side.mode
            ));
        }
    }
    review.push_str("Saved file identity, bytes, stages, operation metadata, HEAD and visiting source owners must remain unchanged. Marker text grants no authority. Continuing remains a separate explicit action. y / C-Enter / C-c C-c accept; n / C-g cancel.");
    review
}
