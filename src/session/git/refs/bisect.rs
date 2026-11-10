use super::*;
use crate::git::{BisectAction, BisectResult};

pub(in crate::session) struct Prompt {
    refs: Arc<Refs>,
    bad: Option<String>,
    _admission: crate::git::budget::Lease,
}
impl Session {
    pub(in crate::session::git) fn prompt_git_bisect(&mut self) -> Action {
        let Some(refs) = self
            .git
            .refs()
            .cloned()
            .filter(|_| self.git.visible && self.git.page == Page::Sequence)
        else {
            return self
                .message("Open git-operation-status before preparing manual bisect boundaries");
        };
        if let Err(e) = self.git.engine.check_refs(&refs) {
            return self.error(&e);
        }
        if let Err(e) = self.git_branch_sources(&refs) {
            return self.error(&e);
        }
        let admission = match self.git.engine.source_admission(16384) {
            Ok(a) => a,
            Err(e) => return self.error(&e),
        };
        if let Err(e) = self.input.argument("Explicit bad revision: ", "") {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::GitBisect(Box::new(Prompt {
            refs,
            bad: None,
            _admission: admission,
        })));
        self.message(
            "Manual bisect: enter explicit bad and good boundaries; no test command will run",
        )
    }
    pub(in crate::session) fn git_bisect_argument(
        &mut self,
        mut prompt: Prompt,
        text: &str,
    ) -> Action {
        if text.is_empty() || text.len() > 4000 || text.contains('\0') {
            return self.message("Bisect boundary requires 1..4000 bytes without NUL");
        }
        if prompt.bad.is_none() {
            prompt.bad = Some(text.into());
            if let Err(e) = self.input.argument("Explicit good revision: ", "") {
                return self.error(&e.to_string());
            }
            self.pending = Some(Pending::GitBisect(Box::new(prompt)));
            return self.message("Manual bisect: enter the explicit known-good boundary");
        }
        match self.git.engine.prepare_bisect(
            prompt.refs,
            prompt.bad.as_deref().expect("captured bad boundary"),
            text,
        ) {
            Ok(id) => {
                self.git.refs.bisect_preparing = Some((id, None));
                self.message("Preparing exact manual bisect boundaries without changing Git")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn request_git_bisect(&mut self, action: BisectAction) -> Action {
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Open Git Status before requesting manual bisect control");
        };
        match self.git.engine.sequence(context) {
            Ok(id) => {
                self.git.refs.bisect_preparing = Some((id, Some(action)));
                self.git.page = Page::Sequence;
                self.git.painted = None;
                self.message("Checking exact manual bisect candidate and original context")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn publish_git_bisect_preparation(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        let action = self.git.refs.bisect_preparing.take().and_then(|(_, a)| a);
        if !self.git.visible || self.pending.is_some() || self.input.focus() != Focus::Editing {
            return;
        }
        let Some(context) = update
            .context
            .filter(|_| update.published && update.context == self.git.state.context)
            .and_then(|id| self.git.engine.context(id))
        else {
            let detail = self
                .git
                .engine
                .record(update.operation)
                .and_then(|r| r.diagnostic.as_deref())
                .unwrap_or("inspect git-processes")
                .to_string();
            self.message(&format!("Manual bisect preparation failed: {detail}"));
            return;
        };
        let _admission = match self.git.engine.source_admission(65536) {
            Ok(a) => a,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        let (refs, action, message, review) = if let Some(action) = action {
            let Some(sequence) = context.sequence.clone() else {
                self.message("No fresh bisect state is available");
                return;
            };
            if let Err(e) = sequence.bisect_admission(action) {
                self.error(&e);
                return;
            }
            let review = bisect_control_review(&sequence, action);
            (
                std::sync::Arc::clone(&sequence.refs),
                RefAction::BisectControl(sequence, action),
                format!(
                    "Confirm manual bisect {}? y=accept, n=cancel",
                    action.label()
                ),
                review,
            )
        } else {
            let Some(plan) = context.bisect_plan.clone() else {
                self.message("No exact bisect plan is available");
                return;
            };
            let review = format!(
                "Manual bisect boundary review\nExplicit bad commit:\n{}\nExplicit good commit:\n{}\nOriginal branch:\n{}\nOriginal object:\n{}\nEach candidate needs your separate good/bad/skip decision. Reset restores this checked original context. No test/build command runs. y / C-Enter / C-c C-c accepts; Return does not; n / C-g cancels.",
                plan.bad.hex(),
                plan.good.hex(),
                label(&plan.refs.head_ref),
                plan.refs.head.map_or("unavailable".into(), |id| id.hex())
            );
            (
                std::sync::Arc::clone(&plan.refs),
                RefAction::BisectStart(plan),
                "Confirm manual bisect with these exact boundaries? y=accept, n=cancel".into(),
                review,
            )
        };
        let sources = match self.git_branch_sources(&refs) {
            Ok(s) => s,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        self.confirm_git_ref(
            Proposal {
                refs,
                action,
                sources: Some(sources),
            },
            message,
            review,
        );
    }
}

impl Work {
    pub(in crate::session::git) fn bisect_summary(&self) -> Option<&'static str> {
        self.state.context.and_then(|id|self.engine.context(id)).and_then(|c|c.sequence.as_ref()).map(|s| {
            match s.bisect.as_ref().map(|b|b.result) {
                Some(BisectResult::Searching)=>"Manual bisect candidate checked; explicitly classify good/bad/skip or reset",
                Some(BisectResult::Found(_))=>"Manual bisect first bad commit identified; explicit reset remains separate",
                Some(BisectResult::Ambiguous)=>"Manual bisect remains ambiguous because of skipped candidates; no unique first bad is verified",
                None=>"Manual bisect reset verified; original branch, object, index and disk restored",
            }
        })
    }
}

fn bisect_control_review(
    sequence: &crate::git::Sequence,
    action: crate::git::BisectAction,
) -> String {
    let state = sequence.bisect.as_ref().expect("admitted bisect");
    let plan = state.plan.as_ref().expect("admitted original lineage");
    let mut review = format!(
        "Manual bisect control review\nExplicit action: {}\nExact current candidate:\n{}\nKnown bad:\n{}\nKnown good boundaries: {}\nSkipped candidates: {}\nOriginal branch:\n{}\nOriginal object:\n{}\nResult: {}\nReset restores only this checked original context. Classifications express your result; no test/build is run. y / C-Enter / C-c C-c accepts; Return does not; n / C-g cancels.",
        action.label(),
        sequence
            .refs
            .head
            .map_or("unavailable".into(), |id| id.hex()),
        state.bad.hex(),
        state.good.len(),
        state.skipped.len(),
        label(&plan.refs.head_ref),
        plan.refs.head.map_or("unavailable".into(), |id| id.hex()),
        state.result.label()
    );
    for id in &state.good {
        review.push_str(&format!("\nExact good boundary:\n{}", id.hex()));
    }
    for id in &state.skipped {
        review.push_str(&format!("\nExact skipped candidate:\n{}", id.hex()));
    }
    review
}
