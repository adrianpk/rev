# Integrate Git Changes

Use a repository with a checked current branch, clean index and working tree,
and saved visiting sources. Configure remotes and upstreams deliberately
before using the native transport actions.

## Fetch, then choose an integration policy

1. Open Status with `C-x g`, then b / `git-refs`.
2. Select a configured remote row and press `&` / `git-fetch`. Inspect the result in `$` / `git-processes` and refreshed tracking refs.
3. Select the exact commit-bearing ref to integrate. Choose `=` / `git-merge-fast-forward` for fast-forward only, i / `git-merge` to allow a merge, or I / `git-merge-no-fast-forward` to require a merge commit when there is work.
4. Review full source/current identities and policy. Accept with y, `C-Enter` or `C-c C-c`; cancel with n / `C-g`. Return does not accept.

For rebase, choose e / `git-rebase` on the exact target. `git-rebase-autosquash`
prepares an ordered pick/fixup/squash review with full IDs. Inspect that order
before accepting. No interactive external editor or executable todo command
is launched. Incompatible autostash, updateRefs or mergeOptions policy requires
an explicit configuration change; rev does not silently rewrite it.

## Resolve a stopped operation

Open `git-operation-status` to inspect Git's actual family, original/current
objects and conflict paths. Return inspects the selected conflict.

- Return in Conflict visits the canonical editable result. Edit and save it, then return to inspect the conflict again.
- `1`, `2` or `3` prepares complete Base, Current or Applied side replacement. Review the displayed operation-relative roles: during rebase, Current is the destination and Applied is the replayed commit. Replacement changes the saved file without staging it.
- x / `git-conflict-stage` reviews the exact saved result or saved absence. K / `git-conflict-delete` reviews deletion from disk and index. Unsaved visiting results block these mutations.

Only supported complete text/mode observations grant replacement or staging.
Binary, structural, submodule and partial cases remain visible for inspection.
Removing conflict markers does not stage or continue an operation. Optional
`git-conflict-explain` copies context to the configured AI owner for read-only
explanation; manual inspection and resolution remain usable without it.

After staging, invoke the matching `git-merge-continue`, `git-rebase-continue`,
`git-cherry-pick-continue` or `git-revert-continue` and accept its refreshed review.
Matching skip/abort commands are separate reviewed choices where supported.
`git-operation-continue`, `git-operation-skip` and `git-operation-abort` select
only a compatible actual sequence. Leaving the view or cancelling its process
does not abort Git-owned sequence state.

## Apply an ordered set or investigate a regression

In History, a / `git-sequence-mark` appends an exact revision to an ordered
cherry-pick/revert set; U removes a mark. P / `git-cherry-pick` and V /
`git-revert` review the complete order and required merge-parent choices before
execution. Completed prefixes and conflict stops remain actual repository state.
These marks are distinct from the `*` range base and Status marks.

Use `git-bisect-start` with explicit bad/good boundaries for manual bisect.
Inspect/test the checked candidate yourself, then choose `git-bisect-good`,
`git-bisect-bad` or `git-bisect-skip`. No test command runs automatically.
`git-bisect-reset` reviews restoration of the original branch/object; skipped
candidates may leave an ambiguous result.

## Push one reviewed branch

In Refs select one local branch and press p / `git-push`. Alternatively mark
one local branch with `*`, then navigate to its configured remote and invoke
`git-push`; `-` clears that separate source mark. Preparation requires a
consistent configured upstream. The review identifies the full source ref and
object, remote name and full destination ref. Only y accepts; n / `C-g`
cancels and Return does not submit.

The action sends one exact branch object without force. Local tag creation
does not grant tag publication. Mirror, expanded/multiple endpoint or conflicting
push policy is refused. Hooks, authentication, signing and receiver protection
remain Git-owned. A successful child is verified against the actual push
endpoint; an unavailable or differing observation stays unverified.

## Keep or stop owned work

q restores editing while started Git work stays owned. `$` / `git-processes`
reopens active/recent operations; select the actual active operation and use
`git-process-cancel` to request bounded termination. Application exit asks
before stopping active work; declining keeps the editor and operation running.
Inspect actual effects after cancellation, timeout or failure before deciding
whether to retry.

[Git reference](../reference/git.md) · [Git ownership](../explanation/git-ownership.md)
