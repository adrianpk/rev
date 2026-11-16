# Slice 5: Completed-frame continuity

Status: delivered
Delivery set: rev-delimiter-presentation
Plan: [Delimiter Presentation Delivery](../../../plan/rev-delimiter-presentation.md)
Tracker: [Delimiter Presentation Tracker](../../../tracker/rev-delimiter-presentation.md)
Branch: `test/rev-delimiter-presentation-05-continuity`
PR: [#71](https://forge.adrianpk.com/rev/rev/pulls/71)

## Purpose

Deliver T5.1-T5.3: the 24 frozen completed-frame bursts, source/projection
transitions, and bounded native qualification and input-service observations.

## Delivered Behavior

All 24 Text/Org/Markdown, non-modal/modal and depth/match/pulse/off bursts pass
32 insertions and 32 Backspaces inside leaf. All 1536 completed writer frames
retain the expected source, mapped endpoint identities, foregrounds, heading
symbols/colors, prefix geometry and cursor. Ordinary body point selects no
arbitrary enclosing pair; explicit matching remains available in every style.
Presentation controls and navigation preserve source revision and edit history.

Invalidating delimiter/quote/escape/newline edits discard old facts before a new
poll. Native observations cover resize, split-view policy/point divergence,
reload, Unicode/CRLF, narrow wrapping, folded read-only source, mark mapping and
ordinary transient-selection deactivation. NO_COLOR preserves heading geometry
and active underline without RGB or terminal blink.

An admitted neutral edit now retains its proved delimiter snapshot across native
view re-admission. The first failing burst exposed a plain-colored completed
frame: source advancement retired tickets, then empty-interest pruning discarded
the mapped snapshot before its live view requested the new range. Pruning now
retains that proof while a live interested source view exists. Off/closed views
still release it; source qualification and retention bounds remain unchanged.

## Implementation Notes

The existing completion observer is shared through `tests/support/completed.rs`.
Its Show-footer boundary, split-footer handling and original regression remain
unchanged. Every complete writer update is decoded with the existing cell helper;
partial projection frames cannot satisfy a positive source-frame assertion.
The frozen source/profile/style matrix and explicit input palette provide
independent byte, position and color expectations. Initial document analysis
must establish a positive heading baseline before a burst begins.

The native drain-owner case executes all four styles, pins the independent
32-event/4 ms constants, and verifies that an expired budget leaves the next
queued event untouched. The frozen 1000-line workload uses one Press followed
by repeats for 96 Down and 12 Up events, then Release. All eight profile/style
native journeys retain source/history and stop at row 84. Controlled expiry and
350 ms idle advancement leave no pair deadline, movement or recurring action.

The focused continuity batch took 41.92 s after compilation, within the initial
1-3 minute native estimate. The broader presentation suite took 137.88 s.
Native first progress was 3.616-8.243 ms, release-to-settled-frame 3.222-4.084 ms,
and the largest complete native event/frame observation turn was 29.940 ms.
These host observations include projection, qualification and writer decoding;
the 4 ms target applies to individual cooperative input/qualification slices.
They establish no physical terminal latency or idle CPU result.

Measured native qualification, with at most 256 tokens per poll:

| Workload | Inspected bytes | Visible spans | Retained bytes | Maximum poll us |
| --- | ---: | ---: | ---: | ---: |
| depth 128 | 256 | 256 | 61056 | 184 |
| depth 256 | 512 | 512 | 79488 | 286 |
| depth 257, limited | 272 | 0 | 42624 | 424 |
| 5000 adjacent pairs, view limited | 10000 | 8192 | 964224 | 258 |
| 262145-byte token, limited | 262144 | 0 | 32384 | 358 |
| offscreen mate source | 5002 | 2 | 50816 | 206 |

Cancellation retires the pending ticket and leaves less than 4096 retained bytes.
Each observed native frame also checks the unchanged 2 MiB buffer metadata cap.
The existing Unreleased feature entry now states that delimiter colors remain
stable while typing, using changelog-normalize.

Implementation/report introduction: `84572a9c0ac1ae78595683ec290fef374c0c642b`.
Canonical review: [PR #71](https://forge.adrianpk.com/rev/rev/pulls/71).

## Contracts Added or Changed

- Observe all 24 frozen bursts and every completed writer frame independently.
- Preserve mapped proof across live view re-admission after neutral source edits.
- Pin native count/time input service and verify repeat/release quiescence.
- Preserve conservative invalidation, bounded retention and source coordinates.
- Preserve frozen acceptance bytes, command inventory, bindings and dependencies.

## Files of Interest

- `tests/pair_presentation/continuity/`: bursts, transitions and measured workloads.
- `tests/support/completed.rs`: shared existing completed-writer observer.
- `examples/completion_acceptance/terminal.rs`: unchanged observer use/regression.
- `src/session/events/tests/presentation.rs`: native count/time input-owner case.
- `src/buffer/pairs.rs`, `src/pairs/state/presentation.rs`, `src/pairs/presentation/store.rs`: interested-view proof retention.
- `CHANGELOG.md`: stable delimiter colors in the existing product entry.

## Validation

All three ordered required commands PASS once through looper validate at clean
published `67f94244d2982f545be9f363408f62aadced76b5`, integrated by PR #71.
Fetched ancestry, clean state and no owned processes verified; the integrated
worktree was removed without force. Integration and the combined gate remain
controller-owned. Every selected filter is nonempty; no test is ignored.

- PASS: `cargo fmt --all -- --check`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation continuity -- --test-threads=1`: nine cases at the exact integrated HEAD.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation -- --test-threads=1`: 77 cases, including all 24 bursts and 18 pulse regressions.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs qualification -- --test-threads=1`: 21 cases.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation lifecycle -- --test-threads=1`: four cases.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib pair_presentation_continuity -- --test-threads=1 --nocapture`: one case across all four styles.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --example completion-acceptance every_completed_update -- --test-threads=1`: original observer regression.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: documentation checks and 22 reference cases.
- PASS: focused native source policy over all 16 changed Rust source/test files.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --test pair_presentation --example completion-acceptance -- -D warnings`.
- PASS: preserved implementation hook at staged tree
  `c54f2ece614dd021f572cea50494cf2e20a43260`: full workspace formatting and
  Clippy, zero blocking/deferred diagnostics, native source policy over all
  2008 first-party Rust files and 81 Cargo roots. The PR-reference commit changes
  operational documentation only.
- PASS: `git diff --check`.
- PASS: frozen acceptance SHA-256 remains
  `8a0ccc8e6a9ac4ebb24f4445c2f314b472045f6a426014b53ced065f477da771`.

## Risks and Follow-ups

Slice 6 retains the native drain-owner regression in final acceptance and owns
the two representative Linux PTYs, physical progress/release and idle CPU
observations, and User Guide acceptance. No PTY or delivery-set gate ran here.
The existing vendored crossterm parentheses warning is unchanged. The controller
owns integration, the combined gate and successor startup.
