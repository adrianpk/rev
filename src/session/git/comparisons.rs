use super::{Action, Page, Session, Target, Work};
use crate::{
    display::git::label,
    git::{Completeness, Diff, DiffLine, Group, HunkId, OperationId, Update},
};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Inline,
    Detail,
}

pub(super) struct Request {
    pub operation: OperationId,
    mode: Mode,
}

pub(super) struct Resume {
    group: Group,
    path: PathBuf,
    mode: Mode,
}

#[derive(Default)]
pub(super) struct Inspection {
    pub detail: Option<Arc<Diff>>,
    pub request: Option<Request>,
    pub after_refresh: Option<Resume>,
    pub expanded: bool,
    pub expanded_hunk: Option<HunkId>,
    pub top: usize,
    pub selected: Option<HunkId>,
}

impl Inspection {
    pub fn return_to_status(&mut self) {
        if let Some(request) = &mut self.request {
            request.mode = Mode::Inline;
        }
        if let Some(resume) = &mut self.after_refresh {
            resume.mode = Mode::Inline;
        }
    }
    pub fn resume(&self, page: Page) -> Option<Resume> {
        let mode = match page {
            Page::Diff => Mode::Detail,
            Page::Status if self.expanded => Mode::Inline,
            _ => return None,
        };
        let diff = self.detail.as_ref()?;
        Some(Resume {
            group: diff.group,
            path: diff.change.path.clone(),
            mode,
        })
    }
}

// Labels escape controls and invalid UTF-8, while exact bytes remain in Diff.
// Only visible worker-indexed chunks are converted into presentation strings.
pub(super) fn line_label(diff: &Diff, line: &DiffLine) -> String {
    label(
        diff.evidence[line.bytes.clone()]
            .strip_suffix(b"\n")
            .unwrap_or(&diff.evidence[line.bytes.clone()]),
    )
}

impl Work {
    pub(super) fn diff_status(&self) -> String {
        let Some(diff) = self.inspection.detail.as_ref() else {
            return "Checking comparison".into();
        };
        format!(
            "{}{:?} / {:?}{}{}",
            if self.engine.detail_current(diff) {
                ""
            } else {
                "STALE / unverified: "
            },
            diff.kind,
            diff.completeness,
            if diff.binary { " / binary" } else { "" },
            if self.inspection.request.is_some() {
                " / checking comparison"
            } else {
                ""
            },
        )
    }
    pub(super) fn diff_header(&self, row: usize) -> String {
        let Some(diff) = self.inspection.detail.as_ref() else {
            return if row == 0 {
                "Git Diff: checking comparison".into()
            } else {
                String::new()
            };
        };
        match row {
            0 => format!(
                "Git Diff: {} — {}",
                diff.group.label(),
                label(diff.change.path.as_os_str().as_encoded_bytes())
            ),
            1 => format!(
                "Baseline: {:?}; {} semantic hunks; read-only{}",
                diff.baseline,
                diff.hunks.len(),
                if diff.completeness != Completeness::Complete {
                    "; TRUNCATED detail"
                } else {
                    ""
                }
            ),
            2 => self.diff_status(),
            _ => String::new(),
        }
    }
}

