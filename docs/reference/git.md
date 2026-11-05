# Native Git

Native Git uses the installed Git executable. It has no extension or Org
prerequisite. Loading startup preferences does not launch Git or discover a
repository. Missing Git leaves ordinary editing usable and reports an
unavailable Git observation when requested.

## Entry and navigation

`C-x g` opens `git-status` in both startup profiles. M-x and modal colon invoke
the same named commands. Repository selection follows the visiting file or
selected directory, with the launch directory as fallback. Linked worktrees
retain distinct worktree/Git directories and their common directory; bare,
unborn, detached and unavailable states are explicit.

| Context | Frequent controls | Exact inventory |
| --- | --- | --- |
| Status | g refresh, Tab expand, Return visit, s/u stage/unstage, k discard, */-/0 marks, c compose, h History, b Refs, $ Processes, ? actions, q return | [Status keys](bindings.md#git-defaults-and-availability) |
| Composer | Ordinary text; C-Enter or C-c C-c submit; C-g or C-c C-k cancel | [Composer keys](bindings.md#git-commit-defaults-and-availability) |
| History | Tab revision, * range base, Return range, f next page, A amend, F fixup, a/U ordered marks, P/V cherry-pick/revert | [History keys](bindings.md#git-history-defaults-and-availability) |
| Refs | n/S/R/D branch create/switch/rename/delete, t/T local tag create/delete, & fetch, p push, =/i/I merge policies, e rebase | [Refs keys](bindings.md#git-refs-defaults-and-availability) |
| Operation / Conflict | Inspect actual sequence; Return visit conflict/result; 1/2/3 complete side, K delete, x stage | [Sequence keys](bindings.md#git-sequence-defaults-and-availability), [conflict keys](bindings.md#git-conflict-defaults-and-availability) |
| Stash / Worktree | Exact object/path selection with separately reviewed actions | [Stash keys](bindings.md#git-stash-defaults-and-availability), [worktree keys](bindings.md#git-worktree-defaults-and-availability) |

Letter keys belong to their context: q returns from navigation but inserts text
in a composer. Return visits/inspects in navigation and inserts a newline in a
composer; it does not grant mutation consent. Some reviews accept y, others
explicit composer submission. The displayed review and exact command reference
state which acceptance applies. Pointer targets are qualified by the current
painted observation, expansion, viewport and dimensions.

## Command families

The [complete Git command table](commands.md#native-git-status) owns exact names
and behavior. All controls remain callable by name, including actions without
a default chord. The contextual maps are configurable through `keys.git`,
`keys.git_commit`, `keys.git_history`, `keys.git_inspect`, `keys.git_refs`,
`keys.git_sequence`, `keys.git_conflict`, `keys.git_stash` and `keys.git_worktree`.
The [configuration sample](config.toml) contains their defaults.

[Review and commit](../how-to/review-and-commit.md) covers saved/index changes.
[Integrate changes](../how-to/integrate-git-changes.md) covers separate fetch,
reviewed push, integration, ordered revisions and conflicts. Reflog and blame
are read-only inspections. Stashes and linked worktrees retain full object/path
identities; ordinal labels and escaped display paths cannot authorize changes.

## Startup preferences and resource bounds

The [Git preference catalog](defaults.md#git-preferences) owns every `git.*`
key, range, relationship and origin; values are frozen at startup. Defaults
include 10-second observation, 60-second operation and 120-second transport
timeouts; 1,024 status paths, 256 refs, 32 remotes and four 128-entry history
pages; 1 MiB combined child output, 512 KiB detail, 64 KiB diagnostics and eight
process records within a 2 MiB aggregate. Invalid related values reject the
whole candidate and keep documented defaults with diagnostics.

Fixed admission allows four repository contexts and one active Git child in
each; direct arbitrary commands retain exclusive Git admission until cleanup.
Sixteen navigation contexts and 256 marks share a 32 MiB charged Git budget.
Workers have 1 MiB scratch. Fair stream service admits at most 64 KiB and 64
coalesced events per editor turn. These logical capacities are distinct from
whole-process RSS or hard filesystem latency promises.

Drafts admit 64 KiB; final argv admits 64 tokens including executable/prefix,
4,096 bytes per token and 64 KiB total. An explicit shell string admits 4,096
bytes. Cancellation uses a 500 ms graceful interval and two-second cleanup
deadline. Unresolved cleanup remains visible and owned. Quiet Git has no polling
timer or background fetch/history walk. Truncation, unavailable detail and
failed freshness remain explicit and cannot authorize a partial mutation.

## Outcomes

Processes retains actual exit, bounded output/diagnostics and checked outcomes,
including stopped Git sequences and unknown effects. Credentials and raw
transport/helper output remain private. Cancellation, timeout or exit zero
does not imply rollback or verified success. [Git ownership](../explanation/git-ownership.md)
explains source reconciliation and lifetime; the [Git learning path](../tutorials/user-guide/git/index.md)
provides a small local exercise.
