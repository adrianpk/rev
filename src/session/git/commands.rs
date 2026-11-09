use super::*;

impl Session {
    pub(super) fn dispatch_git_command(&mut self, command: GitCommand) -> Action {
        self.git_integration_command(command)
    }

    fn git_integration_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::Merge => self.confirm_git_integration(crate::git::IntegrationMode::Merge),
            GitCommand::MergeFastForward => {
                self.confirm_git_integration(crate::git::IntegrationMode::FastForward)
            }
            GitCommand::MergeNoFastForward => {
                self.confirm_git_integration(crate::git::IntegrationMode::NoFastForward)
            }
            GitCommand::Rebase => self.confirm_git_integration(crate::git::IntegrationMode::Rebase),
            GitCommand::Conflicts => self.open_git_sequence(),
            GitCommand::OperationStatus => self.open_git_sequence(),
            GitCommand::RebaseAutosquash => self.prepare_git_autosquash(),
            command => self.git_bisect_command(command),
        }
    }

    fn git_bisect_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::BisectStart => self.prompt_git_bisect(),
            GitCommand::BisectGood => self.request_git_bisect(crate::git::BisectAction::Good),
            GitCommand::BisectBad => self.request_git_bisect(crate::git::BisectAction::Bad),
            GitCommand::BisectSkip => self.request_git_bisect(crate::git::BisectAction::Skip),
            GitCommand::BisectReset => self.request_git_bisect(crate::git::BisectAction::Reset),
            command => self.git_conflicts_command(command),
        }
    }

    fn git_conflicts_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::ConflictUseBase => self.prepare_git_resolution(
                crate::git::ResolutionAction::Use(crate::git::ConflictRole::Base),
            ),
            GitCommand::ConflictUseCurrent => self.prepare_git_resolution(
                crate::git::ResolutionAction::Use(crate::git::ConflictRole::Current),
            ),
            GitCommand::ConflictUseApplied => self.prepare_git_resolution(
                crate::git::ResolutionAction::Use(crate::git::ConflictRole::Applied),
            ),
            GitCommand::ConflictDelete => {
                self.prepare_git_resolution(crate::git::ResolutionAction::Delete)
            }
            GitCommand::ConflictStage => {
                self.prepare_git_resolution(crate::git::ResolutionAction::Stage)
            }
            GitCommand::ConflictExplain => self.explain_git_conflict(),
            GitCommand::ConflictVisit => self.open_git_conflict(),
            command => self.git_worktrees_command(command),
        }
    }

    fn git_worktrees_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::WorktreeList => self.open_git_worktrees(),
            GitCommand::WorktreeVisit => self.visit_git_worktree(),
            GitCommand::WorktreeCreate => {
                self.prompt_git_worktree(repository::worktrees::Intent::Create)
            }
            GitCommand::WorktreeLock => {
                self.prompt_git_worktree(repository::worktrees::Intent::Lock)
            }
            GitCommand::WorktreeUnlock => {
                self.prompt_git_worktree(repository::worktrees::Intent::Unlock)
            }
            GitCommand::WorktreePrune => {
                self.prompt_git_worktree(repository::worktrees::Intent::Prune)
            }
            GitCommand::WorktreeRemove => {
                self.prompt_git_worktree(repository::worktrees::Intent::Remove)
            }
            command => self.git_stashes_command(command),
        }
    }

    fn git_stashes_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::StashList => self.open_git_stashes(),
            GitCommand::StashShow => self.show_git_stash(),
            GitCommand::StashCreate => self.prompt_git_stash(repository::Intent::Create),
            GitCommand::StashApply => self.prompt_git_stash(repository::Intent::Apply),
            GitCommand::StashPop => self.prompt_git_stash(repository::Intent::Pop),
            GitCommand::StashBranch => self.prompt_git_stash(repository::Intent::Branch),
            GitCommand::StashDrop => self.prompt_git_stash(repository::Intent::Drop),
            command => self.git_merge_control_command(command),
        }
    }

    fn git_merge_control_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::MergeContinue => self.request_git_control(
                crate::git::SequenceControl::Continue,
                Some(crate::git::SequenceFamily::Merge),
            ),
            GitCommand::MergeAbort => self.request_git_control(
                crate::git::SequenceControl::Abort,
                Some(crate::git::SequenceFamily::Merge),
            ),
            GitCommand::RebaseContinue => self.request_git_control(
                crate::git::SequenceControl::Continue,
                Some(crate::git::SequenceFamily::Rebase),
            ),
            GitCommand::RebaseSkip => self.request_git_control(
                crate::git::SequenceControl::Skip,
                Some(crate::git::SequenceFamily::Rebase),
            ),
            GitCommand::RebaseAbort => self.request_git_control(
                crate::git::SequenceControl::Abort,
                Some(crate::git::SequenceFamily::Rebase),
            ),
            command => self.git_application_control_command(command),
        }
    }

    fn git_application_control_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::CherryPickContinue => self.request_git_control(
                crate::git::SequenceControl::Continue,
                Some(crate::git::SequenceFamily::CherryPick),
            ),
            GitCommand::CherryPickSkip => self.request_git_control(
                crate::git::SequenceControl::Skip,
                Some(crate::git::SequenceFamily::CherryPick),
            ),
            GitCommand::CherryPickAbort => self.request_git_control(
                crate::git::SequenceControl::Abort,
                Some(crate::git::SequenceFamily::CherryPick),
            ),
            GitCommand::RevertContinue => self.request_git_control(
                crate::git::SequenceControl::Continue,
                Some(crate::git::SequenceFamily::Revert),
            ),
            GitCommand::RevertSkip => self.request_git_control(
                crate::git::SequenceControl::Skip,
                Some(crate::git::SequenceFamily::Revert),
            ),
            GitCommand::RevertAbort => self.request_git_control(
                crate::git::SequenceControl::Abort,
                Some(crate::git::SequenceFamily::Revert),
            ),
            GitCommand::OperationContinue => {
                self.request_git_control(crate::git::SequenceControl::Continue, None)
            }
            GitCommand::OperationSkip => {
                self.request_git_control(crate::git::SequenceControl::Skip, None)
            }
            GitCommand::OperationAbort => {
                self.request_git_control(crate::git::SequenceControl::Abort, None)
            }
            command => self.git_refs_command(command),
        }
    }

    fn git_refs_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::TagCreate => self.prompt_git_tag(),
            GitCommand::TagDelete => self.confirm_git_tag_delete(),
            GitCommand::Push => self.prepare_git_push(),
            GitCommand::Fetch => self.fetch_git_remote(),
            GitCommand::Refs => self.open_git_refs(),
            GitCommand::BranchCreate => self.prompt_git_branch(refs::Intent::Create),
            GitCommand::BranchSwitch => self.prompt_git_branch(refs::Intent::Switch),
            GitCommand::BranchRename => self.prompt_git_branch(refs::Intent::Rename),
            GitCommand::BranchDelete => self.prompt_git_branch(refs::Intent::Delete),
            command => self.git_application_command(command),
        }
    }

    fn git_application_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::ShowRange => self.show_git_range(),
            GitCommand::CherryPick => {
                self.prepare_git_application(crate::git::ApplyKind::CherryPick)
            }
            GitCommand::Revert => self.prepare_git_application(crate::git::ApplyKind::Revert),
            GitCommand::SequenceMark => self.mark_git_revision(true, false, false),
            GitCommand::SequenceUnmark => self.mark_git_revision(true, true, false),
            command => self.git_history_command(command),
        }
    }

    fn git_history_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::Reflog => self.open_git_reflog(),
            GitCommand::Blame => self.open_git_blame(),
            GitCommand::Log => self.open_git_history(),
            GitCommand::Show => self.show_git_revision(),
            GitCommand::LogNextPage => self.continue_git_history(true),
            command => self.git_creation_command(command),
        }
    }

    fn git_creation_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::Clone => self.compose_git_family(crate::git::Family::Clone),
            GitCommand::CloneShallow => self.compose_git_family(crate::git::Family::CloneShallow),
            GitCommand::CloneBare => self.compose_git_family(crate::git::Family::CloneBare),
            GitCommand::CloneMirror => self.compose_git_family(crate::git::Family::CloneMirror),
            GitCommand::Init => self.compose_git_family(crate::git::Family::Init),
            GitCommand::Ignore => self.visit_git_ignore(false),
            GitCommand::IgnoreExclude => self.visit_git_ignore(true),
            command => self.git_submodule_command(command),
        }
    }

    fn git_submodule_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::SubmoduleAdd => self.compose_git_family(crate::git::Family::SubmoduleAdd),
            GitCommand::SubmoduleList => self.compose_git_family(crate::git::Family::SubmoduleList),
            GitCommand::SubmoduleUpdate => {
                self.compose_git_family(crate::git::Family::SubmoduleUpdate)
            }
            GitCommand::SubmoduleSync => self.compose_git_family(crate::git::Family::SubmoduleSync),
            GitCommand::SubmoduleDeinit => {
                self.compose_git_family(crate::git::Family::SubmoduleDeinit)
            }

            command => self.git_subtree_command(command),
        }
    }

    fn git_subtree_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::SubtreeAdd => self.compose_git_family(crate::git::Family::SubtreeAdd),
            GitCommand::SubtreeMerge => self.compose_git_family(crate::git::Family::SubtreeMerge),
            GitCommand::SubtreePull => self.compose_git_family(crate::git::Family::SubtreePull),
            GitCommand::SubtreePush => self.compose_git_family(crate::git::Family::SubtreePush),
            GitCommand::SubtreeSplit => self.compose_git_family(crate::git::Family::SubtreeSplit),
            command => self.git_notes_command(command),
        }
    }

    fn git_notes_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::NotesAdd => self.compose_git_family(crate::git::Family::NotesAdd),
            GitCommand::NotesRemove => self.compose_git_family(crate::git::Family::NotesRemove),
            GitCommand::NotesPrune => self.compose_git_family(crate::git::Family::NotesPrune),
            command => self.git_sparse_command(command),
        }
    }

    fn git_sparse_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::SparseEnable => self.compose_git_family(crate::git::Family::SparseEnable),
            GitCommand::SparseDisable => self.compose_git_family(crate::git::Family::SparseDisable),
            GitCommand::SparseReapply => self.compose_git_family(crate::git::Family::SparseReapply),
            GitCommand::SparseSet => self.compose_git_family(crate::git::Family::SparseSet),
            GitCommand::SparseAdd => self.compose_git_family(crate::git::Family::SparseAdd),
            command => self.git_output_command(command),
        }
    }

    fn git_output_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::FormatPatch => self.compose_git_family(crate::git::Family::FormatPatch),
            GitCommand::SaveDiff => self.compose_git_family(crate::git::Family::SaveDiff),
            GitCommand::ApplyPatch => self.compose_git_family(crate::git::Family::ApplyPatch),
            GitCommand::ApplyMailbox => self.compose_git_family(crate::git::Family::ApplyMailbox),
            GitCommand::RequestPull => self.compose_git_family(crate::git::Family::RequestPull),
            command => self.git_direct_command(command),
        }
    }

    fn git_direct_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::RunCommand => self.compose_git_family(crate::git::Family::RunCommand),
            GitCommand::RunShellCommand => {
                self.compose_git_family(crate::git::Family::RunShellCommand)
            }
            command => self.git_commit_command(command),
        }
    }

    fn git_commit_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::Commit => self.compose_git_commit(),
            GitCommand::CommitAmend => self.prepare_git_amend(),
            GitCommand::CommitFixup => self.prompt_git_fixup(),
            GitCommand::CommitSubmit | GitCommand::CommitCancel => {
                self.message("No commit composer is active")
            }
            command => self.git_status_command(command),
        }
    }

    fn git_status_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::Status => self.open_git(),
            GitCommand::Quit => self.quit_git(),
            GitCommand::Refresh => self.refresh_git(),
            GitCommand::ToggleSection => self.toggle_git_section(),
            GitCommand::Diff => self.open_git_diff(comparisons::Mode::Detail),
            GitCommand::VisitFile => self.visit_git_file(),
            GitCommand::Discard => self.discard_git(),
            command => self.git_index_command(command),
        }
    }

    fn git_index_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::Stage => self.change_git_index(crate::git::IndexDirection::Stage),
            GitCommand::Unstage => self.change_git_index(crate::git::IndexDirection::Unstage),
            GitCommand::Mark => self.mark_git_page(),
            GitCommand::Unmark => self.unmark_git_page(false),
            GitCommand::UnmarkAll => self.unmark_git_page(true),
            command => self.git_processes_command(command),
        }
    }

    fn git_processes_command(&mut self, command: GitCommand) -> Action {
        match command {
            GitCommand::Dispatch => {
                if !self.git.visible {
                    self.message("git-dispatch requires a Git presentation")
                } else {
                    self.git.state.menu = true;
                    Action::Redraw
                }
            }
            GitCommand::Processes => self.open_git_processes(),
            GitCommand::ProcessShow => self.show_git_process(),
            GitCommand::ProcessCancel => self.cancel_git_process(),
            _ => unreachable!("Git command dispatched by operation family"),
        }
    }

    fn mark_git_page(&mut self) -> Action {
        match self.git.page {
            Page::Refs => self.mark_git_push(false),
            Page::History => self.mark_git_revision(false, false, false),
            _ => self.mark_git(),
        }
    }
    fn unmark_git_page(&mut self, all: bool) -> Action {
        if self.git.page == Page::History {
            self.mark_git_revision(false, true, all)
        } else if self.git.page == Page::Refs && !all {
            self.mark_git_push(true)
        } else {
            self.unmark_git(all)
        }
    }

    fn cancel_git_presentation_input(&mut self) -> Action {
        if let Some(action) = self.cancel_git_discard() {
            return action;
        }
        if self.cancel_quit() {
            return self.message("Quit canceled; Git work retained");
        }
        self.pending = None;
        self.input.cancel();
        self.git.state.menu = false;
        self.message("Git input canceled; owned work retained")
    }

    pub(super) fn git_presentation_command(&mut self, command: Command) -> Action {
        match command {
            Command::Quit => self.quit(),
            Command::GenerateNewBuffer => self.generate_new_buffer(),
            Command::OpenDirectory => self.open_directory(),
            Command::ContextClose => self.close_context(),
            Command::DashboardOpen => self.dashboard_open(),
            command => self.git_project_command(command),
        }
    }

    fn git_project_command(&mut self, command: Command) -> Action {
        match command {
            Command::ProjectCreate => self.project_create(),
            Command::ProjectPromote => self.project_promote(),
            Command::ProjectSwitch | Command::ProjectList => self.project_list(),
            command => self.git_project_management_command(command),
        }
    }

    fn git_project_management_command(&mut self, command: Command) -> Action {
        match command {
            command @ (Command::ProjectRename
            | Command::ProjectForget
            | Command::ProjectSetRoot
            | Command::ProjectPromoteToWorkspace
            | Command::ProjectDuplicateAsWorkspace) => self.manage_context(command),
            command => self.git_workspace_command(command),
        }
    }

    fn git_workspace_command(&mut self, command: Command) -> Action {
        match command {
            Command::WorkspaceCreate => self.workspace_create(),
            Command::WorkspaceOpen | Command::WorkspaceList => self.workspace_list(),
            Command::WorkspaceAddDirectory => self.workspace_add_directory(),
            Command::WorkspaceAddProject => self.workspace_add_project(),
            Command::WorkspaceRemoveRoot => self.workspace_remove_root(),
            command @ (Command::WorkspaceRename
            | Command::WorkspaceForget
            | Command::WorkspaceSetRootPath) => self.manage_context(command),
            command => self.git_navigation_command(command),
        }
    }

    fn git_navigation_command(&mut self, command: Command) -> Action {
        match command {
            Command::ExecuteExtended => self.extended(),
            Command::KeyboardQuit => self.cancel_git_presentation_input(),
            Command::Scroll(Scroll::NextLine) => self.move_git(1, false),
            Command::Scroll(Scroll::PreviousLine) => self.move_git(1, true),
            Command::Scroll(Scroll::PageForward) => self.move_git(self.git_extent(), false),
            Command::Scroll(Scroll::PageBackward) => self.move_git(self.git_extent(), true),
            Command::Motion(Motion::BufferStart) => self.move_git(usize::MAX, true),
            Command::Motion(Motion::BufferEnd) => self.move_git(usize::MAX, false),
            _ => self.message("Git presentation is read-only; ? lists available actions"),
        }
    }
}
