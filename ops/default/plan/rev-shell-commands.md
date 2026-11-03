# Native Shell Commands Delivery

Status: Approved
Date: 2026-11-03
Delivery set: rev-shell-commands
Slice strategy: behavior-first
Approved: owner, 2026-11-03; requested two specifications and starting the agent-selected first capability; this map is derived under that execution authority
Planning baseline: `ebec1fcc5c10e7d66e3e1ed2743dda370d25e91d`
Spec: [Shell Commands and Text Filters](../spec/editor-shell-commands.md)
Model: [Shell Commands Model](../spec/editor-shell-commands-model.md)
Tracker: [Shell Commands Tracker](../tracker/rev-shell-commands.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands --test configuration --test commands --test external_changes -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::ai::results -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`
Active slice: Slice 1 - One-shot commands

## Outcome and authority

Deliver SHL01-SHL08 in one autonomous Looper run through dev: explicit command
entry, bounded one-shot execution, exact stdin/stdout filters, checked native
edits, both-profile command interaction and native/PTY acceptance.

The owner delegated which of the two capabilities starts first. Shell commands
are selected for immediate text-processing value without a PTY emulator. This
authorizes the preparation commit/push, four recorded slice worktrees, one
canonical PR per slice, controller-owned Rebase + Fast-forward integration into
dev, the fixed final gate and documentation closure. Looper owns progress and
merge verification; do not request individual human slice merges.

Interactive terminal implementation, the broader optional extension protocol,
other open tickets, native AI API, main promotion, releases, mirrors, global
installation and website deployment are outside this run.

## Strategy and native ownership

Each slice exposes an independently passing behavior. Do not stage fake handlers,
disable an old assertion for convenience or add another command interpreter.
Actual shell interpretation belongs to the explicitly selected external shell.

Reuse native configuration, command/argument prompts, keymaps, buffer/view
identities, observed edits/history, plain-text output buffers, external-file
observation and native jobs/cancellation. Inspect the existing Org IO process
group, nonblocking streams, finite limits and cleanup helpers before adding
mechanisms. A genuinely shared low-level helper may move to an editor-level
owner; preserve all existing callers and their policy. Do not turn shell
commands into an Org adapter or give AI another tool.

Use safe Rust and existing dependencies; no new crate or Cargo.lock churn is
authorized. Name command-specific owners directly, such as src/shell,
src/session/shell and src/config/shell, while fitting actual repository layout.
Keep external text out of code, dynamic command registration and raw terminal
control. Preserve the selected six commands and five preferences from the spec.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | One-shot commands | `feat/rev-shell-commands-01-execution` | `feat(shell): run bounded native shell commands` | `ops/default/report/slices/rev-shell-commands/slice-1-execution.md` |
| Slice 2 | Checked text filters | `feat/rev-shell-commands-02-filters` | `feat(shell): filter regions and insert checked command output` | `ops/default/report/slices/rev-shell-commands/slice-2-filters.md` |
| Slice 3 | Native interaction and lifecycle | `feat/rev-shell-commands-03-interaction` | `feat(shell): integrate command keys and checked job lifecycle` | `ops/default/report/slices/rev-shell-commands/slice-3-interaction.md` |
| Slice 4 | Joint acceptance and guidance | `feat/rev-shell-commands-04-acceptance` | `test(shell): verify shell commands and document text filters` | `ops/default/report/slices/rev-shell-commands/slice-4-acceptance.md` |

## Slice 1: One-shot commands

Prerequisite: approved preparation on clean dev, no active Rev controller.
Requirements: SHL01-SHL03, SHL05-SHL07.

- T1.1: Admit the five shell preferences with complete defaults, sample, source origins and whole-candidate failure; preserve existing configuration behavior.
- T1.2: Deliver shell-command, cancellation and retained plain-text output through existing native commands/prompts and one bounded nonblocking owned process; capture directory and distinguish data from shell syntax.
- T1.3: Exercise real sh/printf, startup environment and cwd, output/diagnostic bounds, nonzero/invalid results, concurrent stream drainage, deadline and descendant cleanup in tests/shell_commands.rs. Preserve existing Org process contracts.

## Slice 2: Checked text filters

Prerequisite: Slice 1 integrated into dev.
Requirements: SHL03-SHL06.

- T2.1: Capture linear selections and insertion points before prompts, supply exact bounded stdin followed by EOF, and deliver shell-command-on-region, shell-filter-region and shell-command-insert.
- T2.2: Publish only complete successful stdout through native observed edits and one undo; preserve canonical newlines, source positions, readonly admission and strict stale/reload/closed-target rejection.
- T2.3: Verify real sort/tr with unsaved input, empty successful results, Unicode/newlines, cancelled prompts, read-only input, read-only edit rejection, edited/reloaded targets, pane switches and normal undo.

## Slice 3: Native interaction and lifecycle

Prerequisite: Slice 2 integrated into dev.
Requirements: SHL01, SHL04-SHL08.

- T3.1: Wire the four non-modal M-!/M-|/C-u sequences through effective native keymaps and prompt history; preserve Org C-u prefixes, configured overrides and modal motions.
- T3.2: Exercise shared M-x/colon invocation, Visual-origin selection, inner cancellation priority, no focus stealing, output reopening, one active job, source rebind and ordinary save/quit decisions with owned cleanup.
- T3.3: Complete startup/frame/status and idle integration without an always-running worker or idle timer. Reconcile command/configuration catalogs and delivered Unreleased behavior in the same slices.

## Slice 4: Joint acceptance and guidance

Prerequisite: Slice 3 integrated into dev.
Requirements: SHL01-SHL08 together.

- T4.1: Add examples/shell-command-acceptance.rs and scripts/check-shell-commands.sh using existing native/Rust/PTY support, real installed sh/sort/tr and isolated fixtures; add focused source-quality coverage for the new owners.
- T4.2: Measure finite near-limit input, collection/publication and post-job idle behavior; verify both profiles in representative Linux PTYs with resize, cancellation, canonical selection, undo and terminal restoration.
- T4.3: Add approachable Markdown guidance and exact references/defaults/bindings, link them through the existing editor path and Spore projection, and finalize meaningful Unreleased notes. Prepare clean dev for the unchanged controller gate.

## Validation scope and cost

The gate protects shell configuration/commands, canonical edit application,
modal selection/prefixes, external reload and the low-level process ownership
used by Org. The five native AI result tests protect the shared observed-edit
path without calling Pi. It is not the exhaustive repository suite and does
not run historical LSP, Babel, export or external-provider journeys.

Use the exact ordered prefixes below. Focus development checks on affected
cases; do not repeat every acceptance journey after every task. The final
combined gate runs once against the integrated clean candidate.

Current representative command/configuration suites take about 3 minutes
combined. New focused native cases should add seconds rather than minutes.
The acceptance driver adds two representative real PTY journeys and one finite
resource case. A changed release crate took about 4 minutes in the preceding
delivery; warm acceptance can reuse its private artifact. Expect roughly 5-9
minutes for an integrated warm gate depending on compilation, not an idle
deadline. Do not start parallel Rust jobs to speed it up.

## Build resources and cleanup

Prefix every Cargo-building command, including Bash drivers, with the four
low-footprint Cargo variables shown below: one build job, no incremental cache
and no dev/test debug information. Keep a private target per worktree. Never
share a mutable target between active branches, use tmpfs for build output or
interrupt another process.

Before each build require at least 1 GiB free on the target filesystem. The host
has about 3.7 GiB free at preparation; keep this run's artifacts bounded.
After a current-set slice is canonically integrated and its receipts are saved,
a later worker may reclaim only that completed slice's private target with
cargo clean --target-dir, after checking its exact owned worktree path, clean
source, merged PR/ancestry and absence of active build/validator processes.
Retain the source worktree, commits, reports and controller receipts. Do not
clean the primary target, other delivery sets, shared caches or unrelated files.
If admission still fails, retain the checkpoint and report the resource blocker.

## Required validation prefixes

### Slice 1

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands --test configuration -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 2

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands --test external_changes -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::ai::results -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 3

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands --test commands -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 4

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

## Final gate and closure

After four integrations the controller runs the exact Full gate above on clean
dev. No nightly workflow is dispatched and no historical gate is substituted.
Allow at most two bounded correction PRs for affected shell acceptance, preserving
the same frozen commands, run, conversation, counters and reports. Do not modify
Looper, add slices/dependencies, weaken assertions or change product scope to
make a check pass.

Only the following documentation paths may change during final closure:

- `ops/default/plan/rev-shell-commands.md`
- `ops/default/tracker/rev-shell-commands.md`
- `ops/default/report/slices/rev-shell-commands/slice-1-execution.md`
- `ops/default/report/slices/rev-shell-commands/slice-2-filters.md`
- `ops/default/report/slices/rev-shell-commands/slice-3-interaction.md`
- `ops/default/report/slices/rev-shell-commands/slice-4-acceptance.md`
- `ops/default/spec/editor-shell-commands.md`
- `ops/default/spec/editor-shell-commands-model.md`
- `ops/default/spec/rev-editor.md`
- `ops/default/README.md`

Mark plan/tracker and the selected shell spec/model Completed only after the
controller's successful gate receipt. Reports become delivered and the pending
index records actual evidence. Keep terminal/extension drafts untouched during
runtime delivery and closure.
