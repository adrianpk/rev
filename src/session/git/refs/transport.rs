use super::*;

pub(super) struct PushMark {
    pub context: ContextId,
    pub name: Vec<u8>,
    pub object: crate::git::ObjectId,
}
impl Session {
    pub(in crate::session::git) fn mark_git_push(&mut self, clear: bool) -> Action {
        if !self.git.visible || self.git.page != Page::Refs {
            return self.message("Push marks require current Refs");
        }
        if clear {
            self.git.refs.push_mark = None;
            return self.message("Push source mark cleared");
        }
        let Some(refs) = self.git.refs() else {
            return self.message("Current Refs required");
        };
        if let Err(e) = self.git.engine.check_refs(refs) {
            return self.error(&e);
        }
        let Some(reference) = self
            .git
            .refs
            .selected
            .as_ref()
            .and_then(|n| refs.reference(n))
            .filter(|r| {
                r.kind == RefKind::Branch && r.symbolic.is_empty() && r.object_type == b"commit"
            })
        else {
            return self.message("Mark one exact local branch as push source");
        };
        self.git.refs.push_mark = Some(PushMark {
            context: refs.observation.context,
            name: reference.name.clone(),
            object: reference.object,
        });
        self.message("One exact push source branch marked independently of navigation")
    }
    pub(in crate::session::git) fn prepare_git_push(&mut self) -> Action {
        let Some(refs) =
            self.git.refs().cloned().filter(|_| {
                self.git.visible && self.git.page == Page::Refs && !self.git.refs.failed
            })
        else {
            return self.message("Select a current local branch or mark a push source in Refs");
        };
        let source = if let Some(mark) = &self.git.refs.push_mark {
            if mark.context != refs.observation.context
                || refs
                    .reference(&mark.name)
                    .is_none_or(|r| r.object != mark.object)
            {
                return self.message(
                    "Marked push source changed; clear or mark its exact current object again",
                );
            }
            mark.name.clone()
        } else if let Some(name) = &self.git.refs.selected {
            name.clone()
        } else {
            return self.message(
                "Mark one exact source branch before using a selected remote's push route",
            );
        };
        match self
            .git
            .engine
            .prepare_push(refs, &source, self.git.refs.remote_selected.as_deref())
        {
            Ok(id) => {
                self.git.refs.push_preparing = Some(id);
                self.git.painted = None;
                self.message("Checking the exact source branch and its configured upstream before push review")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn publish_git_push_preparation(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.refs.push_preparing = None;
        if !self.git.visible
            || self.git.page != Page::Refs
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let Some(intent) = update
            .context
            .filter(|_| update.published)
            .and_then(|id| self.git.engine.push_intent(id))
            .cloned()
        else {
            let diagnostic = self
                .git
                .engine
                .record(update.operation)
                .and_then(|r| r.diagnostic.as_deref())
                .unwrap_or("inspect git-processes");
            self.message(&format!(
                "Push preparation failed: {}",
                label(diagnostic.as_bytes())
            ));
            return;
        };
        let message = format!(
            "Push {} at {} to configured {} destination {}? y=accept, n=cancel",
            label(intent.source()),
            intent.object().hex(),
            label(intent.remote()),
            label(intent.destination())
        );
        let review = format!(
            "Single upstream branch push review\nExact source ref:\n{}\nExact source object:\n{}\nConfigured upstream remote:\n{}\nExact destination ref:\n{}\nOne branch; no force, deletion, tags, mirror or extra refspecs.\nTransport/configuration values remain private.\nUp/Down or PageUp/PageDown scroll. y accepts; n or C-g cancels.",
            label(intent.source()),
            intent.object().hex(),
            label(intent.remote()),
            label(intent.destination())
        );
        self.confirm_git_ref(
            Proposal {
                refs: std::sync::Arc::clone(intent.refs()),
                action: RefAction::Push(intent),
                sources: None,
            },
            message,
            review,
        );
    }
}
