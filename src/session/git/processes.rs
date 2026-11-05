use super::{Action, Page, Session, Work};
use crate::{
    display::git::DiagnosticLines,
    git::{OperationId, Record},
};

impl Work {
    pub(super) fn process_ids(&self) -> impl Iterator<Item = OperationId> + '_ {
        self.engine
            .progress()
            .map(|p| p.operation)
            .chain(self.engine.records().iter().rev().map(|r| r.id))
    }
    pub(super) fn process_header(&self, row: usize) -> String {
        match (self.page, row) {
            (Page::Processes, 0) => "Git Processes: active first, then recent".into(),
            (Page::Processes, 1) => "Owned Git operations; exact session identities".into(),
            (Page::Processes, 2) => self
                .process_selected
                .filter(|id| !self.process_ids().any(|current| current == *id))
                .map_or(String::new(), |_| {
                    "Selected record expired; choose another operation explicitly".into()
                }),
            (Page::Process(id), 0) => format!("Git Process: {id:?} — {}", self.process_kind(id)),
            (Page::Process(id), 1) => self.process_summary(id).unwrap_or_else(|| {
                "Record expired; this identity cannot target another operation".into()
            }),
            (Page::Process(_), 2) => {
                "Inspection is read-only. Cancellation stops owned work; it does not roll back Git."
                    .into()
            }
            _ => String::new(),
        }
    }
    fn process_kind(&self, id: OperationId) -> &'static str {
        let kind = self
            .engine
            .progress()
            .find(|p| p.operation == id)
            .map(|p| p.kind)
            .or_else(|| self.engine.record(id).map(|r| r.kind));
        match kind {
            Some(crate::git::OperationKind::BisectPrepare) => "manual bisect preparation",
            Some(crate::git::OperationKind::Bisect) => "manual bisect",
            Some(crate::git::OperationKind::ApplicationPrepare) => "ordered revision preparation",
            Some(crate::git::OperationKind::ApplicationControl) => "ordered revision control",
            Some(crate::git::OperationKind::Application) => "ordered revision application",
            Some(crate::git::OperationKind::Integration) => "checked integration",
            Some(crate::git::OperationKind::AutosquashPrepare) => "autosquash preparation",
            Some(crate::git::OperationKind::SequenceControl) => "checked sequence control",
            Some(crate::git::OperationKind::Sequence) => "Git sequence observation",
            Some(crate::git::OperationKind::Push) => "push",
            Some(crate::git::OperationKind::PushPrepare) => "push preparation",
            Some(crate::git::OperationKind::Fetch) => "fetch",
            Some(crate::git::OperationKind::TagPrepare) => "Prepare exact local tag",
            Some(crate::git::OperationKind::TagCreate) => "Create checked local tag",
            Some(crate::git::OperationKind::TagDelete) => "Delete exact local tag",
            Some(crate::git::OperationKind::Refs) => "Refs observation",
            Some(crate::git::OperationKind::Branch) => "Checked local branch",
            Some(crate::git::OperationKind::History) => "history inspection",
            Some(crate::git::OperationKind::Status) => "Status observation",
            Some(crate::git::OperationKind::Diff) => "Diff comparison",
            Some(crate::git::OperationKind::Stage) => "Stage index",
            Some(crate::git::OperationKind::Unstage) => "Unstage index",
            Some(crate::git::OperationKind::Discard) => "Discard working path",
            Some(crate::git::OperationKind::Commit) => "Commit checked index",
            Some(crate::git::OperationKind::CommitPrepare) => "Prepare exact commit composition",
            None => "expired operation",
        }
    }
    fn process_summary(&self, id: OperationId) -> Option<String> {
        if let Some(progress) = self.engine.progress().find(|p| p.operation == id) {
            return Some(format!(
                "{id:?}  {}  {:?}  {}s  child {}{}",
                self.process_kind(id),
                progress.phase,
                progress.started.elapsed().as_secs(),
                progress.child.map_or("none".into(), |id| id.to_string()),
                if progress.stopping {
                    "  cancellation requested"
                } else {
                    ""
                },
            ));
        }
        let record = self.engine.record(id)?;
        Some(format!(
            "{id:?}  {}  {:?}  effects {:?}  {}ms  {}{}",
            self.process_kind(id),
            record.outcome,
            record.effects,
            record.ended.duration_since(record.started).as_millis(),
            record
                .exit
                .map_or("exit unavailable".into(), |s| s.to_string()),
            if record.output_truncated {
                "  TRUNCATED output"
            } else {
                ""
            },
        ))
    }
    pub(super) fn process_rows(&self) -> Box<dyn Iterator<Item = (String, bool)> + '_> {
        match self.page {
            Page::Status
            | Page::Diff
            | Page::History
            | Page::Refs
            | Page::Sequence
            | Page::Revision
            | Page::Reflog
            | Page::Blame
            | Page::RevisionFile => Box::new(std::iter::empty()),
            Page::Processes => Box::new(self.process_ids().map(|id| {
                let selected = self.process_selected == Some(id);
                let summary = self.process_summary(id).unwrap_or_default();
                (
                    format!("{} {summary}", if selected { '>' } else { ' ' }),
                    selected,
                )
            })),
            Page::Process(id) => match self.engine.record(id) {
                Some(record) => Box::new(record_lines(record).map(|text| (text, false))),
                None => Box::new(std::iter::once((
                    if self.engine.progress().any(|p| p.operation == id) {
                        "The worker owns capture; available output is retained when it completes."
                    } else {
                        "No retained output for this operation identity."
                    }
                    .into(),
                    false,
                ))),
            },
        }
    }
}

