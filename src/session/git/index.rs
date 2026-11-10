use super::{Action, Page, Session, Target, comparisons::Mode};
use crate::git::{Effects, IndexDirection, IndexTarget, Outcome, Update};

#[derive(Clone, Copy)]
pub(super) enum Intent {
    Change(IndexDirection),
    Mark,
    Discard,
}

#[derive(Default)]
pub(super) struct Marks {
    pub targets: Vec<IndexTarget>,
    admission: Option<crate::git::budget::Lease>,
}
impl Marks {
    pub fn contains(&self, target: &Target) -> bool {
        self.targets.iter().any(|t| mark_matches(t, target))
    }
    fn duplicate(&self, target: &IndexTarget) -> bool {
        self.targets.iter().any(|t| match (t, target) {
            (IndexTarget::Path(a), IndexTarget::Path(b)) => {
                a.group == b.group && a.change.path == b.change.path
            }
            (IndexTarget::Hunk { hunk: a, .. }, IndexTarget::Hunk { hunk: b, .. }) => a == b,
            _ => false,
        })
    }
}

pub(super) struct Preparing {
    intent: Intent,
    target: Target,
}

impl Session {
    pub(super) fn change_git_index(&mut self, direction: IndexDirection) -> Action {
        if !self.git.marks.targets.is_empty() {
            if !self.git.visible || !matches!(self.git.page, Page::Status | Page::Diff) {
                return self.message("Select Git Status or Diff for a marked index group");
            }
            self.git.state.menu = false;
            return match self
                .git
                .engine
                .change_index_group(direction, &self.git.marks.targets)
            {
                Ok(operation) => {
                    self.git.index_request = Some(operation);
                    self.message("Checking entire marked index group; no partial fallback")
                }
                Err(error) => self.error(&error),
            };
        }
        self.prepare_git_index(Intent::Change(direction))
    }
    pub(super) fn mark_git(&mut self) -> Action {
        self.prepare_git_index(Intent::Mark)
    }
    pub(super) fn unmark_git(&mut self, all: bool) -> Action {
        if !self.git.visible || !matches!(self.git.page, Page::Status | Page::Diff) {
            return self.message("Select Git Status or Diff to change marks");
        }
        if self.git.index_preparing.is_some() {
            return self.message("Wait for the captured comparison before changing marks");
        }
        if all {
            self.git.marks = Marks::default();
        } else if let Some(target) = self.git_index_selection() {
            self.git.marks.targets.retain(|t| !mark_matches(t, &target));
        }
        let message = format!("{} Git targets marked", self.git.marks.targets.len());
        self.message(&message)
    }
    pub(super) fn git_index_selection(&self) -> Option<Target> {
        if self.git.page == Page::Diff {
            self.git.inspection.detail.as_ref().map(|d| {
                self.git.inspection.selected.map_or_else(
                    || Target::Path(d.group, d.change.path.clone()),
                    |id| Target::Hunk(d.group, d.change.path.clone(), id),
                )
            })
        } else {
            self.git.state.selected.clone()
        }
    }
    fn git_index_busy(&self) -> bool {
        self.git.index_request.is_some()
            || self.git.discard_request.is_some()
            || self.git.index_preparing.is_some()
            || self.git.inspection.request.is_some()
    }

