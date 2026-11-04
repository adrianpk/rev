use super::*;
use crate::git::{Blame, Reflog, RevisionFile, RevisionPath};

#[derive(Clone, PartialEq, Eq)]
pub(in crate::session::git) enum DetailTarget {
    Parent(ObjectId),
    Path(PathBuf),
}
enum Row<'a> {
    Parent(ObjectId),
    Path(&'a RevisionPath),
    Text(&'a [u8]),
}
impl Row<'_> {
    fn target(&self) -> Option<DetailTarget> {
        match self {
            Self::Parent(p) => Some(DetailTarget::Parent(*p)),
            Self::Path(p) => Some(DetailTarget::Path(p.path.clone())),
            Self::Text(_) => None,
        }
    }
    fn matches(&self, target: &DetailTarget) -> bool {
        match (self, target) {
            (Self::Parent(a), DetailTarget::Parent(b)) => a == b,
            (Self::Path(a), DetailTarget::Path(b)) => a.path == *b,
            _ => false,
        }
    }
    fn text(&self) -> String {
        match self {
            Self::Parent(p) => format!("Parent: {}", p.hex()),
            Self::Path(p) => format!(
                "Path {} {}{}",
                label(&p.status),
                label(p.path.as_os_str().as_encoded_bytes()),
                p.old_path.as_ref().map_or(String::new(), |old| format!(
                    " <- {}",
                    label(old.as_os_str().as_encoded_bytes())
                ))
            ),
            Self::Text(t) => label(t),
        }
    }
}
fn rows(detail: &RevisionDetail) -> impl Iterator<Item = Row<'_>> {
    detail
        .history
        .revision(detail.object)
        .into_iter()
        .flat_map(|r| &r.parents)
        .copied()
        .map(Row::Parent)
        .chain(detail.paths.iter().map(Row::Path))
        .chain(
            detail
                .metadata
                .split_inclusive(|b| *b == b'\n')
                .flat_map(|l| l.chunks(2048))
                .map(Row::Text),
        )
        .chain(
            detail
                .lines
                .iter()
                .map(|line| Row::Text(&detail.patch[line.clone()])),
        )
}
impl Work {
    pub(in crate::session::git) fn reflog(&self) -> Option<&Arc<Reflog>> {
        self.state
            .context
            .and_then(|id| self.engine.context(id))?
            .reflog
            .as_ref()
    }
    pub(in crate::session::git) fn blame(&self) -> Option<&Arc<Blame>> {
        self.state
            .context
            .and_then(|id| self.engine.context(id))?
            .blame
            .as_ref()
    }
    pub(in crate::session::git) fn revision_file(&self) -> Option<&Arc<RevisionFile>> {
        self.state
            .context
            .and_then(|id| self.engine.context(id))?
            .revision_file
            .as_ref()
    }
    pub(in crate::session::git) fn inspection_token(&self) -> (Option<OperationId>, usize) {
        match self.page {
            Page::Reflog => (self.reflog().map(|r| r.id), self.ancestry.reflog_top),
            Page::Blame => (self.blame().map(|b| b.id), self.ancestry.blame_top),
            Page::RevisionFile => (self.revision_file().map(|f| f.id), self.ancestry.file_top),
            _ => (None, 0),
        }
    }
    fn inspection_state(&self) -> &str {
        if self.ancestry.request.is_some() {
            "CHECKING; confirmed detail retained"
        } else if self.ancestry.failed {
            "STALE: inspection failed; $ diagnostics"
        } else {
            "read-only"
        }
    }
    pub(in crate::session::git) fn inspection_row(&self, row: usize, width: u16) -> (String, bool) {
        match self.page {
            Page::Revision => {
                let Some(detail) = self.revision() else {
                    return (
                        if row == 0 {
                            "Git Revision: checking exact objects".into()
                        } else {
                            String::new()
                        },
                        false,
                    );
                };
                if row < 3 {
                    return (
                        match row {
                            0 => format!(
                                "Git {}: {}",
                                if detail.range { "Range" } else { "Revision" },
                                detail.object.hex()
                            ),
                            1 => format!(
                                "{}: {}",
                                if detail.range {
                                    "Range base"
                                } else {
                                    "Parent comparison"
                                },
                                detail.parent.map_or("empty tree".into(), |p| p.hex())
                            ),
                            _ => format!(
                                "{} / {:?}; Return opens selected parent/file",
                                self.inspection_state(),
                                detail.completeness
                            ),
                        },
                        false,
                    );
                }
                let index = self.ancestry.detail_top + row - 3;
                rows(detail).nth(index).map_or_else(
                    || (String::new(), false),
                    |item| {
                        let selected = index == self.ancestry.detail_cursor;
                        (
                            format!("{} {}", if selected { ">" } else { " " }, item.text()),
                            selected,
                        )
                    },
                )
            }
            Page::Reflog => {
                let Some(reflog) = self.reflog() else {
                    return (
                        if row == 0 {
                            "Git Reflog: checking entries".into()
                        } else {
                            String::new()
                        },
                        false,
                    );
                };
                if row < 3 {
                    return (
                        match row {
                            0 => "Git Reflog: HEAD entries / full immutable objects".into(),
                            1 => format!(
                                "{} entries / {:?}",
                                reflog.entries.len(),
                                reflog.completeness
                            ),
                            _ => format!(
                                "{}; selectors are display metadata",
                                self.inspection_state()
                            ),
                        },
                        false,
                    );
                }
                reflog
                    .entries
                    .get(self.ancestry.reflog_top + row - 3)
                    .map_or_else(
                        || (String::new(), false),
                        |e| {
                            let selected = self.ancestry.reflog_selected.as_ref().is_some_and(
                                |(id, subject)| *id == e.object && *subject == e.subject,
                            );
                            (
                                format!(
                                    "{} {} {} {} {}",
                                    if selected { ">" } else { " " },
                                    e.object.hex(),
                                    label(&e.selector),
                                    label(&e.author),
                                    label(&e.subject)
                                ),
                                selected,
                            )
                        },
                    )
            }
            Page::Blame => {
                let Some(blame) = self.blame() else {
                    return (
                        if row == 0 {
                            "Git Blame: checking attribution".into()
                        } else {
                            String::new()
                        },
                        false,
                    );
                };
                if row < 3 {
                    return (
                        match row {
                            0 => format!(
                                "Git Blame: {}",
                                label(blame.path.as_os_str().as_encoded_bytes())
                            ),
                            1 => format!("HEAD: {}", blame.root.hex()),
                            _ => format!(
                                "{} / {:?}; v revision, Return historical source",
                                self.inspection_state(),
                                blame.completeness
                            ),
                        },
                        false,
                    );
                }
                blame
                    .lines
                    .iter()
                    .flat_map(|l| {
                        std::iter::once((l, l.text.as_slice()))
                            .filter(|(_, text)| text.is_empty())
                            .chain(l.text.chunks(2048).map(move |t| (l, t)))
                    })
                    .nth(self.ancestry.blame_top + row - 3)
                    .map_or_else(
                        || (String::new(), false),
                        |(l, text)| {
                            let selected = self.ancestry.blame_selected == Some(l.final_line);
                            (
                                format!(
                                    "{} {} {}:{} {} {}",
                                    if selected { ">" } else { " " },
                                    l.object.hex(),
                                    label(l.path.as_os_str().as_encoded_bytes()),
                                    l.original_line,
                                    label(&l.author),
                                    label(text)
                                ),
                                selected,
                            )
                        },
                    )
            }
            Page::RevisionFile => {
                let Some(file) = self.revision_file() else {
                    return (
                        if row == 0 {
                            "Git Historical File: checking blob".into()
                        } else {
                            String::new()
                        },
                        false,
                    );
                };
                if row < 3 {
                    return (
                        match row {
                            0 => format!(
                                "Git Historical File: {}",
                                label(file.path.as_os_str().as_encoded_bytes())
                            ),
                            1 => format!("Object: {}", file.object.hex()),
                            _ => format!(
                                "{} / {:?}{} / {} bytes",
                                self.inspection_state(),
                                file.completeness,
                                if file.binary {
                                    " / binary bytes escaped"
                                } else {
                                    ""
                                },
                                file.bytes.len()
                            ),
                        },
                        false,
                    );
                }
                (
                    file_rows(file, width)
                        .nth(self.ancestry.file_top + row - 3)
                        .unwrap_or_default(),
                    false,
                )
            }
            _ => (String::new(), false),
        }
    }
}