fn record_lines(record: &Record) -> impl Iterator<Item = String> + '_ {
    std::iter::once("Recorded child argv (display only):".into())
        .chain(
            record
                .argv
                .iter()
                .flat_map(|arg| DiagnosticLines::new(arg.as_encoded_bytes())),
        )
        .chain(std::iter::once("Diagnostic:".into()))
        .chain(DiagnosticLines::new(
            record.diagnostic.as_deref().unwrap_or("none").as_bytes(),
        ))
        .chain(std::iter::once("stderr:".into()))
        .chain(DiagnosticLines::new(&record.stderr))
        .chain(std::iter::once("stdout:".into()))
        .chain(DiagnosticLines::new(&record.stdout))
        .chain(
            record
                .output_truncated
                .then_some("TRUNCATED: retained diagnostic prefix reached its byte cap".into()),
        )
}

impl Session {
    pub(super) fn open_git_processes(&mut self) -> Action {
        if !self.git.visible && self.layout_active() {
            return self.message(
                "Source layout is in progress; retry process inspection after it completes",
            );
        }
        if !self.git.visible || matches!(self.git.page, Page::Status | Page::Diff) {
            self.git.process_origin = self.git.visible.then_some(self.git.page);
            let selected = self.git.process_ids().next();
            self.git.process_selected = selected;
            self.git.process_top = 0;
        }
        self.git.visible = true;
        self.git.page = Page::Processes;
        self.git.state.menu = false;
        self.git.painted = None;
        self.cancel_pointer_capture();
        self.cancel_terminal_pointer();
        self.input.set_git(true);
        self.message(
            "Git process inspection; q returns, C-g cancels only the selected live operation",
        )
    }
    pub(super) fn show_git_process(&mut self) -> Action {
        if !self.git.visible || !matches!(self.git.page, Page::Processes | Page::Process(_)) {
            return self.message("Select an operation in git-processes first");
        }
        let Some(id) = self.git.process_selected else {
            return self.message("No Git operation is selected");
        };
        if !self.git.process_ids().any(|current| current == id) {
            return self
                .message("Selected Git record expired; choose another operation explicitly");
        }
        self.git.page = Page::Process(id);
        self.git.detail_top = 0;
        self.git.state.menu = false;
        Action::Redraw
    }
    pub(super) fn cancel_git_process(&mut self) -> Action {
        let id = match self.git.page {
            Page::Processes if self.git.visible => self.git.process_selected,
            Page::Process(id) if self.git.visible => Some(id),
            _ => None,
        };
        let Some(id) = id else {
            return self.message("Select an owned operation in git-processes before cancellation");
        };
        match self.git.engine.cancel(id) {
            Ok(()) => self.message(&format!(
                "Cancellation requested for {id:?}; cleanup remains owned"
            )),
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn select_git_process(&mut self, index: usize) -> Action {
        let id = self.git.process_ids().nth(index);
        if let Some(id) = id {
            self.git.process_selected = Some(id);
            self.git.process_top = self
                .git
                .process_top
                .min(index)
                .max(index.saturating_sub(self.git_extent().saturating_sub(1)));
        }
        Action::Redraw
    }
    pub(super) fn move_git_process(&mut self, distance: usize, backward: bool) -> Action {
        let index = self
            .git
            .process_ids()
            .position(|id| Some(id) == self.git.process_selected)
            .unwrap_or(0);
        let next = if backward {
            index.saturating_sub(distance)
        } else {
            index
                .saturating_add(distance)
                .min(self.git.process_ids().count().saturating_sub(1))
        };
        self.select_git_process(next)
    }
    pub(super) fn scroll_git_process(
        &mut self,
        _id: OperationId,
        distance: usize,
        backward: bool,
    ) -> Action {
        let count = self.git.process_rows().count();
        self.git.detail_top = if backward {
            self.git.detail_top.saturating_sub(distance)
        } else {
            self.git
                .detail_top
                .saturating_add(distance)
                .min(count.saturating_sub(self.git_extent()))
        };
        Action::Redraw
    }
}