    pub(super) fn prepare_git_index(&mut self, intent: Intent) -> Action {
        if !self.git.visible || !matches!(self.git.page, Page::Status | Page::Diff) {
            return self.message("Select an exact path or supported hunk in Git Status first");
        }
        if self.git_index_busy() {
            return self.message("A checked index request is already active");
        }
        let target = self.git_index_selection();
        let Some(target @ (Target::Path(_, _) | Target::Hunk(_, _, _))) = target else {
            return self.message("Select one exact path or supported hunk for this index action");
        };
        self.git.state.menu = false;
        if let Some(diff) = self.git.inspection.detail.as_ref()
            && self.git.engine.detail_current(diff)
            && matches_target(diff, &target)
        {
            return self.finish_git_index_target(intent, &target);
        }
        if matches!(target, Target::Hunk(_, _, _)) {
            return self
                .message("Selected hunk is stale; refresh and select a surviving exact hunk");
        }
        // Capture before async comparison. Navigation cannot substitute another
        // path when preflight completes; leaving Git keeps this started work.
        self.git.index_preparing = Some(Preparing { intent, target });
        let action = self.open_git_diff(Mode::Inline);
        if self.git.inspection.request.is_none() {
            self.git.index_preparing = None;
        }
        action
    }
    fn finish_git_index_target(&mut self, intent: Intent, target: &Target) -> Action {
        let Some(diff) = self
            .git
            .inspection
            .detail
            .as_ref()
            .filter(|d| matches_target(d, target))
            .cloned()
        else {
            return self.message("Captured index path/layer did not survive comparison");
        };
        let target = match target {
            Target::Path(_, _) => IndexTarget::Path(diff),
            Target::Hunk(_, _, hunk) => IndexTarget::Hunk { diff, hunk: *hunk },
            Target::Section(_) => return self.message("Index action requires a path or hunk"),
        };
        match intent {
            Intent::Discard => self.discard_git_index_target(target),
            Intent::Mark => self.mark_git_index_target(target),
            Intent::Change(direction) => self.submit_git_index_target(direction, target),
        }
    }

    fn discard_git_index_target(&mut self, target: IndexTarget) -> Action {
        match target {
            IndexTarget::Path(diff) => self.confirm_git_discard(diff),
            _ => self.message("Discard requires one whole tracked path; hunks are unsupported"),
        }
    }

    fn mark_git_index_target(&mut self, target: IndexTarget) -> Action {
        if !self.git.engine.detail_current(target.diff()) {
            return self.message("Cannot mark a stale or incomplete comparison");
        }
        // Validate duplicate membership against the captured target rather
        // than the current cursor, which can move during async comparison.
        let duplicate = self.git.marks.duplicate(&target);
        if duplicate || self.git.marks.targets.len() == 256 {
            return self.message("Git mark is duplicate or exceeds the 256-target bound");
        }
        if self.git.marks.admission.is_none() {
            match self.git.engine.index_marks_admission() {
                Ok(admission) => {
                    self.git.marks.admission = Some(admission);
                    self.git.marks.targets = Vec::with_capacity(256);
                }
                Err(error) => return self.error(&error),
            }
        }
        self.git.marks.targets.push(target);
        let message = format!("{} Git targets marked", self.git.marks.targets.len());
        self.message(&message)
    }

    fn submit_git_index_target(
        &mut self,
        direction: IndexDirection,
        target: IndexTarget,
    ) -> Action {
        match self.git.engine.change_index(direction, target) {
            Ok(operation) => {
                self.git.index_request = Some(operation);
                self.message(
                    "Checking exact index request; working-tree files and source buffers retained",
                )
            }
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn finish_git_index_preparation(&mut self, update: &Update) {
        let Some(preparing) = self.git.index_preparing.take() else {
            return;
        };
        if update.published {
            self.finish_git_index_target(preparing.intent, &preparing.target);
        }
    }
    pub(super) fn publish_git_index(&mut self, update: &Update) {
        self.git.index_request = None;
        if update.published {
            self.git.state.context = update.context;
            self.git.reconcile(self.git_extent());
        }
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
            self.message(
                "Index updated and verified; working-tree files and source buffers retained",
            );
        } else {
            let text = format!(
                "Index action {:?}; effects {:?}; inspect git-processes",
                record.outcome, record.effects
            );
            self.message(&text);
        }
    }
}

fn matches_target(diff: &crate::git::Diff, target: &Target) -> bool {
    match target {
        Target::Path(group, path) => diff.group == *group && diff.change.path == *path,
        Target::Hunk(group, path, id) => {
            diff.group == *group && diff.change.path == *path && diff.hunk(*id).is_some()
        }
        Target::Section(_) => false,
    }
}

fn mark_matches(t: &IndexTarget, target: &Target) -> bool {
    match (t, target) {
        (IndexTarget::Path(diff), Target::Path(group, path)) => {
            diff.group == *group && diff.change.path == *path
        }
        (IndexTarget::Hunk { diff, hunk }, Target::Hunk(group, path, id)) => {
            diff.group == *group && diff.change.path == *path && hunk == id
        }
        _ => false,
    }
}