impl Session {
    pub(in crate::session::git) fn open_git_reflog(&mut self) -> Action {
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Open Git Status before Reflog");
        };
        let result = self.git.engine.reflog(context);
        self.start_git_history(result, Page::Reflog)
    }
    pub(in crate::session::git) fn open_git_blame(&mut self) -> Action {
        if !self.git.visible || self.git.page != Page::Status {
            return self.message("Select an exact current source in Git Status");
        }
        let Some(observation) = self.git.observation() else {
            return self.message("Git repository is still checking");
        };
        let Some(root) = observation.identity.worktree.as_ref() else {
            return self.message("Blame requires a current working tree");
        };
        let path = match self.git.state.selected.as_ref() {
            Some(Target::Path(_, p) | Target::Hunk(_, p, _)) => Some(p.clone()),
            _ => self
                .buffer()
                .ok()
                .and_then(|b| b.file())
                .and_then(|f| f.path().strip_prefix(root).ok())
                .map(PathBuf::from),
        };
        let Some(path) = path else {
            return self.message("Select an exact repository-backed source for blame");
        };
        let result = self.git.engine.blame(observation.context, &path);
        self.start_git_history(result, Page::Blame)
    }
    pub(in crate::session::git) fn show_git_inspection(&mut self) -> Action {
        if self.git.ancestry.request.is_some() || self.git.ancestry.failed {
            return self.message("Refresh this inspection before selecting an exact object");
        }
        let result = match self.git.page {
            Page::Reflog => {
                let Some(reflog) = self.git.reflog().cloned() else {
                    return Action::Wait;
                };
                let Some((object, subject)) = self.git.ancestry.reflog_selected.as_ref() else {
                    return self.message("No reflog entry selected");
                };
                if !reflog
                    .entries
                    .iter()
                    .any(|e| e.object == *object && e.subject == *subject)
                {
                    return self.message("Reflog entry is no longer admitted");
                }
                self.git.engine.show_reflog(reflog, *object)
            }
            Page::Blame => {
                let Some(blame) = self.git.blame().cloned() else {
                    return Action::Wait;
                };
                let Some(line) = self.git.ancestry.blame_selected else {
                    return self.message("No attributed source line selected");
                };
                self.git.engine.inspect_blame(blame, line, false)
            }
            Page::Revision => {
                let Some(detail) = self.git.revision().cloned() else {
                    return Action::Wait;
                };
                if let Some(DetailTarget::Parent(parent)) = self.git.ancestry.detail_selected {
                    let result = self.git.engine.compare_parent(detail, parent);
                    return self.start_git_history(result, Page::Revision);
                }
                if let Err(e) = self.input.argument(
                    "Full parent object ID: ",
                    &detail.parent.map_or(String::new(), |p| p.hex()),
                ) {
                    return self.error(&e.to_string());
                }
                self.pending = Some(super::super::super::Pending::GitHistoryParent(detail));
                return self.message("Choose one exact parent from the revision metadata");
            }
            _ => return self.message("Select a revision, reflog entry or blamed line"),
        };
        self.git.ancestry.revision_origin = self.git.page;
        self.git.ancestry.detail_selected = None;
        self.git.ancestry.detail_cursor = 0;
        self.git.ancestry.detail_top = 0;
        self.start_git_history(result, Page::Revision)
    }
    pub(in crate::session::git) fn toggle_git_inspection(&mut self) -> Action {
        if self.git.ancestry.request.is_some() || self.git.ancestry.failed {
            return self.message("Refresh this inspection before opening retained detail");
        }
        let result = match self.git.page {
            Page::Reflog => return self.show_git_inspection(),
            Page::Blame => {
                let Some(blame) = self.git.blame().cloned() else {
                    return Action::Wait;
                };
                let Some(line) = self.git.ancestry.blame_selected else {
                    return self.message("No attributed source line selected");
                };
                self.git.engine.inspect_blame(blame, line, true)
            }
            Page::Revision => {
                let Some(detail) = self.git.revision().cloned() else {
                    return Action::Wait;
                };
                match self.git.ancestry.detail_selected.as_ref() {
                    Some(DetailTarget::Parent(parent)) => {
                        let result = self.git.engine.compare_parent(detail, *parent);
                        return self.start_git_history(result, Page::Revision);
                    }
                    Some(DetailTarget::Path(path)) => self.git.engine.revision_file(detail, path),
                    _ => return self.message("Select an exact parent or changed path"),
                }
            }
            Page::RevisionFile => return self.quit_git(),
            _ => return self.message("No read-only inspection detail selected"),
        };
        self.git.ancestry.file_origin = self.git.page;
        self.git.ancestry.file_top = 0;
        self.start_git_history(result, Page::RevisionFile)
    }
    pub(in crate::session::git) fn refresh_git_inspection(&mut self) -> Action {
        match self.git.page {
            Page::Reflog => self.open_git_reflog(),
            Page::Blame => {
                let Some(blame) = self.git.blame().cloned() else {
                    return Action::Wait;
                };
                let result = self
                    .git
                    .engine
                    .blame(blame.observation.context, &blame.path);
                self.start_git_history(result, Page::Blame)
            }
            Page::Revision => {
                let Some(detail) = self.git.revision().cloned() else {
                    return Action::Wait;
                };
                let result = self.git.engine.refresh_revision(detail);
                self.start_git_history(result, Page::Revision)
            }
            Page::RevisionFile => {
                let Some(file) = self.git.revision_file().cloned() else {
                    return Action::Wait;
                };
                let result = self.git.engine.refresh_revision_file(file);
                self.start_git_history(result, Page::RevisionFile)
            }
            _ => Action::Wait,
        }
    }
    pub(in crate::session::git) fn reconcile_git_inspection(&mut self, page: Option<Page>) {
        match page {
            Some(Page::Reflog) => {
                let selected = self.git.ancestry.reflog_selected.as_ref();
                let value = self
                    .git
                    .reflog()
                    .and_then(|r| {
                        r.entries
                            .iter()
                            .find(|e| {
                                selected.is_some_and(|(id, subject)| {
                                    *id == e.object && *subject == e.subject
                                })
                            })
                            .or_else(|| r.entries.first())
                    })
                    .map(|e| (e.object, e.subject.clone()));
                self.git.ancestry.reflog_selected = value;
            }
            Some(Page::Blame) => {
                let selected = self.git.ancestry.blame_selected;
                self.git.ancestry.blame_selected = self
                    .git
                    .blame()
                    .and_then(|b| {
                        b.lines
                            .iter()
                            .find(|l| Some(l.final_line) == selected)
                            .or_else(|| b.lines.first())
                    })
                    .map(|l| l.final_line);
            }
            Some(Page::Revision) => {
                let selected = self.git.ancestry.detail_selected.as_ref();
                let value = self
                    .git
                    .revision()
                    .and_then(|d| rows(d).position(|r| selected.is_some_and(|s| r.matches(s))))
                    .unwrap_or(0);
                self.set_git_detail_cursor(value);
            }
            Some(Page::RevisionFile) => {
                let width = self.size.width;
                self.git.ancestry.file_top = self
                    .git
                    .revision_file()
                    .map_or(0, |f| file_line_offset(f, width, f.original_line));
            }
            _ => {}
        }
    }
    fn set_git_detail_cursor(&mut self, index: usize) {
        let target = self
            .git
            .revision()
            .and_then(|d| rows(d).nth(index).and_then(|r| r.target()));
        self.git.ancestry.detail_cursor = index;
        self.git.ancestry.detail_selected = target;
        self.git.ancestry.detail_top = self
            .git
            .ancestry
            .detail_top
            .min(index)
            .max(index.saturating_sub(self.git_extent().saturating_sub(1)));
    }
    pub(in crate::session::git) fn move_git_inspection(
        &mut self,
        distance: usize,
        backward: bool,
    ) -> Action {
        let (index, count) = match self.git.page {
            Page::Revision => (
                self.git.ancestry.detail_cursor,
                self.git.revision().map_or(0, |d| rows(d).count()),
            ),
            Page::Reflog => (
                self.git
                    .reflog()
                    .and_then(|r| {
                        r.entries.iter().position(|e| {
                            self.git
                                .ancestry
                                .reflog_selected
                                .as_ref()
                                .is_some_and(|(id, s)| *id == e.object && *s == e.subject)
                        })
                    })
                    .unwrap_or(0),
                self.git.reflog().map_or(0, |r| r.entries.len()),
            ),
            Page::Blame => (
                self.git
                    .blame()
                    .and_then(|b| {
                        b.lines
                            .iter()
                            .position(|l| Some(l.final_line) == self.git.ancestry.blame_selected)
                    })
                    .unwrap_or(0),
                self.git.blame().map_or(0, |b| b.lines.len()),
            ),
            Page::RevisionFile => (
                self.git.ancestry.file_top,
                self.git
                    .revision_file()
                    .map_or(0, |f| file_rows(f, self.size.width).count()),
            ),
            _ => return Action::Wait,
        };
        let next = if backward {
            index.saturating_sub(distance)
        } else {
            index.saturating_add(distance).min(count.saturating_sub(1))
        };
        match self.git.page {
            Page::Revision => self.set_git_detail_cursor(next),
            Page::Reflog => {
                self.git.ancestry.reflog_selected = self
                    .git
                    .reflog()
                    .and_then(|r| r.entries.get(next).map(|e| (e.object, e.subject.clone())));
                self.git.ancestry.reflog_top = self
                    .git
                    .ancestry
                    .reflog_top
                    .min(next)
                    .max(next.saturating_sub(self.git_extent().saturating_sub(1)));
            }
            Page::Blame => {
                let line = self
                    .git
                    .blame()
                    .and_then(|b| b.lines.get(next))
                    .map(|l| l.final_line);
                self.git.ancestry.blame_selected = line;
                self.git.ancestry.blame_top = self.git.blame().map_or(0, |b| {
                    b.lines[..next]
                        .iter()
                        .map(|l| l.text.len().div_ceil(2048).max(1))
                        .sum()
                });
            }
            Page::RevisionFile => {
                self.git.ancestry.file_top = next.min(count.saturating_sub(self.git_extent()))
            }
            _ => {}
        }
        Action::Redraw
    }
    pub(in crate::session::git) fn select_git_inspection(&mut self, row: usize) -> Action {
        match self.git.page {
            Page::Revision => {
                let index = self.git.ancestry.detail_top + row;
                if self
                    .git
                    .revision()
                    .is_some_and(|d| rows(d).nth(index).is_some())
                {
                    self.set_git_detail_cursor(index);
                }
            }
            Page::Reflog => {
                self.git.ancestry.reflog_selected = self
                    .git
                    .reflog()
                    .and_then(|r| r.entries.get(self.git.ancestry.reflog_top + row))
                    .map(|e| (e.object, e.subject.clone()));
            }
            Page::Blame => {
                self.git.ancestry.blame_selected = self.git.blame().and_then(|b| {
                    b.lines
                        .iter()
                        .flat_map(|l| {
                            std::iter::repeat_n(l.final_line, l.text.len().div_ceil(2048).max(1))
                        })
                        .nth(self.git.ancestry.blame_top + row)
                });
            }
            _ => {}
        }
        Action::Redraw
    }
    pub(in crate::session::git) fn visit_git_inspection_source(&mut self) -> Action {
        let (observation, path) = match self.git.page {
            Page::Revision => {
                let Some(detail) = self.git.revision() else {
                    return Action::Wait;
                };
                let Some(DetailTarget::Path(path)) = self.git.ancestry.detail_selected.as_ref()
                else {
                    return self.message("Select one exact changed path");
                };
                (detail.history.observation.clone(), path.clone())
            }
            Page::Blame => {
                let Some(blame) = self.git.blame() else {
                    return Action::Wait;
                };
                (blame.observation.clone(), blame.path.clone())
            }
            Page::RevisionFile => {
                let Some(file) = self.git.revision_file() else {
                    return Action::Wait;
                };
                (
                    file.observation.clone(),
                    file.current_path.as_ref().unwrap_or(&file.path).clone(),
                )
            }
            _ => return self.message("Select an exact source path"),
        };
        let Some(root) = observation.identity.worktree.as_ref() else {
            return self.message("No current working-tree source exists");
        };
        let path = root.join(path);
        if !std::fs::metadata(&path).is_ok_and(|m| m.is_file()) {
            return self.message("Historical path has no current source file");
        }
        if let Err(e) = self.load_file(&path) {
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

fn file_rows(file: &RevisionFile, width: u16) -> impl Iterator<Item = String> + '_ {
    file.lines.iter().flat_map(move |line| Wrapped {
        text: label(&file.bytes[line.clone()]),
        at: Some(0),
        width: usize::from(width.max(1)),
    })
}
fn file_line_offset(file: &RevisionFile, width: u16, line: usize) -> usize {
    let end = file
        .bytes
        .split_inclusive(|b| *b == b'\n')
        .take(line.saturating_sub(1))
        .map(|p| p.len())
        .sum::<usize>();
    file.lines
        .iter()
        .take_while(|r| r.start < end)
        .map(|r| {
            Wrapped {
                text: label(&file.bytes[r.clone()]),
                at: Some(0),
                width: usize::from(width.max(1)),
            }
            .count()
        })
        .sum()
}
struct Wrapped {
    text: String,
    at: Option<usize>,
    width: usize,
}
impl Iterator for Wrapped {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        use unicode_width::UnicodeWidthChar;
        let start = self.at?;
        let mut cells = 0;
        let mut end = self.text.len();
        for (at, c) in self.text[start..].char_indices() {
            let width = c.width().unwrap_or(0);
            if cells + width > self.width && at > 0 {
                end = start + at;
                break;
            }
            cells += width;
        }
        self.at = (end < self.text.len()).then_some(end);
        Some(self.text[start..end].to_owned())
    }
}
