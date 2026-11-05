# Git Ownership

Git sees repository objects, the index and saved working files. A source buffer
can contain newer unsaved text. Keeping those authorities distinct lets you
inspect a repository without losing an edit or mistaking a display for a file.

## Saved, staged and live text

Status compares saved bytes with the index and HEAD. Saving publishes source
text to disk; staging changes the index; committing records its checked tree.
Each action is explicit. One path may have both staged and unstaged changes.
A generated diff, revision, blame result or process output is read-only data.
Visiting current source reuses its canonical native buffer and ordinary undo,
reload and save behavior.

Mutations that would overwrite a saved path protect modified visiting sources,
including canonical aliases. External file changes reconcile through the
existing automatic/ask policy. Git does not save an unsaved buffer, replace it
with generated output or grant a historical view source-edit authority.
Conflict stage roles depend on the actual operation; a side label alone is
not a universal statement about whose work it contains.

## Observations expire

A native selection carries the full path/object, comparison and repository
generation. Marks retain those identities independently of row positions.
Before a mutation, rev rechecks the relevant HEAD, index, file, configuration
and sequence facts. Changed evidence requires refresh and a new review.
Escaped labels, abbreviated IDs and list ordinals are presentation, not targets.

Complete, truncated and unavailable results have different meanings. A failed
refresh can retain the prior confirmed observation with failed freshness; it
cannot turn it into a current clean result. Bounds can keep a large diff or
binary/structural conflict available for inspection without granting a partial
edit. Checks reduce stale actions but do not lock out other Git clients.

## A process outlives its view

The Git presentation temporarily owns the canvas and input; source windows,
file-tree state and terminals remain retained. q restores a surviving editing
context. It does not stop a started Git child or abort a Git sequence. Each
repository owns at most one active Git child and its process group, streams,
deadlines and retained result. Processes remains available when Git is hidden.

Explicit process cancellation requests bounded termination. Application exit
asks before stopping active Git work; declining preserves both editor and job.
An unresolved cleanup obligation stays owned and visible. A stopped rebase,
cherry-pick, revert, merge or mailbox application is repository state and needs
its own compatible reviewed action; killing a child does not undo it.

## Results describe what was observed

Known operations check their concrete postconditions. An exit code cannot prove
that a remote accepted a branch, a hook preserved a requested tree or a failed
child had no effects. rev retains actual failure, in-progress and unverified
outcomes without automatic retry, reset or rollback. For arbitrary Git/shell
commands the repository effect remains unknown even after exit zero.

Installed Git owns hooks, identity, signing, transport helpers and repository
semantics. Native requests expose explicit argv and reviewed scope; Git's own
aliases and helpers retain their semantics. Shell parsing happens only in the
separately selected shell route. No source directive or startup preference
starts repository work by itself.

[Git reference](../reference/git.md) · [Review and commit](../how-to/review-and-commit.md)
