# Native Shell Commands Tracker

Status: Approved
Date: 2026-11-03
Delivery set: rev-shell-commands
Plan: [Native Shell Commands Delivery](../plan/rev-shell-commands.md)
Spec: [Shell Commands and Text Filters](../spec/editor-shell-commands.md)
Model: [Shell Commands Model](../spec/editor-shell-commands-model.md)
Approved: owner's specification separation and delegated first-capability startup request, 2026-11-03
Active slice: Slice 2 - Checked text filters
Progress owner: Looper checkpoint, canonical PRs and exact-candidate validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | One-shot commands | `feat/rev-shell-commands-01-execution` | `feat(shell): run bounded native shell commands` | `ops/default/report/slices/rev-shell-commands/slice-1-execution.md` |
| Slice 2 | Checked text filters | `feat/rev-shell-commands-02-filters` | `feat(shell): filter regions and insert checked command output` | `ops/default/report/slices/rev-shell-commands/slice-2-filters.md` |
| Slice 3 | Native interaction and lifecycle | `feat/rev-shell-commands-03-interaction` | `feat(shell): integrate command keys and checked job lifecycle` | `ops/default/report/slices/rev-shell-commands/slice-3-interaction.md` |
| Slice 4 | Joint acceptance and guidance | `feat/rev-shell-commands-04-acceptance` | `test(shell): verify shell commands and document text filters` | `ops/default/report/slices/rev-shell-commands/slice-4-acceptance.md` |

## Slice 1: One-shot commands

State: delivered
Dependency: approved preparation on clean dev.
Branch: `feat/rev-shell-commands-01-execution`
Expected PR title: `feat(shell): run bounded native shell commands`
Report: `ops/default/report/slices/rev-shell-commands/slice-1-execution.md`

- [x] T1.1: Admit the five shell preferences with complete defaults, sample, source origins and whole-candidate failure; preserve existing configuration behavior.
- [x] T1.2: Deliver shell-command, cancellation and retained plain-text output through existing native commands/prompts and one bounded nonblocking owned process; capture directory and distinguish data from shell syntax.
- [x] T1.3: Exercise real sh/printf, startup environment and cwd, output/diagnostic bounds, nonzero/invalid results, concurrent stream drainage, deadline and descendant cleanup in tests/shell_commands.rs. Preserve existing Org process contracts.
- [x] Record immutable report introduction and canonical PR: [#242](https://forge.adrianpk.com/adrianpk/rev/pulls/242).

Task commits: T1.1 `37f9791f5516d8e9821b32c71e224031d4a9fc1f`,
T1.2 `dd6c1119bb472b8b73aab861e0ba2951ddd69a1a`,
T1.3 `15a57e4206679e4f71eafc885ab320d538b59a8b`.
Report introduction: [Slice 1 report](https://forge.adrianpk.com/adrianpk/rev/src/commit/15a57e4206679e4f71eafc885ab320d538b59a8b/ops/default/report/slices/rev-shell-commands/slice-1-execution.md).
Integration: canonical PR #242 merged into dev at `5e1622b82ec342e502b12a08792ce45e50d41532`.
Controller receipt `2SHYGPQSSZE6UZAXEFY7T3LYUK` passed all six configured
checks on that exact clean candidate; 167 focused tests and the canonical PR
contract check passed.

## Slice 2: Checked text filters

State: implementing
Dependency: Slice 1 integrated.
Branch: `feat/rev-shell-commands-02-filters`
Expected PR title: `feat(shell): filter regions and insert checked command output`
Report: `ops/default/report/slices/rev-shell-commands/slice-2-filters.md`

Progress: T2.1 region capture, bounded stdin and display are implemented; checked
insertion/replacement complete with T2.2. Three focused region-input tests passed.

- [ ] T2.1: Capture linear selections and insertion points before prompts, supply exact bounded stdin followed by EOF, and deliver shell-command-on-region, shell-filter-region and shell-command-insert.
- [ ] T2.2: Publish only complete successful stdout through native observed edits and one undo; preserve canonical newlines, source positions, readonly admission and strict stale/reload/closed-target rejection.
- [ ] T2.3: Verify real sort/tr with unsaved input, empty successful results, Unicode/newlines, cancelled prompts, read-only input, read-only edit rejection, edited/reloaded targets, pane switches and normal undo.
- [ ] Record exact-head validation, immutable report link and canonical PR.

## Slice 3: Native interaction and lifecycle

State: pending
Dependency: Slice 2 integrated.
Branch: `feat/rev-shell-commands-03-interaction`
Expected PR title: `feat(shell): integrate command keys and checked job lifecycle`
Report: `ops/default/report/slices/rev-shell-commands/slice-3-interaction.md`

- [ ] T3.1: Wire the four non-modal M-!/M-|/C-u sequences through effective native keymaps and prompt history; preserve Org C-u prefixes, configured overrides and modal motions.
- [ ] T3.2: Exercise shared M-x/colon invocation, Visual-origin selection, inner cancellation priority, no focus stealing, output reopening, one active job, source rebind and ordinary save/quit decisions with owned cleanup.
- [ ] T3.3: Complete startup/frame/status and idle integration without an always-running worker or idle timer. Reconcile command/configuration catalogs and delivered Unreleased behavior in the same slices.
- [ ] Record exact-head validation, immutable report link and canonical PR.

## Slice 4: Joint acceptance and guidance

State: pending
Dependency: Slice 3 integrated.
Branch: `feat/rev-shell-commands-04-acceptance`
Expected PR title: `test(shell): verify shell commands and document text filters`
Report: `ops/default/report/slices/rev-shell-commands/slice-4-acceptance.md`

- [ ] T4.1: Add examples/shell-command-acceptance.rs and scripts/check-shell-commands.sh using existing native/Rust/PTY support, real installed sh/sort/tr and isolated fixtures; add focused source-quality coverage for the new owners.
- [ ] T4.2: Measure finite near-limit input, collection/publication and post-job idle behavior; verify both profiles in representative Linux PTYs with resize, cancellation, canonical selection, undo and terminal restoration.
- [ ] T4.3: Add approachable Markdown guidance and exact references/defaults/bindings, link them through the existing editor path and Spore projection, and finalize meaningful Unreleased notes. Prepare clean dev for the unchanged controller gate.
- [ ] Record exact-head validation, immutable report link and canonical PR.

## Validation and controller ownership

Use the plan's exact per-slice ordered prefixes and fixed final gate. The local
configuration is generated outside the repository and frozen before startup.
Controller checkpoints own progress; task checkboxes do not replace integration
or acceptance evidence. Record successful commands and distinguish focused,
combined, native, PTY and resource evidence.

Keep the plan's one-job/no-debug/no-incremental profile, 1 GiB admission floor
and narrowly owned completed-slice target cleanup. Do not clean shared targets
or unrelated worktrees.

## Final gate and closure

- [ ] All four canonical PRs are integrated into dev.
- [ ] Controller records the unchanged combined gate on clean integrated dev.
- [ ] All reports are delivered with exact candidate/receipt evidence.
- [ ] Plan/tracker/spec/model and pending index reflect completion.
- [ ] Closure changes only configured documentation paths.
- [ ] Closure is committed/pushed; clean dev matches origin/dev.

## Closure paths

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
