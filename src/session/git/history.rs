mod inspection;
mod marks;
use super::*;
use crate::git::{History, ObjectId, RevisionDetail};

#[derive(Default)]
pub(super) struct Inspection {
    pub request: Option<(OperationId, Page)>,
    pub selected: Option<ObjectId>,
    pub top: usize,
    pub detail_top: usize,
    pub failed: bool,
    pub base: Option<ObjectId>,
    pub ordered: Vec<ObjectId>,
    pub mark_scope: Option<(ContextId, Option<ObjectId>)>,
    _admission: Option<crate::git::budget::Lease>,
    pub detail_selected: Option<inspection::DetailTarget>,
    pub detail_cursor: usize,
    pub revision_origin: Page,
    pub file_origin: Page,
    pub reflog_selected: Option<(ObjectId, Vec<u8>)>,
    pub reflog_top: usize,
    pub blame_selected: Option<usize>,
    pub blame_top: usize,
    pub file_top: usize,
}

impl Inspection {
    pub(super) fn new(engine: &Repositories) -> std::io::Result<Self> {
        Ok(Self {
            _admission: Some(
                engine
                    .source_admission(65536)
                    .map_err(std::io::Error::other)?,
            ),
            ordered: Vec::with_capacity(256),
            ..Self::default()
        })
    }
}

impl Work {
    pub(super) fn history(&self) -> Option<&Arc<History>> {
        self.state
            .context
            .and_then(|id| self.engine.context(id))?
            .history
            .as_ref()
    }
    pub(super) fn revision(&self) -> Option<&Arc<RevisionDetail>> {
        self.state
            .context
            .and_then(|id| self.engine.context(id))?
            .revision
            .as_ref()
    }
    pub(super) fn history_header(&self, row: usize) -> String {
        let Some(history) = self.history() else {
            return if row == 0 {
                "Git History: checking ancestry".into()
            } else {
                String::new()
            };
        };
        match row {
            0 => format!(
                "History root: {}",
                history
                    .root
                    .map_or("unborn / no commits".into(), |id| id.hex())
            ),
            1 => format!(
                "{} revisions / {} pages / {}",
                history.revisions().count(),
                history.pages.len(),
                if history.page_limit {
                    "PAGE LIMIT reached"
                } else if history.more {
                    "more ancestry: f"
                } else {
                    "complete ancestry"
                }
            ),
            2 => format!(
                "Base: {} / {} ordered / {}{}",
                self.ancestry.base.map_or("none".into(), |id| id.hex()),
                self.ancestry.ordered.len(),
                if self.ancestry.request.is_some() {
                    "Checking history  "
                } else {
                    ""
                },
                if self.ancestry.failed {
                    "STALE: inspection failed; confirmed facts retained; $ diagnostics"
                } else {
                    "read-only / full object identities"
                }
            ),
            _ => String::new(),
        }
    }
}

