# Slice 1: Effective hints

Status: delivered
Delivery set: rev-command-keybinding-hints
Plan: [Command Keybinding Hints Delivery](../../../plan/rev-command-keybinding-hints.md)
Tracker: [Command Keybinding Hints Tracker](../../../tracker/rev-command-keybinding-hints.md)
Branch: `feat/rev-command-keybinding-hints-01-effective`
PR: `#294`
Integrated: `05365b43a51f5aaa7217c489117afc78289b0abe`

## Purpose

Deliver KBH01-KBH05 through effective shortcut annotations in the shared M-x and
modal colon selector, retaining UI02 and CMR01-CMR05 behavior.

## Delivered Behavior

- Each bound command shows one normalized shortcut from the originating profile,
  modal state and focused owner. Shortest normalized sequences win, followed by
  lexical order. Unbound and admitted AI names remain ordinary candidates.
- Local maps, configured replacements, shadowed leaves/prefixes, modal counts,
  tree aliases and live terminal first refusal determine reachability. The
  minibuffer map never substitutes for the originating map.
- Hints remain separate from canonical names, filtering, ranking, Tab text,
  Return dispatch, history and usage. Changing an origin owner retires its hints;
  reopening captures the current owner.
- Complete shortcuts align at the right edge, after the full name and at least
  two cells. A hint that cannot fit disappears as a whole. Names retain existing
  clipping; selection spans both fields and secondary text uses status_text.
- The terminal separator no longer consumes the final command candidate row,
  including a single-result selector opened from the lower terminal panel.

## Implementation Notes

T1.1 and T1.2 source is implemented in
`c18cdc0b5bb3feb05074e4d2bcba07c0afcfba6b`. T1.3 is implemented in
`844ec6c5fb9d0c6f71d1de2ca6f98538423d8d12`: existing-suite regressions, the Unreleased outcome and
this report. Its immutable introduction is that same commit. Canonical
[PR #294](https://forge.adrianpk.com/adrianpk/rev/pulls/294) merged into `dev` at that exact validated head on 2026-11-06.

Input inspects only sequences already admitted by the effective maps, bounded
at eight normalized chords. One isolated input probe owns a map snapshot and
replays those finite sequences through the existing grammar, normalization and
resolver without executing their command outcomes. It owns no persistence task,
provider, live prefix, shell history or usage events. Terminal first-refusal and
tree alias interpretation are shared with their actual dispatch owners.

Palette stores one transient label per canonical command identity before it
takes focus. Repaints and filtering reuse that snapshot. The renderer performs
cell measurement and existing clipping only; it neither looks up maps nor does
filesystem or command I/O. No dependency, configuration key, persistence schema
or model companion is added.

## Contracts Added or Changed

- `Palette::hint(command)` exposes the prepared read-only presentation label.
- Owner transitions retire hints without rewriting query or usage identities.
- Existing selection colors cover the full row, including hint and gap cells.
  Secondary text reuses palette.status_text on the normal background.
- A command selector above a lower terminal panel reserves its separator
  outside the candidate region. Other prompt composition is unchanged.

## Files of Interest

- `src/input/hints.rs`, `src/input/palette.rs`: bounded resolver inspection and
  identity-keyed selector snapshot.
- `src/input/context.rs`, `src/command/tree.rs`,
  `src/session/terminals/input.rs`: shared input-owner rules.
- `src/input/modal.rs`, `src/input/org.rs`, `src/input/dispatch.rs`,
  `src/session/mode.rs`, `src/session/extended.rs`: origin capture/retirement.
- `src/display/completion.rs`, `src/display/style.rs`, `src/session/tree.rs`:
  complete cell-safe fields, existing palette roles and terminal composition.
- `tests/commands/editor_keymaps.rs`, `tests/commands/context_keymaps.rs`,
  `tests/modal/keymaps.rs`, `tests/display/ranking.rs`: focused regressions.
- `CHANGELOG.md`: one Unreleased user-visible outcome.

## Validation

Focused implementation checks passed eight tests with:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test modal --test display hint -- --test-threads=1`

The matrix covers factory and configured maps, normalized Meta and undo aliases,
shortest/lexical choice, the eight-chord bound, unbound AI names, all six modal
states, both entry gestures, local shadowing, tree/terminal ownership, grammar
consumption, identity/ranking, Tab, cancellation, Unicode widths, exact/narrow/
zero geometry, resize and actual selected/unselected frame colors. Existing
ranking assertions inspect the name field; explicit hint assertions protect the
new field. Owned terminal fixtures are shut down through their existing owner.

All five required Slice 1 commands passed locally, in canonical order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test modal --test display -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The required suites passed 336 tests: commands 172, display 44 and modal 120.
Documentation validation passed 596 commands per inventory, 289 configuration
entries, 85 reachable pages and 576 local links/anchors. Implementation-batch
timings were 2.77s, 14.23s, 321.49s, 43.84s, 0.01s
for formatting, Clippy, tests, documentation and whitespace respectively.

The initial focused run incorrectly inspected save-buffer echo on an unnamed
buffer; the corrected fixture verifies its Write file prompt and subsequent
canonical ranking. The terminal fixture then exposed a missing final candidate
row; the separator reservation fixes the actual composition. Focused reruns
pass without disabled tests or weaker acceptance.

Exact clean-HEAD ordered evidence belongs to the controller validation receipt
for the final PR-reference commit. These are Slice 1 checks; the integrated
acceptance gate is not part of this worker result.

## Risks and Follow-ups

Slice 2 owns the approved joint native/PTY acceptance, NO_COLOR journey,
command/User Guide updates, Spore projection and interaction-baseline
reconciliation. The combined full gate, merge and successor startup remain
controller-owned. No publication, installation or release is included.

## Integration

Canonical PR #294 merged into dev at
`05365b43a51f5aaa7217c489117afc78289b0abe`. Receipt
`D6ITQPUQYZ5CJ5CUMWE6JOV236` passed the five ordered commands on that exact
clean candidate, with 336 passing tests and the documentation checks above.
The immutable introduction remains `844ec6c5fb9d0c6f71d1de2ca6f98538423d8d12`.
Slice 2 now owns joint acceptance and guidance; this integration does not claim
the combined delivery-set gate.
