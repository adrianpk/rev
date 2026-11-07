#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitCommand {
    Clone,
    CloneShallow,
    CloneBare,
    CloneMirror,
    Init,
    Ignore,
    IgnoreExclude,
    SubmoduleAdd,
    SubmoduleList,
    SubmoduleUpdate,
    SubmoduleSync,
    SubmoduleDeinit,
    SubtreeAdd,
    SubtreeMerge,
    SubtreePull,
    SubtreePush,
    SubtreeSplit,
    NotesAdd,
    NotesRemove,
    NotesPrune,
    SparseEnable,
    SparseDisable,
    SparseReapply,
    SparseSet,
    SparseAdd,
    FormatPatch,
    SaveDiff,
    RequestPull,
    ApplyPatch,
    ApplyMailbox,
    RunCommand,
    RunShellCommand,

    WorktreeList,
    WorktreeCreate,
    WorktreeVisit,
    WorktreeLock,
    WorktreeUnlock,
    WorktreePrune,
    WorktreeRemove,

    StashList,
    StashCreate,
    StashShow,
    StashApply,
    StashPop,
    StashBranch,
    StashDrop,

    Status,
    Refs,
    Merge,
    MergeFastForward,
    MergeNoFastForward,
    Rebase,
    OperationStatus,
    Conflicts,
    ConflictVisit,
    ConflictExplain,
    ConflictUseBase,
    ConflictUseCurrent,
    ConflictUseApplied,
    ConflictDelete,
    ConflictStage,
    MergeContinue,
    MergeAbort,
    RebaseAutosquash,
    RebaseContinue,
    RebaseSkip,
    RebaseAbort,
    OperationContinue,
    OperationSkip,
    OperationAbort,

    Fetch,
    Push,
    TagCreate,
    TagDelete,

    BranchCreate,
    BranchSwitch,
    BranchRename,
    BranchDelete,

    Log,
    Show,
    LogNextPage,
    ShowRange,
    BisectStart,
    BisectGood,
    BisectBad,
    BisectSkip,
    BisectReset,
    CherryPick,
    CherryPickContinue,
    CherryPickSkip,
    CherryPickAbort,
    RevertContinue,
    RevertSkip,
    RevertAbort,

    Revert,
    SequenceMark,
    SequenceUnmark,
    Reflog,
    Blame,
    Commit,
    CommitAmend,
    CommitFixup,
    CommitSubmit,
    CommitCancel,
    Quit,
    Refresh,
    ToggleSection,
    Diff,
    VisitFile,
    Discard,
    Mark,
    Unmark,
    UnmarkAll,
    Stage,
    Unstage,
    Dispatch,
    Processes,
    ProcessShow,
    ProcessCancel,
}