impl Session {
    pub(super) fn open_git_diff(&mut self, mode: Mode) -> Action {
        if !self.git.visible || !matches!(self.git.page, Page::Status | Page::Diff) {
            return self.message("Select an exact path or hunk in Git Status first");
        }
        self.git.state.menu = false;
        let target = if self.git.page == Page::Diff {
            self.git
                .inspection
                .detail
                .as_ref()
                .map(|d| (d.group, d.change.path.clone()))
        } else {
            match self.git.state.selected.as_ref() {
                Some(Target::Path(group, path) | Target::Hunk(group, path, _)) => {
                    Some((*group, path.clone()))
                }
                _ => None,
            }
        };
        let Some((group, path)) = target else {
            return self.message("Select an exact path or hunk in Git Status first");
        };
        if self.git.inspection.detail.as_ref().is_some_and(|d| {
            d.group == group && d.change.path == path && self.git.engine.detail_current(d)
        }) {
            self.reveal_git_diff(mode);
            return Action::Redraw;
        }
        self.request_git_diff(Resume { group, path, mode })
    }
    pub(super) fn request_git_diff(&mut self, target: Resume) -> Action {
        let Some(observation) = self.git.observation() else {
            return self.message("Git Status is still checking context");
        };
        match self.git.engine.compare(
            observation.context,
            observation.generation,
            target.group,
            &target.path,
        ) {
            Ok(operation) => {
                self.git.inspection.request = Some(Request {
                    operation,
                    mode: target.mode,
                });
                self.message("Checking exact Git comparison")
            }
            Err(error) => self.error(&format!("Git comparison unavailable: {error}")),
        }
    }
    fn reveal_git_diff(&mut self, mode: Mode) {
        self.git.inspection.expanded = true;
        if let Mode::Detail = mode {
            let selected = match self.git.state.selected {
                Some(Target::Hunk(_, _, id)) => Some(id),
                _ => self.git.inspection.selected,
            };
            self.git.inspection.selected = selected;
            self.git.inspection.top = self
                .git
                .inspection
                .detail
                .as_ref()
                .and_then(|d| {
                    d.lines
                        .iter()
                        .position(|line| selected.is_some() && line.hunk == selected)
                })
                .unwrap_or(0);
            self.git.page = Page::Diff;
        }
        self.git.reconcile(self.git_extent());
    }
    pub(super) fn publish_git_diff(&mut self, update: &Update) {
        let request = self
            .git
            .inspection
            .request
            .take()
            .expect("matching comparison request");
        let replacement = update
            .context
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.detail.as_ref())
            .filter(|d| d.id == update.operation && update.published)
            .cloned();
        if let Some(diff) = replacement {
            // Selection survives only when the same path/layer and exact patch
            // unit survive. A display row cannot transfer hunk authority.
            let map = |id: Option<HunkId>| -> Option<HunkId> {
                let old = self.git.inspection.detail.as_ref()?;
                if old.group != diff.group || old.change.path != diff.change.path {
                    return None;
                }
                let hunk = old.hunk(id?)?;
                diff.hunks
                    .iter()
                    .find(|h| {
                        h.old == hunk.old
                            && h.new == hunk.new
                            && diff.evidence[h.patch.clone()] == old.evidence[hunk.patch.clone()]
                    })
                    .map(|h| h.id)
            };
            let selected = map(self.git.inspection.selected);
            let expanded_hunk = map(self.git.inspection.expanded_hunk);
            if let Some(Target::Hunk(group, path, id)) = self.git.state.selected.as_ref() {
                self.git.state.selected = Some(map(Some(*id)).map_or_else(
                    || Target::Path(*group, path.clone()),
                    |id| Target::Hunk(*group, path.clone(), id),
                ));
            }
            self.git.inspection.selected = selected;
            self.git.inspection.expanded_hunk = expanded_hunk;
            self.git.inspection.detail = Some(diff);
            // A completion after q updates retained data without reopening a view.
            if self.git.visible
                && matches!(self.git.page, Page::Status | Page::Diff)
                && self.git.observation().is_some_and(|o| {
                    self.git
                        .inspection
                        .detail
                        .as_ref()
                        .is_some_and(|d| o.context == d.observation.context)
                })
            {
                self.reveal_git_diff(request.mode);
            }
            if self.git.visible
                && self.input.focus() == crate::input::Focus::Editing
                && self.pending.is_none()
            {
                self.message("Git comparison retained as read-only detail");
            }
        } else if self.git.visible
            && self.input.focus() == crate::input::Focus::Editing
            && self.pending.is_none()
        {
            self.message("Git comparison failed; last inspection retained, g refreshes Status");
        }
    }
    pub(super) fn move_git_diff(&mut self, distance: usize, backward: bool) -> Action {
        let count = self
            .git
            .inspection
            .detail
            .as_ref()
            .map_or(0, |d| d.lines.len());
        let index = if backward {
            self.git.inspection.top.saturating_sub(distance)
        } else {
            self.git
                .inspection
                .top
                .saturating_add(distance)
                .min(count.saturating_sub(self.git_extent()))
        };
        self.git.inspection.top = index;
        self.select_git_diff(index)
    }
    pub(super) fn select_git_diff(&mut self, index: usize) -> Action {
        self.git.inspection.selected = self
            .git
            .inspection
            .detail
            .as_ref()
            .and_then(|d| d.lines.get(index))
            .and_then(|line| line.hunk);
        Action::Redraw
    }
    pub(super) fn visit_git_file(&mut self) -> Action {
        if matches!(self.git.page, Page::Processes | Page::Process(_)) && self.git.visible {
            return self.show_git_process();
        }
        if !self.git.visible {
            return self.message("Select a current Git path first");
        }
        let path = if self.git.page == Page::Diff {
            self.git
                .inspection
                .detail
                .as_ref()
                .map(|d| d.change.path.clone())
        } else {
            match self.git.state.selected.as_ref() {
                Some(Target::Path(_, path) | Target::Hunk(_, path, _)) => Some(path.clone()),
                _ => None,
            }
        };
        let Some(path) = path else {
            return self.message("Select a current Git path first");
        };
        // Detail keeps its captured root even when a later repository entry
        // changes the current context. Labels are never used for source ingress.
        let observation = if self.git.page == Page::Diff {
            self.git
                .inspection
                .detail
                .as_ref()
                .map(|d| d.observation.as_ref())
        } else {
            self.git.observation()
        };
        let Some(root) = observation.and_then(|o| o.identity.worktree.as_ref()) else {
            return self.message("No current working-tree source exists");
        };
        let path = root.join(path);
        match std::fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => {}
            _ => return self.message("Selected Git path has no current source file"),
        }
        if let Err(error) = self.load_file(&path) {
            return self.error(&error.to_string());
        }
        self.git.visible = false;
        self.git.painted = None;
        self.input.set_git(false);
        self.terminal_source_focus();
        self.sync_input_context();
        Action::Redraw
    }
}
