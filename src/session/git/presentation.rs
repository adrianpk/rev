use super::*;

impl Session {
    pub(in crate::session) fn git_frame(&mut self) -> Result<Frame, crate::view::Error> {
        if matches!(self.pending, Some(crate::session::Pending::GitRef(_))) {
            return self.git_ref_review_frame();
        }
        if self.git_commit_reviewing() {
            return self.git_commit_review_frame();
        }
        self.git.reconcile(self.git_extent());
        let popup = self.input.palette().and_then(|palette| {
            crate::panel::Composition::navigation(self.size, palette.requested_rows())
        });
        self.input.reveal_commands(
            popup
                .as_ref()
                .and_then(|c| c.auxiliary())
                .map_or(0, |p| p.content().height()),
        );
        let mut frame = self.git_rows_frame()?;
        if let Some(popup) = &popup {
            frame.git_popup(popup)?;
        }
        frame.message_kind(self.echo_kind);
        self.prompt_frame(&mut frame, popup.as_ref())?;
        self.git.painted = Some(Painted {
            worktree: self.git.worktree_token(),
            stash: self.git.stash_token(),
            context: self.git.state.context,
            generation: self.git.observation().map(|o| o.generation),
            top: self.git.state.top,
            collapsed: self.git.state.collapsed,
            size: self.size,
            focus: self.input.focus(),
            menu: self.git.state.menu,
            page: self.git.page,
            processes: self.git.process_ids().collect(),
            process_top: self.git.process_top,
            comparison: self.git.inspection.detail.as_ref().map(|d| d.id),
            comparison_top: self.git.inspection.top,
            history: self.git.history().map(|h| h.id),
            revision: self.git.revision().map(|d| d.id),
            history_top: self.git.ancestry.top,
            revision_top: self.git.ancestry.detail_top,
            inspected: self.git.inspection_token(),
            refs: self.git.refs().map(|r| r.id),
            refs_top: self.git.refs.top,
            conflict: (
                self.git.conflicts.detail.as_ref().map(|c| c.id),
                self.git.conflicts.top,
            ),
            sequence_top: self.git.refs.sequence_top,
            expanded: self.git.inspection.expanded,
            expanded_hunk: self.git.inspection.expanded_hunk,
        });
        Ok(frame)
    }
    fn git_rows_frame(&self) -> Result<Frame, crate::view::Error> {
        let admission = self
            .git
            .engine
            .presentation(self.size.width, self.size.height)?;
        let context = self
            .git
            .state
            .context
            .and_then(|id| self.git.engine.context(id));
        let observation = self.git.observation();
        let mut rows = self.git.items().skip(self.git.state.top);
        let mut comparison_rows = self
            .git
            .inspection
            .detail
            .as_ref()
            .into_iter()
            .flat_map(|d| d.lines.iter().map(move |line| (d.as_ref(), line)))
            .skip(self.git.inspection.top);
        let mut process_rows = self.git.process_rows().skip(match self.git.page {
            Page::Process(_) => self.git.detail_top,
            _ => self.git.process_top,
        });
        Frame::pending(self.buffer()?, self.size, self.buffers.palette())?.git(
            &self.echo,
            self.git.frame_title(),
            admission,
            |row| {
                if let Some(line) = self.git.presentation_line(row, self.size.width) {
                    return line;
                }
                match self.git.page {
                    Page::Diff => self.git.diff_frame_line(row, &mut comparison_rows),
                    Page::Status => {
                        self.git
                            .status_frame_line(row, observation, context, &mut rows)
                    }
                    _ => GitLine::new(
                        if row < 3 {
                            (self.git.process_header(row), false)
                        } else {
                            process_rows.next().unwrap_or_default()
                        },
                        row_role(row),
                    ),
                }
            },
        )
    }
}

fn row_role(row: usize) -> GitRole {
    match row {
        0 => GitRole::Heading,
        1 | 2 => GitRole::Metadata,
        _ => GitRole::Text,
    }
}

