# Slice 3: Acceptance and guidance

Status: delivered
Delivery set: rev-better-file-tree
Plan: [Better File Tree Delivery](../../../plan/rev-better-file-tree.md)
Tracker: [Better File Tree Tracker](../../../tracker/rev-better-file-tree.md)
Branch: `test/rev-better-file-tree-03-acceptance`
PR: `#289`
Integrated: `f42a106ccf2cc967f022b0c59e3e0b1bede57f6a`

## Purpose

Verify BFT01-BFT04 together, complete presentation and migration guidance, and
prepare the candidate for the controller-owned integrated gate and closure.

## Delivered Behavior

- Native acceptance covers both input profiles, omitted presentation flags and
  all four decoration/font combinations. It checks all seven observed kinds,
  fixed symbols, configured foregrounds, focused selection, minimal labels,
  shared bookmark font selection and the frozen startup snapshot.
- Include/root precedence and origins, invalid boolean/RGB/role rejection, narrow
  Unicode clipping and cached canonical hits are verified through public native
  boundaries. Hidden, zero and restored geometry retain cached tree identity.
  Selection-only full-row clicks and explicit opening preserve source/history.
- A separate NO_COLOR process verifies all four settings without RGB output and
  retains reverse-video selection and structural symbols.
- One real PTY journey per profile shows the tree, expands a directory with arrow
  input, opens a file by its icon cell and resizes through 40, 31 and 64 columns.
  Each checks idle behavior, unchanged file/configuration bytes and TTY recovery.
- The existing User Guide chapter has useful decorated/minimal previews and
  startup/font instructions. The reference explains existing-configuration
  behavior; configuration, navigation and gutter baselines reconcile the current
  contract while preserving historical completion evidence.

## Implementation Notes

T3.1 is implemented in `cc2ae9dfc614a8e7956aef79c96ed542a3d093ae`.
T3.2 guidance, baseline reconciliation and the initial T3.3 handoff report are in
report introduction commit `33411913f2e5d77191f61f27068b9d80ecbf8c39`.
Canonical PR #289 merged into dev at its validated final HEAD; the tracker
records both implementation commits.

The driver reuses the existing frame/surface inspector and completion acceptance
PTY helper. Native cases carry the configuration matrix; two PTYs provide real
input/render/resize/shutdown evidence without multiplying terminal sessions
across that matrix. Cached movement and resize are exercised after deliberately
removing only the owned observed directory; source ropes and undo histories stay
unchanged.

The shell builds the editor and driver once, isolates HOME and all XDG roots,
bounds native/PTY driver processes to 20 seconds and each owned terminal to eight
seconds, and retains the existing 1 MiB PTY output cap and bounded channel.
Shutdown verifies cursor, alternate-screen and bracketed-paste restoration,
unchanged terminal attributes and successful child reaping. The EXIT trap removes
only its fixture and asserts that removal.

The guide remains projected from Markdown by the existing publication example
and Spore build. No parallel publication prose is maintained. Slice 2's report
and tracker were closed after verifying canonical PR #288 merged at validated
HEAD `e86542d3fde474cad006a69aeb22ee873c062355`; Slice 3 was activated for this work.

## Contracts Added or Changed

- The new acceptance entrypoint owns a bounded combined native/PTY check for this
  presentation; no dependency, runtime provider or production tree owner is added.
- Reference and tutorial guidance describe startup-only settings, safe ordinary
  symbols, minimal compatibility, palette migration and explicit font support.
- Six affected baseline documents retain their original status, candidate and
  gate metadata. Their new presentation clauses link to the selected feature
  specification/model and separate delivery evidence.

## Files of Interest

- `examples/better-file-tree-acceptance.rs`,
  `examples/better_file_tree_acceptance/`,
  `scripts/check-better-file-tree.sh`: bounded acceptance and cleanup.
- `docs/tutorials/user-guide/05-file-tree.md`,
  `docs/reference/file-tree.md`, `docs/reference/defaults.md`: current guidance.
- `ops/default/spec/editor-configuration.md` and its model,
  `editor-file-navigation.md` and its model, `editor-better-gutter.md` and its
  model: targeted baseline reconciliation.

## Validation

The six required commands passed locally in this order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree --test configuration -- --test-threads=1`
- `bash -n scripts/check-better-file-tree.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 231 tests: configuration 160 and tree 71. Observed command
times were 3.16s formatting, 48.09s all-targets Clippy, 114.28s tests, under 0.01s
shell syntax, 1.09s documentation and 0.01s whitespace. Documentation validation
checked 85 reachable pages, 572 local links/anchors, 596 commands and 289
configuration entries against the live schema/sample. A bounded link check also
resolved all 118 local file links in the changed guide/baseline Markdown.

Focused checks also passed:

- `bash scripts/check-better-file-tree.sh` (11.35s with cached builds).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh` (1.99s).

The native matrix uses ten profile/presentation cases, four fresh-process
NO_COLOR cases and all four settings at six preferred widths (1, 5, 8, 9, 13,
24), plus hidden/zero/restored surfaces. PTY non-modal/modal journeys took
918/987ms and emitted 5022/5041 bytes. Both observed zero CPU ticks during their
350ms idle sample; Linux peak RSS was 29888/30600 kB. These are fixture
observations, not performance guarantees. The initial cold driver/editor build
took 67s; no bounds were raised and no test was disabled or weakened.

Spore projected and built 85 Markdown pages locally. No deployment is implied.
No owned editor process or acceptance temporary directory remained after the
checks. Looper validate passed all six ordered commands on clean final
PR-reference HEAD `f42a106ccf2cc967f022b0c59e3e0b1bede57f6a` in 114.40s.
Canonical PR #289 merged into dev at that same commit. This slice evidence
belongs to that HEAD; the controller final gate tests the prepared candidate.
These slice checks do not replace the controller's frozen integrated gate.

## Risks and Follow-ups

Native and PTY checks prove emitted code points and terminal-cell geometry;
they do not establish the drawing of an installed physical font. The controller
still owns the frozen final gate and delivery closure. The separate
selection/active-buffer synchronization ticket and main/release/publication
boundaries retain their scope.

