use super::*;
pub(in crate::session) mod worktrees;
use crate::git::{ObjectId, StashAction, Stashes, Update};

pub(super) struct Inspection {
    pub request: Option<OperationId>,
    pub preparing: Option<OperationId>,
    selected: Option<ObjectId>,
    cursor: usize,
    pub top: usize,
    detail: Option<usize>,
    failed: bool,
}
impl Inspection {
    pub(super) fn new() -> Self {
        Self {
            request: None,
            preparing: None,
            selected: None,
            cursor: 0,
            top: 0,
            detail: None,
            failed: false,
        }
    }
}
pub(in crate::session) enum Intent {
    Create,
    Apply,
    Pop,
    Branch,
    Drop,
}
pub(in crate::session) struct Prompt {
    stashes: Arc<Stashes>,
    intent: Intent,
    object: Option<ObjectId>,
}
impl Work {
    pub(super) fn stashes(&self) -> Option<&Arc<Stashes>> {
        self.state
            .context
            .and_then(|c| self.engine.context(c))?
            .stashes
            .as_ref()
    }
    pub(super) fn stash_token(&self) -> (Option<OperationId>, usize) {
        (
            self.stashes().map(|s| s.id),
            self.repository.detail.unwrap_or(self.repository.top),
        )
    }
    fn stash_detail_lines(&self, width: usize) -> impl Iterator<Item = String> + '_ {
        let detail = self
            .state
            .context
            .and_then(|c| self.engine.context(c))
            .and_then(|c| c.stash_detail.as_ref())
            .filter(|d| {
                Some(d.object) == self.repository.selected
                    && self.stashes().is_some_and(|s| Arc::ptr_eq(s, &d.stashes))
            });
        std::iter::once(
            detail.map_or("Git Stash: detail unavailable; v refreshes".into(), |d| {
                format!("Git Stash {} / {:?}", d.object.hex(), d.completeness)
            }),
        )
        .chain(
            detail
                .into_iter()
                .flat_map(|d| d.bytes.split(|b| *b == b'\n'))
                .flat_map(|line| line.chunks(2048))
                .map(label),
        )
        .flat_map(move |text| Wrapped {
            text,
            offset: Some(0),
            width: width.max(1),
        })
    }
    pub(super) fn stash_row(&self, row: usize, width: usize) -> (String, bool) {
        if let Some(top) = self.repository.detail {
            return (
                self.stash_detail_lines(width)
                    .nth(top + row)
                    .unwrap_or_default(),
                false,
            );
        }
        let Some(stashes) = self.stashes() else {
            return (
                if row == 0 {
                    "Git Stashes: checking".into()
                } else {
                    String::new()
                },
                false,
            );
        };
        if row < 3 {
            return (
                match row {
                    0 => "Git Stashes: full object identity; saved files only".into(),
                    1 => format!(
                        "{} entries / {:?}{}",
                        stashes.entries.len(),
                        stashes.completeness,
                        if self.repository.failed {
                            " / STALE: inspect diagnostics and refresh"
                        } else {
                            ""
                        }
                    ),
                    _ => "c create / v or Return inspect / a apply / p pop / b branch / k drop"
                        .into(),
                },
                false,
            );
        }
        stashes
            .entries
            .get(self.repository.top + row - 3)
            .map_or((String::new(), false), |e| {
                let selected = Some(e.object) == self.repository.selected
                    && self.repository.top + row - 3 == self.repository.cursor;
                (
                    format!(
                        "{} {} {} {}",
                        if selected { ">" } else { " " },
                        label(&e.selector),
                        e.object.hex(),
                        label(&e.subject)
                    ),
                    selected,
                )
            })
    }
}
impl Session {
    pub(super) fn open_git_stashes(&mut self) -> Action {
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Open Git Status before Stashes");
        };
        match self.git.engine.stashes(context) {
            Ok(id) => {
                self.git.repository.request = Some(id);
                self.git.repository.detail = None;
                self.git.page = Page::Stash;
                self.git.state.menu = false;
                self.git.painted = None;
                self.message("Checking exact Git stash objects")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn select_git_stash(&mut self, index: usize) -> Action {
        if self.git.repository.detail.is_some() {
            return Action::Wait;
        }
        let Some(entry) = self.git.stashes().and_then(|s| s.entries.get(index)) else {
            return Action::Wait;
        };
        self.git.repository.selected = Some(entry.object);
        self.git.repository.cursor = index;
        self.git.repository.top = self
            .git
            .repository
            .top
            .min(index)
            .max(index.saturating_sub(self.git_extent().saturating_sub(1)));
        Action::Redraw
    }
    pub(super) fn move_git_stashes(&mut self, distance: usize, backward: bool) -> Action {
        if let Some(top) = self.git.repository.detail {
            let count = self
                .git
                .stash_detail_lines(usize::from(self.size.width))
                .count();
            self.git.repository.detail = Some(if backward {
                top.saturating_sub(distance)
            } else {
                top.saturating_add(distance).min(count.saturating_sub(1))
            });
            self.git.painted = None;
            return Action::Redraw;
        }
        let Some(stashes) = self.git.stashes() else {
            return Action::Wait;
        };
        let index = self
            .git
            .repository
            .cursor
            .min(stashes.entries.len().saturating_sub(1));
        let next = if backward {
            index.saturating_sub(distance)
        } else {
            index
                .saturating_add(distance)
                .min(stashes.entries.len().saturating_sub(1))
        };
        self.select_git_stash(next)
    }
    pub(super) fn quit_git_stash_detail(&mut self) -> bool {
        if self.git.page == Page::Stash && self.git.repository.detail.take().is_some() {
            self.git.painted = None;
            true
        } else {
            false
        }
    }
    pub(super) fn show_git_stash(&mut self) -> Action {
        if self.quit_git_stash_detail() {
            return Action::Redraw;
        }
        let Some(stashes) = self.git.stashes().cloned().filter(|_| {
            self.git.visible && self.git.page == Page::Stash && !self.git.repository.failed
        }) else {
            return self.message("Open current Stashes and select an exact object");
        };
        let Some(object) = self.git.repository.selected else {
            return self.message("No stash selected");
        };
        match self.git.engine.show_stash(stashes, object) {
            Ok(id) => {
                self.git.repository.request = Some(id);
                self.message("Inspecting selected full stash object without changing sources")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn prompt_git_stash(&mut self, intent: Intent) -> Action {
        let Some(stashes) = self.git.stashes().cloned().filter(|_| {
            self.git.visible && self.git.page == Page::Stash && !self.git.repository.failed
        }) else {
            return self.message("Open a current Stashes list first");
        };
        let prompt = Prompt {
            stashes,
            intent,
            object: self.git.repository.selected,
        };
        if matches!(prompt.intent, Intent::Create | Intent::Branch) {
            if let Err(e) = self.input.argument(
                if matches!(prompt.intent, Intent::Create) {
                    "Stash message (optional): "
                } else {
                    "New stash branch name: "
                },
                "",
            ) {
                return self.error(&e.to_string());
            }
            self.pending = Some(super::super::Pending::GitRepository(Box::new(prompt)));
            Action::Redraw
        } else {
            self.git_repository_argument(prompt, "")
        }
    }
    pub(in crate::session) fn git_repository_argument(
        &mut self,
        prompt: Prompt,
        value: &str,
    ) -> Action {
        let action = match prompt.intent {
            Intent::Create => StashAction::Create {
                message: value.into(),
            },
            _ => {
                let Some(object) = prompt.object else {
                    return self.message("Select an exact stash object first");
                };
                match prompt.intent {
                    Intent::Apply => StashAction::Apply(object),
                    Intent::Pop => StashAction::Pop(object),
                    Intent::Branch => StashAction::Branch {
                        object,
                        name: value.into(),
                    },
                    Intent::Drop => StashAction::Drop(object),
                    Intent::Create => unreachable!(),
                }
            }
        };
        match self.git.engine.prepare_stash(prompt.stashes, action) {
            Ok(id) => {
                self.git.repository.preparing = Some(id);
                self.message("Preparing exact stash action and saved source review")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn publish_git_repository(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            self.git.repository.failed = true;
            return;
        }
        let preparing = self.git.repository.preparing.take() == Some(update.operation);
        self.git.repository.request = None;
        self.git.repository.failed = !update.published
            || self
                .git
                .engine
                .record(update.operation)
                .is_none_or(|r| r.outcome != crate::git::Outcome::Verified);
        if !update.published {
            self.message("Stash request failed or became stale; inspect $ diagnostics and refresh");
            return;
        }
        if update.kind == crate::git::OperationKind::Stash {
            self.git.repository.detail = None;
        }
        if update.kind == crate::git::OperationKind::StashShow {
            self.git.repository.detail = Some(0);
        }
        if !preparing {
            let selected = self.git.stashes().and_then(|s| {
                self.git
                    .repository
                    .selected
                    .filter(|o| s.entry(*o).is_some())
                    .or_else(|| s.entries.first().map(|e| e.object))
            });
            self.git.repository.cursor = self
                .git
                .stashes()
                .and_then(|s| s.entries.iter().position(|e| Some(e.object) == selected))
                .unwrap_or(0);
            self.git.repository.selected = selected;
            self.move_git_stashes(0, false);
            self.observe_files();
            self.git.painted = None;
            self.message(if self.git.repository.failed {
                "Stash effects require inspection; actual conflict/state and diagnostics retained"
            } else {
                "Git Stashes refreshed; exact request verified"
            });
            return;
        }
        if !self.git.visible
            || self.git.page != Page::Stash
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let Some(plan) = update
            .context
            .and_then(|c| self.git.engine.context(c))
            .and_then(|c| c.stash_plan.clone())
        else {
            return;
        };
        let sources = match self.git_stash_sources(&plan) {
            Ok(s) => s,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        let size = 8192
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
        let object = plan.action.object().map_or("new stash".into(), |o| o.hex());
        let mut review = format!(
            "Stash review\nAction: {}\nFull exact object:\n{}\nChecked HEAD:\n{}\nSaved files only; no implicit save.\n",
            plan.action.label(),
            object,
            plan.source.refs.head.map_or("unborn".into(), |h| h.hex())
        );
        match &plan.action {
            StashAction::Create { message } => review.push_str(&format!(
                "Message: {}\nIncludes tracked and untracked files; ignored files stay excluded.\n",
                label(message.as_bytes())
            )),
            StashAction::Branch { name, .. } => review.push_str(&format!(
                "New local branch: {}\nBase: {}\n",
                label(name.as_bytes()),
                plan.base.map_or("unavailable".into(), |b| b.hex())
            )),
            _ => {}
        }
        review.push_str("Affected saved paths:\n");
        for path in &plan.paths {
            review.push_str(&label(path.as_os_str().as_encoded_bytes()));
            review.push('\n');
        }
        review.push_str(
            "y accepts; n or C-g cancels. No automatic conflict resolution or continuation.",
        );
        let message = format!(
            "Confirm stash {} / {}? y=accept, n=cancel",
            plan.action.label(),
            object
        );
        self.confirm_git_ref(
            plan.source.refs.clone(),
            refs::RefAction::Stash(plan),
            Some(sources),
            message,
            review,
        );
    }
}