impl Work {
    fn frame_title(&self) -> &'static str {
        match self.page {
            Page::Worktree => {
                " Git Worktrees  c create  v Status visit  L lock  U unlock  P prune  k remove  q Status"
            }
            Page::Status => {
                " Git Status  c commit  Tab expand  Return visit  g refresh  $ processes  ? actions  q return"
            }
            Page::Conflict => {
                " Git Conflict  Return editable result  Up/Down scroll  g sequence refresh  q Operation"
            }
            Page::Sequence => " Git Operation  g refresh  $ processes  Up/Down scroll  q Status",
            Page::Stash => {
                " Git Stashes  c create  v inspect  a apply  p pop  b branch  k drop  g refresh  q Status"
            }
            Page::Refs => {
                " Git Refs  n branch  S switch  R rename  D delete  t/T tags  ? actions  g refresh  q Status"
            }
            Page::Diff => " Git Diff  g refresh  Return visit source  $ processes  q Status",
            _ => self.detail_title(),
        }
    }
    fn detail_title(&self) -> &'static str {
        match self.page {
            Page::History => {
                " Git History  Tab revision  * base  Return range  a/U ordered marks  f next page  q Status"
            }
            Page::Revision => {
                " Git Revision  Return parent/file  v parent review  g refresh  q return"
            }
            Page::Reflog => " Git Reflog  Return / v revision  g refresh  q Status",
            Page::Blame => " Git Blame  Return historical source  v revision  g refresh  q Status",
            Page::RevisionFile => {
                " Git Historical File  Up/Down scroll  git-visit-file current source  q return"
            }
            Page::Processes => " Git Processes  Return inspect  C-g cancel selected  q return",
            Page::Process(_) => " Git Process  Up/Down scroll  C-g cancel this operation  q return",
            _ => unreachable!("repository page title dispatched by its owner"),
        }
    }
    fn presentation_line(&self, row: usize, width: u16) -> Option<GitLine> {
        let line = if self.state.menu {
            Some(self.menu_row(row))
        } else {
            self.repository_frame_row(row, width)
                .or_else(|| self.history_frame_row(row, width))
        };
        line.map(|line| GitLine::new(line, row_role(row)))
    }
    fn menu_row(&self, row: usize) -> (String, bool) {
        if self.page == Page::Refs {
            return self.refs_menu(row);
        }
        (
            [
                "Git actions",
                "c  git-commit (native composer)",
                "g  git-refresh",
                "Tab  git-toggle-section",
                "Return  git-visit-file",
                "s / u  git-stage / git-unstage",
                "* / - / 0  mark / unmark / clear",
                "k / x d  git-discard (confirmed)",
                "M-x git-diff  selected comparison",
                "$  git-processes",
                "M-x / :  native commands",
                "q / C-g  return to Status",
            ]
            .get(row)
            .unwrap_or(&"")
            .to_string(),
            false,
        )
    }
    fn repository_frame_row(&self, row: usize, width: u16) -> Option<(String, bool)> {
        match self.page {
            Page::Worktree => Some(self.worktree_row(row, usize::from(width.max(1)))),
            Page::Stash => Some(self.stash_row(row, usize::from(width.max(1)))),
            Page::Refs => Some(self.refs_row(row, usize::from(width.max(1)))),
            Page::Conflict => Some(self.conflict_row(row, usize::from(width))),
            Page::Sequence => Some(self.sequence_row(row, usize::from(width))),
            _ => None,
        }
    }
    fn history_frame_row(&self, row: usize, width: u16) -> Option<(String, bool)> {
        match self.page {
            Page::History => Some(self.history_frame_line(row)),
            Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile => {
                Some(self.inspection_row(row, width))
            }
            _ => None,
        }
    }
    fn history_frame_line(&self, row: usize) -> (String, bool) {
        if row < 3 {
            return (self.history_header(row), false);
        }
        self.history()
            .and_then(|h| h.revisions().nth(self.ancestry.top + row - 3))
            .map_or_else(
                || (String::new(), false),
                |r| {
                    let selected = Some(r.object) == self.ancestry.selected;
                    (
                        format!(
                            "{} {}{} {} {}",
                            if selected { ">" } else { " " },
                            self.revision_mark_label(r.object),
                            r.object.hex(),
                            label(&r.authored),
                            label(&r.subject)
                        ),
                        selected,
                    )
                },
            )
    }
    fn diff_frame_line<'a>(
        &self,
        row: usize,
        rows: &mut impl Iterator<Item = (&'a crate::git::Diff, &'a crate::git::DiffLine)>,
    ) -> GitLine {
        if row < 3 {
            return GitLine::new((self.diff_header(row), false), row_role(row));
        }
        let Some((diff, line)) = rows.next() else {
            return GitLine::new((String::new(), false), row_role(row));
        };
        let selected = line.hunk.is_some() && line.hunk == self.inspection.selected;
        let marked = line.hunk.is_some_and(|id| {
            self.marks
                .contains(&Target::Hunk(diff.group, diff.change.path.clone(), id))
        });
        GitLine::new(
            (
                format!(
                    "{}{} {}",
                    if selected { ">" } else { " " },
                    if marked { "*" } else { " " },
                    comparisons::line_label(diff, line)
                ),
                selected,
            ),
            GitRole::Diff(line.kind),
        )
    }
    fn status_frame_line<'a>(
        &self,
        row: usize,
        observation: Option<&Observation>,
        context: Option<&crate::git::Context>,
        rows: &mut impl Iterator<Item = Item<'a>>,
    ) -> GitLine {
        match row {
            0 => GitLine::new((header(observation), false), GitRole::Heading),
            1 => GitLine::new((upstream_label(observation), false), GitRole::Metadata),
            2 => self.status_warning_line(observation, context),
            _ => self.status_path_line(row, observation, rows),
        }
    }
    fn status_warning_line(
        &self,
        observation: Option<&Observation>,
        context: Option<&crate::git::Context>,
    ) -> GitLine {
        let stale = context.is_some_and(|c| c.failure.is_some());
        let truncated = observation
            .and_then(|o| o.status.as_ref())
            .is_some_and(|s| s.completeness != crate::git::Completeness::Complete);
        let text = format!(
            "{}{}{}{}{}",
            self.status_progress_label(),
            if stale {
                "STALE: refresh failed; confirmed observation retained  "
            } else {
                ""
            },
            if self.inspection.expanded {
                self.diff_status()
            } else {
                String::new()
            },
            if truncated {
                "TRUNCATED: path admission reached  "
            } else {
                ""
            },
            self.marks_label(),
        );
        GitLine::new(
            (text, false),
            if stale || truncated {
                GitRole::Warning
            } else {
                GitRole::Metadata
            },
        )
    }
    fn status_progress_label(&self) -> &'static str {
        if self.state.request.is_some()
            || self.index_request.is_some()
            || self.discard_request.is_some()
        {
            "Checking / refreshing  "
        } else {
            ""
        }
    }
    fn marks_label(&self) -> String {
        if self.marks.targets.is_empty() {
            return String::new();
        }
        format!(
            "  {} marked{}; 0 clears",
            self.marks.targets.len(),
            if self
                .marks
                .targets
                .iter()
                .all(|t| self.engine.detail_current(t.diff()))
            {
                ""
            } else {
                " / STALE"
            }
        )
    }
    fn status_path_line<'a>(
        &self,
        row: usize,
        observation: Option<&Observation>,
        rows: &mut impl Iterator<Item = Item<'a>>,
    ) -> GitLine {
        let Some(status) = observation.and_then(|o| o.status.as_ref()) else {
            return missing_status_line(row, observation.is_some());
        };
        if status.paths.is_empty() {
            return GitLine::new(
                (
                    if row == 3 { "Working tree clean" } else { "" }.to_owned(),
                    false,
                ),
                GitRole::Text,
            );
        }
        let item = rows.next();
        self.status_item_line(item.as_ref())
    }
    fn status_item_line(&self, item: Option<&Item<'_>>) -> GitLine {
        let selected = item
            .as_ref()
            .is_some_and(|i| self.state.selected.as_ref().is_some_and(|t| i.matches(t)));
        let marked = item
            .as_ref()
            .is_some_and(|i| self.marks.contains(&i.target()));
        let marker = format!(
            "{}{}",
            if selected { ">" } else { " " },
            if marked { "*" } else { " " }
        );
        let mut role = GitRole::Text;
        let text = match item {
            Some(Item::Section(group, count)) => {
                role = GitRole::Group(*group);
                format!("{marker} {} ({count})", group.label())
            }
            Some(Item::Path(group, path)) => {
                role = GitRole::Group(*group);
                format!(
                    "{marker}   {}",
                    label(path.path.as_os_str().as_encoded_bytes())
                )
            }
            Some(Item::Detail(diff, line)) => {
                role = GitRole::Diff(line.kind);
                format!("{marker}     {}", comparisons::line_label(diff, line))
            }
            None => String::new(),
        };
        GitLine::new((text, selected), role)
    }
}

