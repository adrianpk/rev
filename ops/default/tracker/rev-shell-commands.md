# Native Shell Commands Tracker

Status: Approved
Date: 2026-11-03
Delivery set: rev-shell-commands
Plan: [Native Shell Commands Delivery](../plan/rev-shell-commands.md)
Spec: [Shell Commands and Text Filters](../spec/editor-shell-commands.md)
Model: [Shell Commands Model](../spec/editor-shell-commands-model.md)
Approved: owner's specification separation and delegated first-capability startup request, 2026-11-03
Active slice: Slice 4 - Gate repair 1
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

State: delivered
Dependency: Slice 1 integrated.
Branch: `feat/rev-shell-commands-02-filters`
Expected PR title: `feat(shell): filter regions and insert checked command output`
Report: `ops/default/report/slices/rev-shell-commands/slice-2-filters.md`

Progress: T2.1-T2.3 deliver the three region/insert commands and checked native
publication. Development validation passed 22 native shell tests, the focused
real sort/tr cases and one real-child readonly-transition test.

- [x] T2.1: Capture linear selections and insertion points before prompts, supply exact bounded stdin followed by EOF, and deliver shell-command-on-region, shell-filter-region and shell-command-insert.
- [x] T2.2: Publish only complete successful stdout through native observed edits and one undo; preserve canonical newlines, source positions, readonly admission and strict stale/reload/closed-target rejection.
- [x] T2.3: Verify real sort/tr with unsaved input, empty successful results, Unicode/newlines, cancelled prompts, read-only input, read-only edit rejection, edited/reloaded targets, pane switches and normal undo.
- [x] Record immutable report introduction and canonical PR: [#243](https://forge.adrianpk.com/adrianpk/rev/pulls/243).

Task commits: T2.1 `494aa35190f30f4b80210ce6fb1a6f3437a87f2e`,
T2.2 `6f491a31ce9aa89d547951e757b1e0bd16c03d53`,
T2.3 `071b89040dd1833f4b79779d1186d6192c3a7c9e`.
Report introduction: [Slice 2 report](https://forge.adrianpk.com/adrianpk/rev/src/commit/071b89040dd1833f4b79779d1186d6192c3a7c9e/ops/default/report/slices/rev-shell-commands/slice-2-filters.md).
Integration: canonical PR #243 merged into dev at `ae496d8f79ea864233d64c8c7d43e2a898e0d7bc`.
Controller receipt `4PQQM2NGXFIIZ6A2R6WYQG4VUX` passed all six configured
commands on that exact clean candidate; 48 focused tests and documentation checks
passed. Receipt `LSFGO3J6QTLAMXN5UZML5KAFFD` passed the additional real-child
readonly-transition case on the same head. The canonical PR contract check passed.

## Slice 3: Native interaction and lifecycle

State: delivered
Dependency: Slice 2 integrated.
Branch: `feat/rev-shell-commands-03-interaction`
Expected PR title: `feat(shell): integrate command keys and checked job lifecycle`
Report: `ops/default/report/slices/rev-shell-commands/slice-3-interaction.md`

- [x] T3.1: Wire the four non-modal M-!/M-|/C-u sequences through effective native keymaps and prompt history; preserve Org C-u prefixes, configured overrides and modal motions.
- [x] T3.2: Exercise shared M-x/colon invocation, Visual-origin selection, inner cancellation priority, no focus stealing, output reopening, one active job, source rebind and ordinary save/quit decisions with owned cleanup.
- [x] T3.3: Complete startup/frame/status and idle integration without an always-running worker or idle timer. Reconcile command/configuration catalogs and delivered Unreleased behavior in the same slices.
- [x] Record immutable report introduction and canonical PR: [#244](https://forge.adrianpk.com/adrianpk/rev/pulls/244).

Task commits: T3.1 `7a898d22c8e867cf4dbfd2ab76f7b98a104d434e`,
T3.2 `9e408f818be84c70ba87fa06a821ba061c7a1a5f`.
T3.3 `fd88363c0e76fdde404288d13c3ba82a737303cc`.
Report introduction: [Slice 3 report](https://forge.adrianpk.com/adrianpk/rev/src/commit/fd88363c0e76fdde404288d13c3ba82a737303cc/ops/default/report/slices/rev-shell-commands/slice-3-interaction.md).
Integration: canonical PR #244 merged into dev at `a921b2f9317038cb933eca341c47e393a3f64000`.
Controller receipt `NSV5A2P6YV6LTBB2KFGKZ72KD6` passed all six configured
commands on that exact clean candidate: 218 native/keymap/modal tests plus
formatting, strict Clippy and documentation. The canonical PR contract check passed.

## Slice 4: Joint acceptance and guidance

State: delivered
Dependency: Slice 3 integrated.
Branch: `feat/rev-shell-commands-04-acceptance`
Expected PR title: `test(shell): verify shell commands and document text filters`
Report: `ops/default/report/slices/rev-shell-commands/slice-4-acceptance.md`

- [x] T4.1: Add examples/shell-command-acceptance.rs and scripts/check-shell-commands.sh using existing native/Rust/PTY support, real installed sh/sort/tr and isolated fixtures; add focused source-quality coverage for the new owners.
- [x] T4.2: Measure finite near-limit input, collection/publication and post-job idle behavior; verify both profiles in representative Linux PTYs with resize, cancellation, canonical selection, undo and terminal restoration.
- [x] T4.3: Add approachable Markdown guidance and exact references/defaults/bindings, link them through the existing editor path and Spore projection, and finalize meaningful Unreleased notes. Prepare clean dev for the unchanged controller gate.
- [x] Record immutable report introduction and canonical PR: [#245](https://forge.adrianpk.com/adrianpk/rev/pulls/245).

Task commits: T4.1 `db7c3fb38c81db7b7f471d667f72e16cbc130ebe`,
T4.2 `306286125fae9c44629a84c80d65c135c59e78b9`.
T4.3 `591267724625605c25936a6309d09c8386522821`.
Report introduction: [Slice 4 report](https://forge.adrianpk.com/adrianpk/rev/src/commit/591267724625605c25936a6309d09c8386522821/ops/default/report/slices/rev-shell-commands/slice-4-acceptance.md).
Integration: canonical PR #245 merged into dev at `554d1b770a2cee58328863afac1f0dd2c1e0ba1d`.
Controller receipt `3DQKGFLQ5HKNUHWESKIWUZ5FNL` passed all seven configured
commands on that exact clean head: formatting, strict all-target Clippy, 35 shell
tests, native/resource/both-profile PTY acceptance, documentation, local guide
build and `git diff --check`. Focused receipt `7FMBXVR66YUNL4HKD54J6GMZUW`
passed catalog, readonly-publication and Spore pipe-cell tests on the same head.
The canonical PR contract check passed.

## Validation and controller ownership

Use the plan's exact per-slice ordered prefixes and fixed final gate. The local
configuration is generated outside the repository and frozen before startup.
Controller checkpoints own progress; task checkboxes do not replace integration
or acceptance evidence. Record successful commands and distinguish focused,
combined, native, PTY and resource evidence.

Keep the plan's one-job/no-debug/no-incremental profile, 1 GiB admission floor
and narrowly owned completed-slice target cleanup. Do not clean shared targets
or unrelated worktrees.

Gate preparation on 2026-11-03 verified Rust/Cargo 1.98, Clippy, rustfmt,
Spore and every shell/PTY driver utility. Locked offline metadata resolved all
91 packages and confirmed the primary private target path. The executable
drivers create their own isolated HOME/XDG, source and PTY fixtures and clean up their owned
processes and temporary directories; no persistent service is required.
Only the integrated Slice 3 private target was reclaimed after exact path,
clean-source, merged-PR/ancestry, retained-receipt and inactive-target checks.
Source worktrees and receipts remain available. The 1 GiB filesystem admission
floor was met at preparation and remains required before each controller build.
The unchanged combined gate was not run by the preparation worker.

Gate repair 1: pending on `fix/rev-shell-commands-validation-1`, exact title
`fix(shell): repair approved command and filter acceptance`, existing Slice 4
report. Controller gate receipt `RSHBYQUHAMXM7FQYNTWOPSLFQS` failed on
`858c50f64105cde67ee128e1cb1d0c90b96cc9df` at
`configuration::reference::sample_maps`: the packaged keymaps omit the compiled
shell bindings. Repair only the seven affected template maps; keep complete
declared/effective-map equality, runtime behavior and all gate commands intact.
The retained configuration binary reproduced the missing VimNormal M-! and M-|
entries. Focused regression and the pinned five-command repair prefix precede
controller integration and the unchanged combined gate.

## Final gate and closure

- [x] All four canonical PRs are integrated into dev.
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
