# Slice 3: Enclosing and nested groups

Status: delivered
Delivery set: rev-delimiter-navigation
Plan: [Delimiter Navigation Delivery](../../../plan/rev-delimiter-navigation.md)
Tracker: [Delimiter Navigation Tracker](../../../tracker/rev-delimiter-navigation.md)
Branch: `feat/rev-delimiter-navigation-03-nesting`
PR: [#64](https://forge.adrianpk.com/rev/rev/pulls/64)

## Purpose

Deliver up-pair/down-pair through the existing bounded group and native view
owners, including signed counts, source projection and unchanged linear regions.

## Delivered Behavior

C-M-u leaves the enclosing group backward onto its opening delimiter. C-M-d
enters the next nested group forward just after its opener. M-x exposes both
canonical names. Counts repeat that nesting relationship once in a captured
request; zero is inert. Negative up-pair exits forward after enclosing closers;
negative down-pair enters backward just before closing delimiters.

At an opener, up-pair exits the parent; at a closer it exits the current group.
Point after a closer is outside that group. Missing enclosing/nested levels,
uncertain syntax or exhausted limits refuse atomically. Reverse counted entry
cannot select a nested group from an earlier sibling. Normal source boundaries
remain usable when they are the requested qualified target.

Read-only source and disabled automatic/quote editing remain admitted. Source,
revision, history and active linear mark stay unchanged, including empty active
regions. Scalar endpoints normalize in movement direction to native grapheme
boundaries; tests cover combining/prepend characters and CRLF.

## Implementation Notes

GroupRequest::nesting extends the existing buffer-owned work with an operation
kind. Backward exit uses its qualified prefix stack; forward exit advances the
enclosing depth. Forward entry advances nested depth, while backward entry
visits bounded completed source groups in reverse and retains the newly entered
opening as its lower boundary. One destination stays private until completion.
Counts never reset the reader budget or allocate a per-count target list.

The session reuses captured revision/view/point/mark/editing state, cooperative
layout continuation, cancellation and deferred input. The selected candidate
uses the existing view reveal and ordinary-motion owner. Org folds reveal only
in that view; peer folds remain intact. Markdown uses its existing pretty source
projection. Manual fold installation is an Org-only API; this slice adds no new
Markdown folding facility. Typed pair answers remain separate from group targets.

## Contracts Added or Changed

The catalog grows from 713 to 715. Exactly C-M-u/C-M-d are added to base Editor;
the existing nine source maps inherit the four delivered group chords. Normal/
Visual filtering, explicit overrides, effective hints, modal percent/objects and
component focus retain their owners. Native macro and commit-source guards admit
the two new ordinary motions. Packaged sample maps and documented counts agree.

Limits remain 262,144 inspected bytes, depth 256, 2,097,152 retained metadata
bytes per buffer, 256 lexical/search steps per poll and count magnitude 1,000,000.
The existing four-millisecond poll deadline is a scheduling target.

## Files of Interest

- `src/pairs/groups.rs`: qualified group requests.
- `src/session/pairs/navigation.rs`: captured native view publication.
- `tests/pair_navigation/nesting.rs`: nesting behavior and lifecycle coverage.
- `tests/pair_navigation/nesting/views.rs`: native input, macros and projections.
- `src/session/pairs/navigation/tests.rs`: pending point/mark/focus invalidation.
- `examples/factory_shortcuts_acceptance/matrix.rs`: literal additive expectations.

## Validation

Focused checks are local; the controller owns the combined delivery gate.
All three ordered required commands passed through looper validate at clean,
pushed `3fd22cbbb24f56adecf57b8dade7b242b38f1da4`, integrated by PR #64.

- PASS: `cargo fmt --all -- --check`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation nesting -- --test-threads=1`: 10 tests, 33.63 seconds after compilation.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- mate_stale groups_stale nesting_stale --test-threads=1`: three native invalidation tests.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation -- groups mate::catalog_and_focus --test-threads=1`: 15 across-group, strict-qualification, input and mate catalog/focus regressions.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- delimited_objects structural_targets --test-threads=1`: two existing modal percent/object tests.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example factory-shortcuts-acceptance -- --native target`: 715 commands, six actions, all 28 complete maps in both profiles, and exactly the four delivered base group chords with their existing inheritance. Historical factory inputs remain unchanged.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: 715 commands, 332 configuration entries, 112 reachable pages, 792 local links/anchors, 10 command and 12 configuration reference tests.
- Both implementation and test commits passed their exact staged hooks: workspace formatting, strict all-target/all-feature Clippy and full native policy over 1,963 and 1,965 files respectively, with all 79 Cargo roots selected. Hooks and their private worktree cache remain unchanged.
- PASS: `git diff --check`.

All twelve applicable frozen up/down cases are read without altering their
observation bytes. Additional cases cover nesting endpoints, mixed kinds, signed
and million-count refusal, active/empty mark, readonly source, language exclusions,
graphemes, custom hints, inheritance, macros, peer projections and queued input.
The initial `--nocapture` nesting run passed nine tests and failed a fixture that
called the Org-only fold API on Markdown. The corrected native-projection case
and subsequent complete required suite pass without changing projection policy.

Fresh unoptimized native buffer requests measured four directions (up/down and
both reverses), each with complete and malformed ordinary-string source. Source
allocation precedes timing. Table totals are minimum/maximum microseconds across
the four complete or malformed requests; maximum poll covers all eight.

| Source bytes | Complete total range | Malformed total range | Maximum poll | Inspected bytes | Maximum retained bytes |
| --- | --- | --- | --- | --- | --- |
| 1,024 | 668 / 1,078 | 675 / 1,014 | 546 | 1,024 | 8,736 |
| 65,536 | 46,752 / 50,459 | 45,954 / 48,829 | 1,577 | 65,536 | 8,736 |
| 262,144 | 201,059 / 465,696 | 201,882 / 369,385 | 24,853 | 262,144 | 8,736 |
| 1,048,576 | 203,633 / 281,402 | 200,717 / 342,770 | 16,957 | 262,144 | 2,592 |

The 1 MiB shapes refuse at the byte cap. Malformed smaller shapes refuse without
a target. Depth 256 enters all 256 levels; 257 returns Limited(Depth). Dense
backward-entry facts refuse within 2 MiB. Hard resource caps and at most 256 steps
per poll are asserted. Observed wall-clock polls exceeded the four-millisecond
scheduling target; these samples are measurements, not a latency guarantee.

Decoded C-M-d dispatch on 8,198 bytes measured 319 microseconds; completion plus
queued insertion measured 30,217 microseconds. Physical terminal transport and
representative PTYs remain separate integrated acceptance.

## Risks and Follow-ups

PR #64 integrated into dev. Implementation is `b49dd9aa818d5242e9524ee57f3e6ec2eb181884`;
test/report introduction is `51a95aad55804ebd8f5564a922d9c4f71ce96cc2`.
The validated head is `3fd22cbbb24f56adecf57b8dade7b242b38f1da4`. Fetched ancestry,
cleanliness and inactivity were verified before non-forced owned worktree removal.
Controller-owned combined acceptance remains pending. Syntax-class
expressions and representative physical-input PTYs belong to the next recorded
slices. No full delivery gate was run here. The existing vendored crossterm
parentheses warning remains unchanged.