fn missing_status_line(row: usize, observed: bool) -> GitLine {
    let text = if row != 3 {
        ""
    } else if observed {
        "Bare repository: working-tree Status unavailable"
    } else {
        "Checking repository context"
    };
    GitLine::new((text.to_owned(), false), GitRole::Text)
}
fn upstream_label(observation: Option<&Observation>) -> String {
    observation
        .and_then(|o| o.status.as_ref())
        .and_then(|s| {
            s.upstream.as_ref().map(|u| {
                format!(
                    "Upstream: {}  ahead {}  behind {}",
                    label(u),
                    s.ahead.map_or("unavailable".into(), |n| n.to_string()),
                    s.behind.map_or("unavailable".into(), |n| n.to_string())
                )
            })
        })
        .unwrap_or_default()
}

fn header(observation: Option<&Observation>) -> String {
    let Some(observation) = observation else {
        return "Git: checking repository".into();
    };
    let identity = &observation.identity;
    let root = identity
        .worktree
        .as_deref()
        .unwrap_or(&identity.git_directory);
    let root = root.file_name().unwrap_or(root.as_os_str());
    let head = match observation.status.as_ref().map(|s| &s.head) {
        Some(Head::Unborn { branch }) => format!("{} (unborn)", label(branch)),
        Some(Head::Branch { name, object }) => format!("{} {}", label(name), &object.hex()[..12]),
        Some(Head::Detached { object }) => format!("detached {}", &object.hex()[..12]),
        None => "bare".into(),
    };
    format!("Git: {}  {head}", label(root.as_encoded_bytes()))
}
