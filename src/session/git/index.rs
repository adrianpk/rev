use super::{Action, Page, Session, Target, comparisons::Mode};
use crate::git::{Effects, IndexDirection, IndexTarget, Outcome, Update};

pub(super) struct Preparing {
    direction: IndexDirection,
    target: Target,
}

impl Session {
    pub(super) fn change_git_index(&mut self, direction: IndexDirection) -> Action {
        if !self.git.visible || !matches!(self.git.page, Page::Status | Page::Diff) {
            return self.message("Select an exact path or supported hunk in Git Status first");
        }
        if self.git.index_request.is_some()
            || self.git.index_preparing.is_some()
            || self.git.inspection.request.is_some()
        {
            return self.message("A checked index request is already active");
        }
        let target = if self.git.page == Page::Diff {
            self.git.inspection.detail.as_ref().map(|d| {
                self.git.inspection.selected.map_or_else(
                    || Target::Path(d.group, d.change.path.clone()),
                    |id| Target::Hunk(d.group, d.change.path.clone(), id),
                )
            })
        } else {
            self.git.state.selected.clone()
        };
        let Some(target @ (Target::Path(_, _) | Target::Hunk(_, _, _))) = target else {
            return self.message("Select one exact path or supported hunk for this index action");
        };
        self.git.state.menu = false;
        if let Some(diff) = self.git.inspection.detail.as_ref()
            && self.git.engine.detail_current(diff)
            && matches_target(diff, &target)
        {
            return self.launch_git_index(direction, &target);
        }
        if matches!(target, Target::Hunk(_, _, _)) {
            return self
                .message("Selected hunk is stale; refresh and select a surviving exact hunk");
        }
        // Capture before async comparison. Navigation cannot substitute another
        // path when preflight completes; leaving Git keeps this started work.
        self.git.index_preparing = Some(Preparing { direction, target });
        let action = self.open_git_diff(Mode::Inline);
        if self.git.inspection.request.is_none() {
            self.git.index_preparing = None;
        }
        action
    }
    fn launch_git_index(&mut self, direction: IndexDirection, target: &Target) -> Action {
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
            self.launch_git_index(preparing.direction, &preparing.target);
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
