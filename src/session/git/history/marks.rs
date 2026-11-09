use super::*;
impl Work {
    pub(in crate::session::git) fn revision_target(
        &self,
    ) -> Result<(Arc<History>, ObjectId), String> {
        if !self.visible
            || self.page != Page::History
            || self.ancestry.request.is_some()
            || self.ancestry.failed
        {
            return Err("Select a current confirmed History revision".into());
        }
        let history = self.history().ok_or("History is unavailable")?;
        let target = self
            .ancestry
            .selected
            .filter(|id| history.revision(*id).is_some())
            .ok_or("No exact revision selected")?;
        if self
            .observation()
            .is_none_or(|o| !std::ptr::eq(o, history.observation.as_ref()))
        {
            return Err("History observation changed; reopen History".into());
        }
        Ok((std::sync::Arc::clone(history), target))
    }
    pub(in crate::session::git) fn revision_mark_label(&self, id: ObjectId) -> String {
        let scope = self.history().map(|h| (h.observation.context, h.root));
        let stale = scope != self.ancestry.mark_scope;
        let base = self.ancestry.base == Some(id);
        let order = self.ancestry.ordered.iter().position(|i| *i == id);
        format!(
            "{}{}{}",
            if stale && (base || order.is_some()) {
                "!"
            } else {
                ""
            },
            if base { "*" } else { " " },
            order.map_or(String::new(), |n| format!("[{}]", n + 1))
        )
    }
}
impl Session {
    pub(in crate::session::git) fn mark_git_revision(
        &mut self,
        sequence: bool,
        remove: bool,
        clear: bool,
    ) -> Action {
        if clear {
            self.git.ancestry.base = None;
            self.git.ancestry.ordered.clear();
            self.git.ancestry.mark_scope = None;
            return self.message("History range base and ordered revision marks cleared");
        }
        let (history, target) = match self.git.revision_target() {
            Ok(t) => t,
            Err(e) => return self.error(&e),
        };
        let scope = (history.observation.context, history.root);
        if self.git.ancestry.mark_scope.is_some_and(|s| s != scope) {
            return self.message(
                "Revision mark context changed; git-unmark-all clears its retained roles",
            );
        }
        self.git.ancestry.mark_scope = Some(scope);
        if sequence {
            if remove {
                self.git.ancestry.ordered.retain(|id| *id != target);
            } else if self.git.ancestry.ordered.contains(&target) {
                return self.message("Revision is already in the ordered set");
            } else if self.git.ancestry.ordered.len() >= 256 {
                return self.message("Ordered revision mark limit reached (256)");
            } else {
                self.git.ancestry.ordered.push(target);
            }
        } else if remove {
            if self.git.ancestry.base == Some(target) {
                self.git.ancestry.base = None;
            }
        } else {
            self.git.ancestry.base = Some(target);
        }
        self.message(if sequence {
            "Ordered revision marks updated; range base retained"
        } else {
            "Range base updated; ordered revision marks retained"
        })
    }
    pub(in crate::session::git) fn show_git_range(&mut self) -> Action {
        let (history, end) = match self.git.revision_target() {
            Ok(t) => t,
            Err(e) => return self.error(&e),
        };
        if self.git.ancestry.mark_scope != Some((history.observation.context, history.root)) {
            return self.message("Select one compatible History range base with *");
        }
        let Some(base) = self.git.ancestry.base else {
            return self.message("Select one History range base with *");
        };
        let result = self.git.engine.show_range(history, base, end);
        self.git.ancestry.revision_origin = Page::History;
        self.git.ancestry.detail_selected = None;
        self.git.ancestry.detail_cursor = 0;
        self.git.ancestry.detail_top = 0;
        self.start_git_history(result, Page::Revision)
    }
}
