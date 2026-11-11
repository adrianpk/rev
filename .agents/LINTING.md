# Native Quality Checks

- `make lint`: formatting, strict all-target/all-feature workspace Clippy and
  complete first-party source policy through `scripts/check-quality.sh`.
- `make test-hooks`: real Git index, partial-staging and cache ownership cases.
- `make install-hooks`: explicit local activation after approved acceptance.

Both packages inherit the workspace lint policy. The SDK additionally checks
public error and panic documentation. `examples/source-check.rs` is the single
native source-policy owner; discovery includes every workspace package and Cargo
root and excludes foreign/generated code by ownership. Explicit `--scope PATH`
checks are for focused development and cannot replace final workspace acceptance.

Commit checks use the immutable staged candidate, never an unstaged correction.
Preserve index/worktree bytes, untracked files, ignored data and active builds.
Do not bypass hooks, autoformat, stash, clean caches or relax acceptance to make a
commit pass. Report failing checks against their exact candidate.

Use one Cargo build job, disabled incremental output and no dev/test debuginfo
for delivery checks. Keep ordinary build targets and hook caches exclusive to
their checkout. The controller owns integrated gate execution and hook activation
for an automated delivery set.

## Commit checks and installation

Run `make lint` and `make test-hooks` before explicit `make install-hooks`.
Installation sets repository-local `core.hooksPath=.githooks`. Git uses the hook
files in each linked worktree. An existing different hook configuration or a
non-sample default hook is a conflict: preserve it and resolve the installation
choice explicitly. Installation is idempotent for the same path.

The hook copies the active index, rejects unmerged entries, checks staged
whitespace and exports exact Git blob bytes at a stable private path. Checkout
filters, export-ignore rules and unstaged repairs cannot change those bytes.
Staged source, manifests, lockfiles, formatting/lint policy, attributes and tooling
run complete workspace formatting, Clippy and native source checks. Docs/data-only
commits use whitespace and index-integrity checks without invoking Cargo.
The working hook must match the staged hook. Stage the intended hook version
before committing a hook edit. Tracked symlinks and submodules are rejected;
this repository has neither, and their external content is not a staged snapshot.

Each worktree owns `rev-commit-check/` below its Git directory, with private
permissions, an owner marker and a nonblocking exclusive lock. The separate
`source/` export and `target/` cache never use the checkout's ordinary target.
A competing check fails with an ownership message; retry after its owner finishes.
Cancellation preserves the retained export/cache until the next exclusive owner. Matching blob bytes keep their source timestamps
so Cargo can reuse unchanged inputs.
No hook operation stages, stashes, resets, formats or cleans user files. The hook
also rejects source changes within its export or index changes during validation;
it never rolls back a newly staged candidate.

The cache uses fixed resource settings: one build job, no incremental output and
no dev/test debuginfo. A cold cache compiles dependencies; warm checks reuse that
worktree's cache. Timings and review evidence belong in the active slice report,
not a performance guarantee. Do not share mutable caches across worktrees.
