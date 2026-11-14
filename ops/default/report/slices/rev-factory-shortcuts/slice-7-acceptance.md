# Slice 7: Terminal Acceptance and Guidance

Status: delivered
Delivery set: rev-factory-shortcuts
Plan: [Factory Shortcut Alignment Delivery](../../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../../tracker/rev-factory-shortcuts.md)
Branch: `test/rev-factory-shortcuts-07-acceptance`
PR: `#53` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/53)); merged into dev at `979ba5ebb64310d0a1d1e3801d53689fdf525cec`
Report introduction: [immutable Slice 7 report](https://forge.adrianpk.com/rev/rev/src/commit/6f38eeb9417fb7719952acd0700dd11e941366da/ops/default/report/slices/rev-factory-shortcuts/slice-7-acceptance.md)

## Purpose

Verify the complete accepted shortcut manifest, physical terminal input and
final guidance before controller-owned integration and the delivery-set gate.

## Delivered Behavior

The focused acceptance script compares all 28 factory maps and their compiled
forms in both profiles with the independently frozen catalog plus accepted
edits. It verifies 707 canonical commands, six prompt actions, complete semantic
coverage and exactly 29 accepted records / 49 base-context edits, including
inherited, retired and untouched leaves. Expectations are not generated from
current defaults.

Native cases connect rendered selector hints to actual pane balance under
factory and explicit user assignments, retaining source, history, point and
focus. Three Linux PTYs exercise nonmodal editing, modal editing and local
read-only Git. The guide explains that Normal can advertise retained `C-w =`
while accepted `C-x +` remains available, and explicit context overrides win.

## Implementation Notes

T7.1 batches existing factory-shortcut, command-keymap, Org-context and command-
sequence cases, then configuration and modal keymap cases. The example reuses
the existing native configuration fixture, frame decoder, real-Git fixture,
Linux PTY driver and process metrics. The editor and example build once. No
runtime operation, dependency, default, consent rule or user configuration is
changed by this slice. The existing nonmodal redo-description expectation now
uses the independently accepted P03 `C-M-_` literal; its case, descriptions,
privacy assertions and cancellation checks remain intact.

T7.2 sends physical Escape/Meta, control and multichord bytes through the real
terminal parser. Both editing journeys verify first-text placement, exact
insertion/undo/redo/save, Project-prefix cancellation, Org/Text transitions,
Babel-prefix cancellation, selector hints and the pane chord. The Git journey
uses three local SHA-256 commits with two-entry pages: `+` loads complete
ancestry; `A` reviews the exact numbered full revision; Return cannot accept
that review; cancellation and the `c` prefix retain state; returning to source
preserves HEAD, index, disk and configuration. No remote or transport is used
by that representative journey.

The shared driver retains its eight-second child deadline, 1-MiB output cap,
finite observations, idle CPU check and child/reader cleanup. Each native or
physical invocation keeps a 30-second outer deadline. All three journeys check
TTY attributes, alternate screen, bracketed paste and cursor restoration.
Only owned fixtures are removed. Native variants retain existing single-thread
settings; no historical feature gate is chained into this script.

Frozen catalog SHA-256:
`bd694628253ddd3338e83aa5b79b1821429d50f5a1f6256e8490e0d99a715e3a`.
Accepted manifest SHA-256:
`e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf`.
The script pins both inputs. The semantic matrix, audit outputs, historical
acceptance inventories and startup template remain unchanged.

## Contracts Added or Changed

FSA01-FSA10 acceptance and guidance; existing native ownership, protected keys,
source admission, operation-specific review and explicit overrides remain.

## Files of Interest

- [Acceptance script](../../../../../scripts/check-factory-shortcuts.sh) and
  [entrypoint](../../../../../examples/factory-shortcuts-acceptance.rs).
- [Independent map comparison](../../../../../examples/factory_shortcuts_acceptance/matrix.rs),
  [native effects](../../../../../examples/factory_shortcuts_acceptance/native.rs)
  and [physical journeys](../../../../../examples/factory_shortcuts_acceptance/pty.rs).
- [Existing selector regression](../../../../../tests/commands/editor_keymaps.rs)
  and [modal guidance](../../../../../docs/tutorials/user-guide/11-modal-editing.md).
- [Accepted contract](../../rev-factory-shortcuts/mapping-contract.md).

## Validation

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: PASS; full-workspace formatting, strict Clippy and native policy across 1,918 files, two packages and 76 Cargo roots.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-factory-shortcuts.sh`: PASS; 89 selected command tests, 11 configuration keymap tests, six modal keymap tests, independent complete-map comparison, four native hint-to-effect variants and three Linux PTYs.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: PASS; 110 reachable pages, 780 local links/anchors, 707 names in each combined inventory, 322 schema entries, 10 command-reference and 12 configuration-reference tests; all 28 sample maps equal compiled defaults.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`: PASS; 111 Markdown pages projected for Spore, strict link checks, recursive render and download finalization into the owned ignored build output.
- `git diff --check`: PASS.

Focused `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands editor_keymaps::description_rows -- --test-threads=1`: PASS. Audit-document local links, incidental-identifier scan and both frozen digest checks also pass.

Measured local costs, with the ordinary build target warm after implementation:
lint 23.08 seconds; factory acceptance 297.81; documentation 25.81; guide 5.24;
diff check 0.04 (351.98 seconds total). The initial expanded all-target lint
took 206.06 seconds. Successful staged T7.1/T7.2/T7.3 hooks took 6.84/135.20/72.86 seconds;
their export caches remain separate from the ordinary target. All Cargo work
retains one job, disabled incremental output and no dev/test debuginfo.

The complete script's physical journeys took 1,857/2,094/2,511 ms and emitted
47,018/49,718/35,585 bytes for nonmodal/modal/Git respectively. Each idle
observation used zero CPU ticks over 350 ms; peak RSS was 37,824/41,108/36,976 kB.
The native process used one thread. All outputs remain below the existing cap.

The controller recorded all five exact required commands at clean HEAD
`979ba5ebb64310d0a1d1e3801d53689fdf525cec` after canonical PR references, in 286.73 seconds.
Canonical PR validation passed; PR #53 is integrated into dev at that same
commit. This evidence belongs to the slice candidate, not the later preparation
commit. The later controller-owned integrated full gate passed at
`72b8d40d816ee1cf2eb1aec333658fcb6a9c503e`; its exact command and evidence
are recorded in the completed tracker.

The first selected command batch at T7.2 commit
`601a67ec41b5b05fec312c78fa73d11842b9ef85` passed 88 cases and failed one stale nonmodal
redo-hint expectation. The accepted P03 literal corrects that expectation;
the focused description-row regression passes. Earlier implementation probes
corrected a right-aligned mode observation and split terminal workflow phases
to meet source-policy complexity limits. A first staged candidate exceeded
nesting depth; the retained failed tree is
`cb75d0dcc68e9537e3ae83dd44ab3da166c56132`. Table-driven profile expectations
corrected it. Assertions, resource limits and policy thresholds were preserved.

Controller full-gate command at `72b8d40d816ee1cf2eb1aec333658fcb6a9c503e`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-factory-shortcuts.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'`: PASS.

The gate repeats the accepted-map, native-effect and three physical journeys,
along with workspace lint, real hook acceptance, documentation and guide
projection. Source, resource limits, assertions and frozen manifest identity
are unchanged. The full-gate log SHA-256 is
`5e2d7927bb242e0a5bdc94a2828e92acc88d41c5594ae6074e5b335c1e7a4d2b`.

## Risks and Follow-ups

Physical acceptance uses the existing Linux PTY driver. Graphical terminals and
other platforms are outside these representative journeys. The controller
completed the unchanged full gate; originating-ticket and delivery-metadata
closure are now complete under T7.3. After verified integration, the clean, unlocked and
inactive Slice 7 worktree was removed without force; its branch and unrelated
worktrees remain intact.
