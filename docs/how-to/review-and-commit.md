# Review and Commit Changes

Use an existing repository with installed Git. Save the source changes you
intend Git to read; retain or undo other unsaved edits deliberately. Git
Status observes saved files and the index.

## Stage an exact change

1. Open `C-x g` / `git-status` and refresh with g.
2. Select a path under Unstaged or Untracked. Tab expands a comparison; `git-diff` opens read-only detail.
3. Press s on a whole path or on a supported expanded text hunk. Select a Staged target and press u to unstage it.
4. Inspect the refreshed Staged and Unstaged groups before composing a commit.

For a group, mark with `*`, unmark with `-` and clear all with `0`. With marks
present, s/u applies to the checked group. Whole paths must support the same
direction. Hunk groups must contain nonoverlapping hunks from one exact path
and layer. Mixed, stale, duplicate or partial targets reject before launch.
Refresh does not retarget old marks; clear and select them again.

To discard one tracked working-file change, select its whole unstaged path
and press k. Review and accept with y, or cancel with n / `C-g`. This restores
the file from its index state. It does not change that index, discard a hunk
or marked group, or delete an untracked file. A modified visiting buffer
blocks discard. Removing text from a Git view never edits the source.

## Commit selected index content

Press c in Status, edit the message and submit with `C-c C-c` or `C-Enter`.
Return inserts a newline. `C-g` or `C-c C-k` cancels before execution and retains
the draft for this session. Check History and Processes after completion;
source saving, staging and committing remain separate actions.

For amend, open History, select the observed HEAD and press A /
`git-commit-amend`. For a fixup, select the exact target revision and press F /
`git-commit-fixup`. Their first explicit composer submission prepares a review;
a second explicit submission accepts it. Cancel a review with q / `C-g`.
Amend changes the checked HEAD; fixup creates a new commit and does not run
autosquash. Both preserve Git hooks, identity and signing policy.

## Inspect a failure before retrying

Open `$` / `git-processes`, select the operation and press Return for retained
facts. Distinguish failed child exit, rejected preconditions, stopped sequence
and unverified effects. Refresh Status and inspect actual index/HEAD before
retrying; cancellation does not undo effects that already happened.

See [Git controls](../reference/git.md) for exact keys and
[Git ownership](../explanation/git-ownership.md) for source and file authority.