impl Session {
    pub(super) fn open_git_history(&mut self) -> Action {
        if !self.git.visible {
            return self.message("Open Git Status before selecting History");
        }
        let Some(context) = self.git.state.context else {
            return self.message("Repository context is still checking");
        };
        if !matches!(
            self.git.status().map(|s| &s.head),
            Some(Head::Branch { .. } | Head::Unborn { .. })
        ) {
            if let Err(e) = self.input.argument("Exact history root revision: ", "") {
                return self.error(&e.to_string());
            }
            self.pending = Some(super::super::Pending::GitHistoryRoot(context));
            return self.message("Detached or bare History requires an explicit root revision");
        }
        self.git_history_root(context, "")
    }
    pub(in crate::session) fn git_history_root(
        &mut self,
        context: ContextId,
        root: &str,
    ) -> Action {
        if root.is_empty()
            && !matches!(
                self.git.status().map(|s| &s.head),
                Some(Head::Branch { .. } | Head::Unborn { .. })
            )
        {
            return self.message("Detached or bare History requires a nonempty explicit root");
        }
        let result = self
            .git
            .engine
            .log(context, (!root.is_empty()).then_some(root));
        self.start_git_history(result, Page::History)
    }
    pub(super) fn start_git_history(
        &mut self,
        result: Result<OperationId, String>,
        page: Page,
    ) -> Action {
        match result {
            Ok(operation) => {
                self.git.ancestry.request = Some((operation, page));
                self.git.state.menu = false;
                self.git.page = page;
                self.git.painted = None;
                self.message("Checking exact Git history objects")
            }
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn continue_git_history(&mut self, next: bool) -> Action {
        let Some(history) = self.git.history().cloned().filter(|_| {
            self.git.visible && matches!(self.git.page, Page::History | Page::Revision)
        }) else {
            return self.message("Select a current History observation first");
        };
        if self.git.page == Page::Revision && !next {
            let Some(detail) = self.git.revision().cloned() else {
                return Action::Wait;
            };
            let result = self.git.engine.refresh_revision(detail);
            return self.start_git_history(result, Page::Revision);
        }
        let result = self.git.engine.continue_history(history, next);
        self.start_git_history(result, Page::History)
    }
    pub(super) fn show_git_revision(&mut self) -> Action {
        if self.git.visible
            && matches!(
                self.git.page,
                Page::Reflog | Page::Blame | Page::Revision | Page::RevisionFile
            )
        {
            return self.show_git_inspection();
        }

        let Some(history) = self
            .git
            .history()
            .cloned()
            .filter(|_| self.git.visible && self.git.page == Page::History)
        else {
            return self.message("Select an exact revision in History first");
        };
        let Some(object) = self.git.ancestry.selected else {
            return self.message("History has no selected revision");
        };
        let Some(revision) = history.revision(object) else {
            return self.message("Selected revision is no longer admitted");
        };
        let parent = revision.parents.first().copied();
        let result = self.git.engine.show_revision(history, object, parent);
        self.git.ancestry.revision_origin = Page::History;
        self.git.ancestry.detail_top = 0;
        self.start_git_history(result, Page::Revision)
    }
    pub(in crate::session) fn git_history_parent(
        &mut self,
        detail: Arc<RevisionDetail>,
        value: &str,
    ) -> Action {
        let parent = match ObjectId::parse(value.as_bytes(), detail.object.format()) {
            Ok(p) => p,
            Err(e) => return self.error(&e),
        };
        let result = self.git.engine.compare_parent(detail, parent);
        self.git.ancestry.detail_top = 0;
        self.start_git_history(result, Page::Revision)
    }
    pub(super) fn publish_git_history(&mut self, update: &crate::git::Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        let requested = self.git.ancestry.request.take().map(|(_, page)| page);
        self.git.ancestry.failed = !update.published;
        if update.published {
            let selected = self.git.ancestry.selected;
            self.git.ancestry.selected = self.git.history().and_then(|h| {
                selected
                    .filter(|id| h.revision(*id).is_some())
                    .or_else(|| h.revisions().next().map(|r| r.object))
            });
        }
        if update.published {
            self.reconcile_git_inspection(requested);
        }
        self.message(if update.published {
            "Git history inspected; source, index and HEAD unchanged"
        } else {
            "History inspection failed; confirmed facts retained; $ diagnostics"
        });
    }
    pub(super) fn move_git_history(&mut self, distance: usize, backward: bool) -> Action {
        if matches!(
            self.git.page,
            Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
        ) {
            return self.move_git_inspection(distance, backward);
        }
        let Some(history) = self.git.history() else {
            return Action::Wait;
        };
        let index = history
            .revisions()
            .position(|r| Some(r.object) == self.git.ancestry.selected)
            .unwrap_or(0);
        let next = if backward {
            index.saturating_sub(distance)
        } else {
            index
                .saturating_add(distance)
                .min(history.revisions().count().saturating_sub(1))
        };
        let selected = history.revisions().nth(next).map(|r| r.object);
        self.git.ancestry.selected = selected;
        self.git.ancestry.top = self
            .git
            .ancestry
            .top
            .min(next)
            .max(next.saturating_sub(self.git_extent().saturating_sub(1)));
        Action::Redraw
    }
    pub(super) fn select_git_history(&mut self, row: usize) -> Action {
        if self.git.page != Page::History {
            return self.select_git_inspection(row);
        }
        if self.git.page == Page::History {
            self.git.ancestry.selected = self.git.history().and_then(|h| {
                h.revisions()
                    .nth(self.git.ancestry.top + row)
                    .map(|r| r.object)
            });
        }
        Action::Redraw
    }
}