impl GitCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Clone => "git-clone",
            Self::CloneShallow => "git-clone-shallow",
            Self::CloneBare => "git-clone-bare",
            Self::CloneMirror => "git-clone-mirror",
            Self::Init => "git-init",
            Self::Ignore => "git-ignore",
            Self::IgnoreExclude => "git-ignore-exclude",
            Self::SubmoduleAdd => "git-submodule-add",
            Self::SubmoduleList => "git-submodule-list",
            Self::SubmoduleUpdate => "git-submodule-update",
            Self::SubmoduleSync => "git-submodule-sync",
            Self::SubmoduleDeinit => "git-submodule-deinit",
            Self::SubtreeAdd => "git-subtree-add",
            Self::SubtreeMerge => "git-subtree-merge",
            Self::SubtreePull => "git-subtree-pull",
            Self::SubtreePush => "git-subtree-push",
            Self::SubtreeSplit => "git-subtree-split",
            Self::NotesAdd => "git-notes-add",
            Self::NotesRemove => "git-notes-remove",
            Self::NotesPrune => "git-notes-prune",
            Self::SparseEnable => "git-sparse-checkout-enable",
            Self::SparseDisable => "git-sparse-checkout-disable",
            Self::SparseReapply => "git-sparse-checkout-reapply",
            Self::SparseSet => "git-sparse-checkout-set",
            Self::SparseAdd => "git-sparse-checkout-add",
            Self::FormatPatch => "git-format-patch",
            Self::SaveDiff => "git-save-diff",
            Self::RequestPull => "git-request-pull",
            Self::ApplyPatch => "git-apply-patch",
            Self::ApplyMailbox => "git-apply-mailbox",
            Self::RunCommand => "git-run-command",
            Self::RunShellCommand => "git-run-shell-command",

            Self::WorktreeList => "git-worktree-list",
            Self::WorktreeCreate => "git-worktree-create",
            Self::WorktreeVisit => "git-worktree-visit",
            Self::WorktreeLock => "git-worktree-lock",
            Self::WorktreeUnlock => "git-worktree-unlock",
            Self::WorktreePrune => "git-worktree-prune",
            Self::WorktreeRemove => "git-worktree-remove",

            Self::StashList => "git-stash-list",
            Self::StashCreate => "git-stash-create",
            Self::StashShow => "git-stash-show",
            Self::StashApply => "git-stash-apply",
            Self::StashPop => "git-stash-pop",
            Self::StashBranch => "git-stash-branch",
            Self::StashDrop => "git-stash-drop",

            Self::Merge => "git-merge",
            Self::MergeFastForward => "git-merge-fast-forward",
            Self::MergeNoFastForward => "git-merge-no-fast-forward",
            Self::Rebase => "git-rebase",
            Self::MergeContinue => "git-merge-continue",
            Self::MergeAbort => "git-merge-abort",
            Self::RebaseAutosquash => "git-rebase-autosquash",
            Self::RebaseContinue => "git-rebase-continue",
            Self::RebaseSkip => "git-rebase-skip",
            Self::RebaseAbort => "git-rebase-abort",
            Self::OperationContinue => "git-operation-continue",
            Self::OperationSkip => "git-operation-skip",
            Self::OperationAbort => "git-operation-abort",
            Self::Conflicts => "git-conflicts",
            Self::ConflictUseBase => "git-conflict-use-base",
            Self::ConflictUseCurrent => "git-conflict-use-current",
            Self::ConflictUseApplied => "git-conflict-use-applied",
            Self::ConflictDelete => "git-conflict-delete",
            Self::ConflictStage => "git-conflict-stage",
            Self::ConflictExplain => "git-conflict-explain",
            Self::ConflictVisit => "git-conflict-visit",
            Self::OperationStatus => "git-operation-status",
            Self::TagCreate => "git-tag-create",
            Self::TagDelete => "git-tag-delete",

            Self::Push => "git-push",
            Self::Fetch => "git-fetch",
            Self::Refs => "git-refs",
            Self::BranchCreate => "git-branch-create",
            Self::BranchSwitch => "git-branch-switch",
            Self::BranchRename => "git-branch-rename",
            Self::BranchDelete => "git-branch-delete",

            Self::ShowRange => "git-show-range",
            Self::CherryPickContinue => "git-cherry-pick-continue",
            Self::CherryPickSkip => "git-cherry-pick-skip",
            Self::CherryPickAbort => "git-cherry-pick-abort",
            Self::RevertContinue => "git-revert-continue",
            Self::RevertSkip => "git-revert-skip",
            Self::RevertAbort => "git-revert-abort",
            Self::BisectStart => "git-bisect-start",
            Self::BisectGood => "git-bisect-good",
            Self::BisectBad => "git-bisect-bad",
            Self::BisectSkip => "git-bisect-skip",
            Self::BisectReset => "git-bisect-reset",
            Self::CherryPick => "git-cherry-pick",
            Self::Revert => "git-revert",
            Self::SequenceMark => "git-sequence-mark",
            Self::SequenceUnmark => "git-sequence-unmark",
            Self::Reflog => "git-reflog",
            Self::Blame => "git-blame",
            Self::Log => "git-log",
            Self::Show => "git-show",
            Self::LogNextPage => "git-log-next-page",
            Self::Status => "git-status",
            Self::Commit => "git-commit",
            Self::CommitAmend => "git-commit-amend",
            Self::CommitFixup => "git-commit-fixup",
            Self::CommitSubmit => "git-commit-submit",
            Self::CommitCancel => "git-commit-cancel",
            Self::Quit => "git-quit",
            Self::Refresh => "git-refresh",
            Self::ToggleSection => "git-toggle-section",
            Self::Diff => "git-diff",
            Self::VisitFile => "git-visit-file",
            Self::Discard => "git-discard",
            Self::Mark => "git-mark",
            Self::Unmark => "git-unmark",
            Self::UnmarkAll => "git-unmark-all",
            Self::Stage => "git-stage",
            Self::Unstage => "git-unstage",
            Self::Dispatch => "git-dispatch",
            Self::Processes => "git-processes",
            Self::ProcessShow => "git-process-show",
            Self::ProcessCancel => "git-process-cancel",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Clone => {
                "Review literal clone arguments and an explicit absent destination, then verify the created regular repository."
            }
            Self::CloneShallow => {
                "Prepare a depth-one clone at an explicit destination and verify its shallow state and full HEAD."
            }
            Self::CloneBare => {
                "Review a bare clone destination and verify its separate Git-only repository identity."
            }
            Self::CloneMirror => {
                "Review a mirror clone destination and verify bare and mirror configuration without a later push."
            }
            Self::Init => {
                "Prepare initialization at an explicit absent destination and verify its unborn repository identity."
            }
            Self::Ignore => {
                "Visit the ordinary worktree .gitignore file without inserting a pattern or saving text."
            }
            Self::IgnoreExclude => {
                "Visit the ordinary common Git info/exclude file through its native visiting owner."
            }
            Self::SubmoduleAdd => {
                "Review a submodule metadata name, source URL and literal path, then verify its gitlink and checkout."
            }
            Self::SubmoduleList => {
                "Review recursive submodule status and inspect the actual bounded listing without repository effects."
            }
            Self::SubmoduleUpdate => {
                "Review one exact gitlink path and initialize or check out its indexed commit without forcing dirty files."
            }
            Self::SubmoduleSync => {
                "Review one declared submodule path and verify its registered URL against .gitmodules."
            }
            Self::SubmoduleDeinit => {
                "Review non-forced deinitialization of one checked clean submodule, preserving its parent gitlink."
            }
            Self::SubtreeAdd => {
                "Review an explicit prefix, source repository and pinned full branch object before importing a subtree."
            }
            Self::SubtreeMerge => {
                "Review a full local commit and subtree prefix, retaining actual merge/conflict state."
            }
            Self::SubtreePull => {
                "Pin one explicit source branch before fetching and merging into a reviewed subtree prefix."
            }
            Self::SubtreePush => {
                "Review one explicit endpoint and full destination branch, then verify its published subtree object and tree."
            }
            Self::SubtreeSplit => {
                "Review an exact commit and prefix, then inspect the full split object while preserving local HEAD and saved files."
            }
            Self::NotesAdd => {
                "Review a full notes ref, exact target object and literal message; verify the stored note bytes."
            }
            Self::NotesRemove => {
                "Review one existing note on a full object ID and remove only that note."
            }
            Self::NotesPrune => {
                "Review exact missing-object note targets and prune only those candidates."
            }
            Self::SparseEnable => {
                "Review enabling cone mode and verify actual tracked-file materialization."
            }
            Self::SparseDisable => {
                "Review disabling sparse checkout and verify that tracked files return."
            }
            Self::SparseReapply => {
                "Review reapplying the captured cone selection to a checked clean working tree."
            }
            Self::SparseSet => {
                "Review literal relative directories and replace the cone selection without forcing saved changes."
            }
            Self::SparseAdd => {
                "Review literal relative directories and add them to the captured cone selection."
            }
            Self::FormatPatch => {
                "Review an explicit local commit range and complete mailbox bytes, then save to one absent output file."
            }
            Self::SaveDiff => {
                "Review two local commits and a complete binary diff, then save to one absent output file."
            }
            Self::RequestPull => {
                "Review local commit anchors and one explicit published remote branch, then save request text without sending mail."
            }
            Self::ApplyPatch => {
                "Review one complete local patch file, then apply its sealed bytes to the checked index and working tree."
            }
            Self::ApplyMailbox => {
                "Review one complete local mailbox, then apply its decoded patches as checked commits."
            }
            Self::RunCommand => {
                "Compose literal Git argv and review executable, cwd and unknown postconditions before execution."
            }
            Self::RunShellCommand => {
                "Explicitly compose one shell command using the configured shell program and arguments, with exact review."
            }

            Self::WorktreeList => {
                "List linked worktrees, their branches, lock reasons and prune state."
            }
            Self::WorktreeCreate => {
                "Review a new linked worktree and new local branch at the exact observed HEAD, using a checked absent destination."
            }
            Self::WorktreeVisit => {
                "Open checked Git Status for the selected worktree identity while retaining native source buffers and windows."
            }
            Self::WorktreeLock => {
                "Review an exact linked-worktree path and explicit one-line lock reason."
            }
            Self::WorktreeUnlock => {
                "Review unlocking one exact linked-worktree record without changing its saved files."
            }
            Self::WorktreePrune => {
                "Review all captured missing unlocked worktree records and prune only while the complete registry remains unchanged."
            }
            Self::WorktreeRemove => "Review removal of a clean, unlocked linked worktree.",

            Self::StashList => {
                "Open a bounded exact stash list; subjects and ordinal labels never supply mutation authority."
            }
            Self::StashCreate => {
                "Review and stash saved tracked and untracked changes, excluding ignored files; protect affected unsaved buffers."
            }
            Self::StashShow => {
                "Inspect the selected full stash object as bounded read-only patch detail with explicit partial status."
            }
            Self::StashApply => {
                "Review and apply one full stash object to checked clean saved state while keeping its stash entry."
            }
            Self::StashPop => {
                "Apply one full stash object and drop only the revalidated captured entry after success; retain the stash on conflict."
            }
            Self::StashBranch => {
                "Review a new branch at the selected stash base, restore its full object and remove its checked entry only on success."
            }
            Self::StashDrop => {
                "Review removal of one exact stash entry without changing the saved working files or index."
            }

            Self::Merge => "Review merge with fast-forward allowed.",
            Self::MergeFastForward => {
                "Review one exact target and require fast-forward-only integration."
            }
            Self::MergeNoFastForward => "Review merge-commit policy.",
            Self::Rebase => "Review ordinary rebase onto the selected exact commit.",
            Self::MergeContinue => "Refresh and confirm continuation of a compatible merge.",
            Self::MergeAbort => "Refresh and confirm restoration of the original merge context.",
            Self::RebaseAutosquash => "Review the finite ordered source plan before autosquash.",
            Self::RebaseContinue => {
                "Refresh and confirm continuation after saved staged resolutions."
            }
            Self::RebaseSkip => "Refresh and confirm skipping the current compatible rebase step.",
            Self::RebaseAbort => "Refresh and confirm restoration of the original rebase context.",
            Self::OperationContinue => {
                "Review continuation against the fresh actual compatible family."
            }
            Self::OperationSkip => "Review skipping against the fresh actual compatible family.",
            Self::OperationAbort => {
                "Review abort against exact original lineage and fresh Git state."
            }
            Self::Conflicts => {
                "Inspect actual Git sequence state and exact selectable conflict paths."
            }
            Self::ConflictUseBase => {
                "Review complete Base replacement of only the checked working file."
            }
            Self::ConflictUseCurrent => {
                "Review complete operation-relative Current replacement without staging."
            }
            Self::ConflictUseApplied => {
                "Review complete operation-relative Applied replacement without staging."
            }
            Self::ConflictDelete => {
                "Review exact deletion from disk and index, leaving continue separate."
            }
            Self::ConflictStage => {
                "Review staging of captured saved text or absence; protect unsaved results and reject stale evidence."
            }
            Self::ConflictExplain => {
                "Ask AI to explain the selected conflict without applying a resolution."
            }
            Self::ConflictVisit => {
                "Inspect selected stages, then visit the canonical editable result."
            }
            Self::OperationStatus => "Inspect actual Git-owned sequence state and conflicts.",
            Self::TagCreate => {
                "Confirm a local lightweight tag or explicitly compose an annotated tag."
            }
            Self::TagDelete => "Confirm deletion of the exact local tag object.",

            Self::Push => {
                "Review one exact source branch and its configured upstream destination before publication."
            }
            Self::Fetch => "Fetch the selected configured remote without integration.",
            Self::Refs => "Inspect exact native repository refs.",
            Self::BranchCreate => "Confirm a new local branch at the selected full commit.",
            Self::BranchSwitch => "Confirm a checked clean local branch switch.",
            Self::BranchRename => "Confirm the exact local branch and new name.",
            Self::BranchDelete => "Confirm non-forced local deletion.",

            Self::ShowRange => {
                "Compare the marked exact base and selected endpoint in their explicit order."
            }
            Self::CherryPickContinue => {
                "Refresh and review continue of the exact retained cherry-pick request."
            }
            Self::CherryPickSkip => {
                "Refresh and review skip of the exact retained cherry-pick request."
            }
            Self::CherryPickAbort => {
                "Refresh and review abort of the exact retained cherry-pick request."
            }
            Self::RevertContinue => {
                "Refresh and review continue of the exact retained revert request."
            }
            Self::RevertSkip => "Refresh and review skip of the exact retained revert request.",
            Self::RevertAbort => "Refresh and review abort of the exact retained revert request.",
            Self::BisectStart => "Review good and bad revisions and start a manual bisect.",
            Self::BisectGood => {
                "Review the exact current candidate and record the user-supplied good verdict."
            }
            Self::BisectBad => {
                "Review the exact current candidate and record the user-supplied bad verdict."
            }
            Self::BisectSkip => {
                "Review and skip the current candidate; retain ambiguity when skipped commits prevent a unique first bad."
            }
            Self::BisectReset => {
                "Review and restore the checked original branch/object; reject missing or changed original lineage."
            }
            Self::CherryPick => {
                "Review exact ordered revisions and explicit merge mainlines before applying them."
            }
            Self::Revert => {
                "Review exact ordered revisions and explicit merge mainlines before reverting them."
            }
            Self::SequenceMark => {
                "Append an exact ordered revision without changing the range base."
            }
            Self::SequenceUnmark => {
                "Remove an ordered revision, retaining remaining order and the base."
            }
            Self::Reflog => "Inspect bounded exact HEAD reflog entries.",
            Self::Blame => "Attribute the selected or visiting source at checked HEAD.",
            Self::Log => "Open exact bounded ancestry; detached HEAD collects an explicit root.",
            Self::Show => {
                "Inspect full revision metadata, changed paths and exact parent comparison."
            }
            Self::LogNextPage => "Continue exact ancestry from its pinned root.",
            Self::Status => "Discover the current repository and open full-canvas native Status.",
            Self::Commit => {
                "Open native commit text directly; a subsequent c is literal message text."
            }
            Self::CommitAmend => {
                "Prepare exact observed HEAD composition, then explicitly review the replacement commit."
            }
            Self::CommitFixup => {
                "Resolve one exact target, compose its fixup body and review before submission."
            }
            Self::CommitSubmit => {
                "Submit the active native composer; prepared arguments require checked review before execution."
            }
            Self::CommitCancel => {
                "Cancel before process start; retain commit/tag drafts and restore the source owner."
            }
            Self::Quit => {
                "Dismiss the innermost menu or return to the retained editing context; owned work continues."
            }
            Self::Refresh => {
                "Request a replacement observation; retain confirmed Status on failure."
            }
            Self::ToggleSection => {
                "Collapse or expand the selected Status group, path comparison or semantic hunk."
            }
            Self::Diff => "Inspect the exact selected comparison as read-only detail.",
            Self::VisitFile => {
                "Visit the canonical current source, preserving an existing modified buffer."
            }
            Self::Discard => {
                "Confirm one tracked unstaged whole path from its index; protect dirty visiting buffers."
            }
            Self::Mark => {
                "Mark one exact current path or supported hunk independently of navigation."
            }
            Self::Unmark => "Remove the exact selected path or hunk mark.",
            Self::UnmarkAll => "Clear all retained path and hunk marks.",
            Self::Stage => {
                "Stage the exact selected path, supported text hunk or complete compatible marked group; unsaved source text is retained."
            }
            Self::Unstage => {
                "Unstage the selected path, hunk or marked group without changing working files."
            }
            Self::Dispatch => "Show the available native Git actions.",
            Self::Processes => {
                "Inspect active and recent owned operations without cancelling them."
            }
            Self::ProcessShow => {
                "Inspect the selected operation; the Return binding dispatches through git-visit-file."
            }
            Self::ProcessCancel => {
                "Cancel the selected live operation through its exact identity; menus and prompts keep first refusal."
            }
        }
    }
}
