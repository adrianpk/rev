use super::*;
use crate::git::{Conflict, ConflictRole, Update};

pub(super) struct Inspection {
    pub request: Option<OperationId>,
    pub detail: Option<Arc<Conflict>>,
    pub selected: Option<PathBuf>,
    pub top: usize,
    _admission: crate::git::budget::Lease,
}
impl Inspection {
    pub(super) fn new(engine: &Repositories) -> std::io::Result<Self> {
        Ok(Self {
            request: None,
            detail: None,
            selected: None,
            top: 0,
            _admission: engine
                .source_admission(8192)
                .map_err(std::io::Error::other)?,
        })
    }
}
impl Work {
    pub(super) fn conflict_path(&self) -> Option<&std::path::Path> {
        let sequence = self
            .state
            .context
            .and_then(|id| self.engine.context(id))?
            .sequence
            .as_ref()?;
        self.conflicts
            .selected
            .as_deref()
            .filter(|p| sequence.conflicts.iter().any(|c| c == p))
            .or_else(|| sequence.conflicts.first().map(PathBuf::as_path))
    }
    fn conflict_lines(&self, width: usize) -> impl Iterator<Item = String> + '_ {
        let detail = self.conflicts.detail.as_ref();
        let header = (0..5).map(move |row| {
            let Some(c) = detail else { return if row == 0 { "Git Conflict: checking exact index sides".into() } else { String::new() }; };
            match row {
                0 => format!("Git Conflict: {}", label(c.change.path.as_os_str().as_encoded_bytes())),
                1 => format!("Kind: {} ({}/{}) / {:?}", c.kind(), char::from(c.change.index), char::from(c.change.worktree), c.completeness),
                2 => format!("Actual operation: {:?}; index stages retain their exact Git meanings", c.sequence.family),
                3 => "Read-only captured sides; Return visits the canonical editable saved-result file".into(),
                _ => "Editing or removing markers never stages or continues the sequence".into(),
            }
        });
        let sides = ConflictRole::ALL.into_iter().flat_map(move |role| {
            let side = detail.and_then(|c| c.side(role));
            (0..side.map_or(2, |s| 3 + if s.text { s.lines.len() } else { 0 })).map(move |row| {
                if row == 0 { return format!("{} / index stage {} / {}", role.label(), role.stage(), role.meaning(detail.and_then(|c| c.sequence.family))); }
                let Some(s) = side else { return "Missing stage (deletion or structural conflict); no bytes inferred".into(); };
                match row {
                    1 => format!("Object {} / mode {:06o}", s.object.hex(), s.mode),
                    2 => format!("{} bytes / {:?} / {}", s.bytes.len(), s.completeness, if s.binary { "binary; text actions unavailable" } else if s.text { "complete UTF-8 text" } else { "unsupported mode, encoding or partial content; text actions unavailable" }),
                    _ => label(&s.bytes[s.lines[row-3].clone()]),
                }
            })
        });
        header.chain(sides).flat_map(move |text| Wrapped {
            text,
            offset: Some(0),
            width: width.max(1),
        })
    }
    pub(super) fn conflict_row(&self, row: usize, width: usize) -> (String, bool) {
        (
            self.conflict_lines(width)
                .nth(self.conflicts.top + row)
                .unwrap_or_default(),
            false,
        )
    }
}
impl Session {
    pub(super) fn open_git_conflict(&mut self) -> Action {
        if self.git.visible && self.git.page == Page::Conflict {
            return self.visit_git_conflict_result();
        }
        if !self.git.visible || self.git.page != Page::Sequence {
            return self
                .message("Open git-conflicts and select one exact conflict before inspection");
        }
        let Some(sequence) = self
            .git
            .state
            .context
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.sequence.clone())
        else {
            return self.message("Git-owned sequence state is still checking");
        };
        let Some(path) = self.git.conflict_path().map(std::path::Path::to_path_buf) else {
            return self.message("No exact unmerged path is available");
        };
        match self.git.engine.inspect_conflict(sequence, &path) {
            Ok(id) => {
                self.git.conflicts.request = Some(id);
                self.git.painted = None;
                self.message(
                    "Checking exact conflict stages, operation roles and bounded side content",
                )
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn publish_git_conflict(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.conflicts.request = None;
        if update.published
            && let Some(detail) = update
                .context
                .and_then(|id| self.git.engine.context(id))
                .and_then(|c| c.conflict.clone())
        {
            self.git.conflicts.detail = Some(detail);
            self.git.conflicts.top = 0;
            if self.git.visible
                && self.git.page == Page::Sequence
                && self.pending.is_none()
                && self.input.focus() == Focus::Editing
            {
                self.git.page = Page::Conflict;
                self.message("Checked conflict sides retained as read-only inspection; Return visits editable result");
            }
        } else if self.git.visible && self.pending.is_none() && self.input.focus() == Focus::Editing
        {
            self.message(
                "Conflict inspection failed or became stale; refresh actual Git sequence state",
            );
        }
        self.git.painted = None;
    }
    pub(super) fn move_git_conflict(&mut self, distance: usize, backward: bool) -> Action {
        let count = self
            .git
            .conflict_lines(usize::from(self.size.width))
            .count();
        self.git.conflicts.top = if backward {
            self.git.conflicts.top.saturating_sub(distance)
        } else {
            self.git
                .conflicts
                .top
                .saturating_add(distance)
                .min(count.saturating_sub(self.git_extent()))
        };
        self.git.painted = None;
        Action::Redraw
    }
    pub(super) fn visit_git_conflict_result(&mut self) -> Action {
        let Some(detail) = self
            .git
            .conflicts
            .detail
            .as_ref()
            .filter(|_| self.git.visible && self.git.page == Page::Conflict)
        else {
            return self.message("Inspect one exact conflict first");
        };
        let Some(root) = detail.sequence.refs.observation.identity.worktree.as_ref() else {
            return self.message("Conflict has no working-tree result");
        };
        let path = root.join(&detail.change.path);
        let Ok(actual) = std::fs::canonicalize(&path) else {
            return self.message("Conflict result is missing; no file is created by inspection");
        };
        if !actual.starts_with(root)
            || !std::fs::symlink_metadata(&path)
                .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
        {
            return self
                .message("Conflict result is not a canonical ordinary file inside this worktree");
        }
        if let Err(e) = self.load_file(&actual) {
            return self.error(&e.to_string());
        }
        self.git.visible = false;
        self.git.painted = None;
        self.input.set_git(false);
        self.terminal_source_focus();
        self.sync_input_context();
        Action::Redraw
    }
}
