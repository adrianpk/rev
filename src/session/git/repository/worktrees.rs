use super::*;
use crate::git::{WorktreeAction, Worktrees};

pub(in crate::session::git) struct Inspection {
    pub request: Option<OperationId>,
    pub preparing: Option<OperationId>,
    selected: Option<PathBuf>,
    pub top: usize,
    detail: Option<usize>,
    failed: bool,
}
impl Inspection {
    pub(in crate::session::git) fn new() -> Self {
        Self {
            request: None,
            preparing: None,
            selected: None,
            top: 0,
            detail: None,
            failed: false,
        }
    }
}
#[derive(Clone, Copy)]
pub(in crate::session) enum Intent {
    Create,
    Lock,
    Unlock,
    Prune,
    Remove,
}
enum Step {
    Destination,
    Branch(PathBuf),
    Reason,
}
pub(in crate::session) struct Prompt {
    worktrees: Arc<Worktrees>,
    selected: Option<PathBuf>,
    step: Step,
}

impl Work {
    pub(in crate::session::git) fn worktrees(&self) -> Option<&Arc<Worktrees>> {
        self.state
            .context
            .and_then(|c| self.engine.context(c))?
            .worktrees
            .as_ref()
    }
    pub(in crate::session::git) fn worktree_token(&self) -> (Option<OperationId>, usize) {
        (
            self.worktrees().map(|w| w.id),
            self.worktrees.detail.unwrap_or(self.worktrees.top),
        )
    }
    fn worktree_detail_lines(&self, width: usize) -> impl Iterator<Item = String> + '_ {
        let list = self.worktrees();
        let entry = self
            .worktrees
            .selected
            .as_ref()
            .and_then(|p| list?.entry(p));
        (0..12)
            .map(move |row| {
                let Some(e) = entry else {
                    return "Worktree detail unavailable; refresh the list".into();
                };
                match row {
                    0 => "Git Worktree: exact captured identity".into(),
                    1 => format!(
                        "Working path: {}",
                        label(e.path.as_os_str().as_encoded_bytes())
                    ),
                    2 => format!(
                        "Per-worktree Git directory: {}",
                        label(e.git_directory.as_os_str().as_encoded_bytes())
                    ),
                    3 => format!(
                        "Common directory: {}",
                        list.map_or(String::new(), |w| label(
                            w.observation
                                .identity
                                .common_directory
                                .as_os_str()
                                .as_encoded_bytes()
                        ))
                    ),
                    4 => format!(
                        "Full HEAD: {}",
                        e.head.map_or("unborn / unavailable".into(), |h| h.hex())
                    ),
                    5 => format!("Branch: {}", label(&e.branch)),
                    6 => format!(
                        "Bare: {} / Detached: {} / Available: {}",
                        e.bare, e.detached, e.available
                    ),
                    7 => format!(
                        "Lock: {}",
                        e.locked.as_ref().map_or("unlocked".into(), |r| label(r))
                    ),
                    8 => format!(
                        "Prune: {}",
                        e.prunable
                            .as_ref()
                            .map_or("not eligible".into(), |r| label(r))
                    ),
                    9 => "Removal checks saved files, Git sequences and native resource owners."
                        .into(),
                    10 => "No forced removal, implicit source save or repository rollback.".into(),
                    _ => "Return / q list; v visits checked Git Status; Up/Down scroll.".into(),
                }
            })
            .flat_map(move |text| Wrapped {
                text,
                offset: Some(0),
                width: width.max(1),
            })
    }
    pub(in crate::session::git) fn worktree_row(&self, row: usize, width: usize) -> (String, bool) {
        if let Some(top) = self.worktrees.detail {
            return (
                self.worktree_detail_lines(width)
                    .nth(top + row)
                    .unwrap_or_default(),
                false,
            );
        }
        let Some(list) = self.worktrees() else {
            return (
                if row == 0 {
                    "Git Worktrees: checking".into()
                } else {
                    String::new()
                },
                false,
            );
        };
        if row < 3 {
            return (
                match row {
                    0 => "Git Worktrees: distinct working and common identities".into(),
                    1 => format!(
                        "{} entries / {:?}{}",
                        list.entries.len(),
                        list.completeness,
                        if self.worktrees.failed {
                            " / STALE: inspect diagnostics and refresh"
                        } else {
                            ""
                        }
                    ),
                    _ => {
                        "c create / v Status visit / L lock / U unlock / P prune / k remove".into()
                    }
                },
                false,
            );
        }
        list.entries
            .get(self.worktrees.top + row - 3)
            .map_or((String::new(), false), |e| {
                let selected = self.worktrees.selected.as_ref() == Some(&e.path);
                (
                    format!(
                        "{} {}{}{} {} {}",
                        if selected { ">" } else { " " },
                        if e.locked.is_some() { "[locked] " } else { "" },
                        if e.prunable.is_some() {
                            "[prunable] "
                        } else {
                            ""
                        },
                        label(e.path.as_os_str().as_encoded_bytes()),
                        e.head.map_or("unborn".into(), |h| h.hex()),
                        label(&e.branch),
                    ),
                    selected,
                )
            })
    }
}
impl Session {
    pub(in crate::session::git) fn open_git_worktrees(&mut self) -> Action {
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Open Git Status before Worktrees");
        };
        match self.git.engine.worktrees(context) {
            Ok(id) => {
                self.git.worktrees.request = Some(id);
                self.git.worktrees.detail = None;
                self.git.page = Page::Worktree;
                self.git.state.menu = false;
                self.git.painted = None;
                self.message("Checking exact Git worktree paths and identities")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn select_git_worktree(&mut self, index: usize) -> Action {
        if self.git.worktrees.detail.is_some() {
            return Action::Wait;
        }
        let Some(path) = self
            .git
            .worktrees()
            .and_then(|w| w.entries.get(index))
            .map(|e| e.path.clone())
        else {
            return Action::Wait;
        };
        self.git.worktrees.selected = Some(path);
        self.git.worktrees.top = self
            .git
            .worktrees
            .top
            .min(index)
            .max(index.saturating_sub(self.git_extent().saturating_sub(1)));
        Action::Redraw
    }
    pub(in crate::session::git) fn move_git_worktrees(
        &mut self,
        distance: usize,
        backward: bool,
    ) -> Action {
        if let Some(top) = self.git.worktrees.detail {
            let count = self
                .git
                .worktree_detail_lines(usize::from(self.size.width))
                .count();
            self.git.worktrees.detail = Some(if backward {
                top.saturating_sub(distance)
            } else {
                top.saturating_add(distance).min(count.saturating_sub(1))
            });
            self.git.painted = None;
            return Action::Redraw;
        }
        let Some(list) = self.git.worktrees() else {
            return Action::Wait;
        };
        let index = list
            .entries
            .iter()
            .position(|e| Some(&e.path) == self.git.worktrees.selected.as_ref())
            .unwrap_or(0);
        self.select_git_worktree(if backward {
            index.saturating_sub(distance)
        } else {
            index
                .saturating_add(distance)
                .min(list.entries.len().saturating_sub(1))
        })
    }
    pub(in crate::session::git) fn toggle_git_worktree_detail(&mut self) -> Action {
        self.git.worktrees.detail = if self.git.worktrees.detail.is_some() {
            None
        } else {
            Some(0)
        };
        self.git.painted = None;
        Action::Redraw
    }
    pub(in crate::session::git) fn quit_git_worktree_detail(&mut self) -> bool {
        if self.git.page == Page::Worktree && self.git.worktrees.detail.take().is_some() {
            self.git.painted = None;
            true
        } else {
            false
        }
    }
    fn current_worktrees(&self) -> Result<Arc<Worktrees>, String> {
        self.git
            .worktrees()
            .cloned()
            .filter(|_| {
                self.git.visible && self.git.page == Page::Worktree && !self.git.worktrees.failed
            })
            .ok_or("Open a current Worktrees list first".into())
    }
    pub(in crate::session::git) fn visit_git_worktree(&mut self) -> Action {
        let result = (|| {
            let list = self.current_worktrees()?;
            let path = self
                .git
                .worktrees
                .selected
                .clone()
                .ok_or("Select one exact worktree")?;
            self.git.engine.visit_worktree(list, &path)
        })();
        match result {
            Ok(id) => {
                self.git.worktrees.request = Some(id);
                self.message("Checking selected worktree for a distinct Git Status visit")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn prompt_git_worktree(&mut self, intent: Intent) -> Action {
        let list = match self.current_worktrees() {
            Ok(w) => w,
            Err(e) => return self.error(&e),
        };
        let selected = self.git.worktrees.selected.clone();
        match intent {
            Intent::Create | Intent::Lock => {
                let prompt = Prompt {
                    worktrees: list,
                    selected,
                    step: if matches!(intent, Intent::Create) {
                        Step::Destination
                    } else {
                        Step::Reason
                    },
                };
                if let Err(e) = self.input.argument(
                    if matches!(intent, Intent::Create) {
                        "New linked worktree path: "
                    } else {
                        "Worktree lock reason: "
                    },
                    "",
                ) {
                    return self.error(&e.to_string());
                }
                self.pending = Some(super::super::super::Pending::GitWorktree(Box::new(prompt)));
                Action::Redraw
            }
            _ => {
                let action = if matches!(intent, Intent::Prune) {
                    WorktreeAction::Prune
                } else {
                    let Some(path) = selected else {
                        return self.message("Select an exact worktree path");
                    };
                    if matches!(intent, Intent::Unlock) {
                        WorktreeAction::Unlock(path)
                    } else {
                        WorktreeAction::Remove(path)
                    }
                };
                self.prepare_git_worktree(list, action)
            }
        }
    }
    pub(in crate::session) fn git_worktree_argument(
        &mut self,
        mut prompt: Prompt,
        value: &str,
    ) -> Action {
        match prompt.step {
            Step::Destination => {
                if value.is_empty() || value.len() > 4096 || value.contains('\0') {
                    return self.message("Worktree destination requires 1..=4096 non-NUL bytes");
                }
                let path = PathBuf::from(value);
                let root = prompt
                    .worktrees
                    .observation
                    .identity
                    .worktree
                    .as_ref()
                    .unwrap_or(&prompt.worktrees.observation.identity.git_directory);
                let path = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                prompt.step = Step::Branch(path);
                if let Err(e) = self.input.argument("New worktree branch name: ", "") {
                    return self.error(&e.to_string());
                }
                self.pending = Some(super::super::super::Pending::GitWorktree(Box::new(prompt)));
                Action::Redraw
            }
            Step::Branch(path) => {
                let target =
                    prompt
                        .worktrees
                        .observation
                        .status
                        .as_ref()
                        .and_then(|s| match s.head {
                            Head::Branch { object, .. } | Head::Detached { object } => Some(object),
                            Head::Unborn { .. } => None,
                        });
                let Some(target) = target else {
                    return self.message("Worktree creation requires an exact existing HEAD");
                };
                self.prepare_git_worktree(
                    prompt.worktrees,
                    WorktreeAction::Create {
                        path,
                        branch: value.into(),
                        target,
                    },
                )
            }
            Step::Reason => {
                let Some(path) = prompt.selected else {
                    return self.message("Select an exact worktree path");
                };
                self.prepare_git_worktree(
                    prompt.worktrees,
                    WorktreeAction::Lock {
                        path,
                        reason: value.into(),
                    },
                )
            }
        }
    }
    fn prepare_git_worktree(&mut self, list: Arc<Worktrees>, action: WorktreeAction) -> Action {
        match self.git.engine.prepare_worktree(list, action) {
            Ok(id) => {
                self.git.worktrees.preparing = Some(id);
                self.message("Preparing exact worktree action and ownership review")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn publish_git_worktrees(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            self.git.worktrees.failed = true;
            return;
        }
        let preparing = self.git.worktrees.preparing.take() == Some(update.operation);
        self.git.worktrees.request = None;
        self.git.worktrees.failed = !update.published
            || self
                .git
                .engine
                .record(update.operation)
                .is_none_or(|r| r.outcome != crate::git::Outcome::Verified);
        if !update.published {
            self.message(
                "Worktree request failed or became stale; inspect $ diagnostics and refresh",
            );
            return;
        }
        if update.kind == crate::git::OperationKind::WorktreeVisit {
            if self.git.visible
                && self.git.page == Page::Worktree
                && self.pending.is_none()
                && self.input.focus() == Focus::Editing
            {
                self.git.state = State {
                    context: update.context,
                    ..State::default()
                };
                self.git.page = Page::Status;
                self.git.inspection = comparisons::Inspection::default();
                self.git.reconcile(self.git_extent());
                self.git.painted = None;
                self.message("Visited checked linked worktree Git Status; source buffers and windows retained");
            }
            return;
        }
        if !preparing {
            self.git.worktrees.selected = self.git.worktrees().and_then(|w| {
                self.git
                    .worktrees
                    .selected
                    .as_ref()
                    .filter(|p| w.entry(p).is_some())
                    .cloned()
                    .or_else(|| w.entries.first().map(|e| e.path.clone()))
            });
            self.git.worktrees.detail = None;
            self.move_git_worktrees(0, false);
            self.git.painted = None;
            self.message(if self.git.worktrees.failed {
                "Worktree effects require inspection; actual paths and diagnostics retained"
            } else {
                "Git Worktrees refreshed; exact request verified"
            });
            return;
        }
        if !self.git.visible
            || self.git.page != Page::Worktree
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let Some(plan) = update
            .context
            .and_then(|c| self.git.engine.context(c))
            .and_then(|c| c.worktree_plan.clone())
        else {
            return;
        };
        let sources = match self.git_worktree_sources(&plan) {
            Ok(s) => s,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        let size = 16384
            + plan
                .paths
                .iter()
                .map(|p| p.as_os_str().len() * 8 + 32)
                .sum::<usize>();
        let _admission = match self.git.engine.source_admission(size) {
            Ok(a) => a,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        let mut review = format!(
            "Worktree review\nAction: {}\nCommon repository:\n{}\nChecked source HEAD: {}\nExact paths:\n",
            plan.action.label(),
            label(
                plan.worktrees
                    .observation
                    .identity
                    .common_directory
                    .as_os_str()
                    .as_encoded_bytes()
            ),
            plan.source.refs.head.map_or("unborn".into(), |h| h.hex())
        );
        for path in &plan.paths {
            review.push_str(&label(path.as_os_str().as_encoded_bytes()));
            review.push('\n');
            if let Some(e) = plan.worktrees.entry(path) {
                review.push_str(&format!(
                    "Git directory: {}\nFull target HEAD: {}\n",
                    label(e.git_directory.as_os_str().as_encoded_bytes()),
                    e.head.map_or("unavailable".into(), |h| h.hex())
                ));
            }
        }
        match &plan.action {
            WorktreeAction::Create { branch, target, .. } => review.push_str(&format!(
                "New branch: {}\nFull checkout target:\n{}\n",
                label(branch.as_bytes()),
                target.hex()
            )),
            WorktreeAction::Lock { reason, .. } => {
                review.push_str(&format!("Lock reason: {}\n", label(reason.as_bytes())))
            }
            _ => {}
        }
        review.push_str("No force, implicit source save, process termination or rollback. y accepts; n or C-g cancels.");
        let message = format!(
            "Confirm worktree {}? y=accept, n=cancel",
            plan.action.label()
        );
        self.confirm_git_ref(
            std::sync::Arc::clone(&plan.source.refs),
            refs::RefAction::Worktree(plan),
            Some(sources),
            message,
            review,
        );
    }
}
