use super::*;

impl Session {
    pub(in crate::session) fn git_input(&mut self, event: &Event) -> Option<Action> {
        self.git_input_contexts();
        if let Some(action) = self.git_ref_review_input(event) {
            return Some(action);
        }
        if self.git.composer.is_some() && matches!(event, Event::Mouse(_)) {
            return Some(Action::Wait);
        }
        if !self.git.visible || self.input.focus() != Focus::Editing {
            return None;
        }
        if self.git_process_interrupt(event) {
            return Some(self.cancel_git_process());
        }
        match event {
            Event::Paste(_) => Some(self.message("Git presentation is read-only")),
            Event::Mouse(mouse) => Some(self.git_mouse_input(*mouse)),
            _ => None,
        }
    }
    fn git_input_contexts(&mut self) {
        self.input
            .set_git_worktree(self.git.visible && self.git.page == Page::Worktree);
        self.input
            .set_git_stash(self.git.visible && self.git.page == Page::Stash);
        self.input
            .set_git_conflict(self.git.visible && self.git.page == Page::Conflict);
        self.input
            .set_git_sequence(self.git.visible && self.git.page == Page::Sequence);
        self.input
            .set_git_refs(self.git.visible && self.git.page == Page::Refs);
        self.input
            .set_git_history(self.git.visible && self.git.page == Page::History);
        self.input.set_git_inspect(
            self.git.visible
                && matches!(
                    self.git.page,
                    Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
                ),
        );
    }
    fn git_process_interrupt(&self, event: &Event) -> bool {
        matches!(self.git.page, Page::Processes | Page::Process(_))
            && !self.git.state.menu
            && self.pending.is_none()
            && self.input.prefix().is_none()
            && crate::input::Input::interrupt(event)
    }
    fn git_mouse_input(&mut self, mouse: crossterm::event::MouseEvent) -> Action {
        if !self.git_mouse_admitted(mouse) {
            return Action::Wait;
        }

        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            && !self
                .git
                .painted
                .as_ref()
                .is_some_and(|painted| painted.current(&self.git, self.size, self.input.focus()))
        {
            return self.message("Git view changed; redraw before selecting a target");
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.move_git(self.buffers.snapshot().settings().mouse.scroll_lines, true)
            }
            MouseEventKind::ScrollDown => {
                self.move_git(self.buffers.snapshot().settings().mouse.scroll_lines, false)
            }
            MouseEventKind::Down(MouseButton::Left) if self.git_mouse_selectable(mouse) => {
                self.select_git_mouse_row(mouse.row)
            }
            _ => Action::Wait,
        }
    }
    fn git_mouse_admitted(&self, mouse: crossterm::event::MouseEvent) -> bool {
        self.mouse_enabled
            && !self.git.state.menu
            && mouse.column < self.size.width
            && mouse.row < self.size.height.saturating_sub(2)
    }
    fn git_mouse_selectable(&self, mouse: crossterm::event::MouseEvent) -> bool {
        mouse.modifiers.is_empty()
            && (mouse.row >= 3 || self.git.page == Page::Sequence)
            && mouse.row < self.size.height.saturating_sub(2)
            && mouse.column < self.size.width
            && !self.git.state.menu
    }
    fn select_git_mouse_row(&mut self, row: u16) -> Action {
        let index = usize::from(row - 3);
        match self.git.page {
            Page::Sequence => {
                self.select_git_sequence(self.git.refs.sequence_top + usize::from(row))
            }
            Page::Worktree => self.select_git_worktree(self.git.worktrees.top + index),
            Page::Stash => self.select_git_stash(self.git.repository.top + index),
            Page::Refs => self.select_git_ref(self.git.refs.top + index),
            Page::Processes => self.select_git_process(self.git.process_top + index),
            _ => self.select_git_mouse_detail(index),
        }
    }
    fn select_git_mouse_detail(&mut self, index: usize) -> Action {
        if matches!(
            self.git.page,
            Page::History | Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
        ) {
            return self.select_git_history(index);
        }
        if self.git.page == Page::Status {
            return self.select_git_row(self.git.state.top + index);
        }
        if self.git.page == Page::Diff {
            return self.select_git_diff(self.git.inspection.top + index);
        }
        Action::Wait
    }
}

impl Painted {
    fn current(&self, work: &Work, size: crate::display::Size, focus: Focus) -> bool {
        self.repository_current(work)
            && self.view_current(work, size, focus)
            && self.process_current(work)
            && self.comparison_current(work)
    }
    fn repository_current(&self, work: &Work) -> bool {
        self.refs == work.refs().map(|r| r.id)
            && self.conflict
                == (
                    work.conflicts.detail.as_ref().map(|c| c.id),
                    work.conflicts.top,
                )
            && self.context == work.state.context
            && self.generation == work.observation().map(|o| o.generation)
            && self.worktree == work.worktree_token()
            && self.stash == work.stash_token()
            && self.inspected == work.inspection_token()
            && self.history == work.history().map(|h| h.id)
            && self.revision == work.revision().map(|d| d.id)
    }
    fn view_current(&self, work: &Work, size: crate::display::Size, focus: Focus) -> bool {
        self.refs_top == work.refs.top
            && self.sequence_top == work.refs.sequence_top
            && self.history_top == work.ancestry.top
            && self.revision_top == work.ancestry.detail_top
            && self.top == work.state.top
            && self.collapsed == work.state.collapsed
            && self.size == size
            && self.focus == focus
            && self.menu == work.state.menu
            && self.page == work.page
    }
    fn process_current(&self, work: &Work) -> bool {
        self.processes == work.process_ids().collect::<Vec<_>>()
            && self.process_top == work.process_top
    }
    fn comparison_current(&self, work: &Work) -> bool {
        self.comparison == work.inspection.detail.as_ref().map(|d| d.id)
            && self.comparison_top == work.inspection.top
            && self.expanded == work.inspection.expanded
            && self.expanded_hunk == work.inspection.expanded_hunk
    }
}
