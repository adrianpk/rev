# Slice 4: Syntax-class expression motions

Status: reviewing
Delivery set: rev-delimiter-navigation
Plan: [Delimiter Navigation Delivery](../../../plan/rev-delimiter-navigation.md)
Tracker: [Delimiter Navigation Tracker](../../../tracker/rev-delimiter-navigation.md)
Branch: `feat/rev-delimiter-navigation-04-expressions`
PR: pending

## Purpose

Deliver the approved forward-sexp/backward-sexp syntax-class expression motions.

## Delivered Behavior

C-M-f/forward-sexp and C-M-b/backward-sexp cross mode-defined atoms, ordinary
strings and complete forms. Signed counts share one request; zero is inert,
negative reverses direction and normal file boundaries are admitted. A later
malformed, uncertain or limited destination refuses the entire counted motion.
Source, revision, history and the active linear mark remain unchanged, including
read-only source and empty active regions with automatic editing disabled.

C decimal punctuation separates syntax units, script dollar identifiers and CSS
hyphenated identifiers stay together, and Plain retains text-mode word
apostrophes. HTML crosses qualified tag tokens and attribute strings without
becoming an angle mate or whole-element jump. Known unterminated ordinary C
strings may reach EOF; raw/triple/template/embedded uncertainty stays conservative.
Org/Markdown retain native literal exclusions and selected-view source reveal.

## Implementation Notes

GroupRequest::expression adds an operation to the existing buffer-owned work.
Atoms, strings, forms and tag ranges use its reader, stack and bounded reverse
facts. There is one private destination, no per-count target vector or rescan.
GroupAnswer remains navigation evidence; the typed-pair answer API never exposes
these ranges as Confirmed facts. HTML token state commits only after successful
lexical advancement so a yielded prefix remains resumable.

The session reuses captured source/view/point/mark/focus, native cancellation,
deferred input, grapheme normalization and ordinary-motion publication. Org
reveals the selected fold while preserving peer folds. Markdown keeps its
existing pretty source projection; no new folding facility is introduced.

## Contracts Added or Changed

The catalog grows from 715 to 717. Exactly C-M-f/C-M-b join the four delivered
base group chords. Independent literal expectations preserve all original
commands/actions and complete maps in both profiles, with existing source-map
inheritance, modal filtering, custom overrides and effective hints. The packaged
sample exposes 133 Editor, 134 Insert and 133 Replace leaves.

The fixed limits remain 262,144 inspected bytes, depth 256, 2,097,152 retained
metadata bytes per buffer, 256 steps per poll and count magnitude 1,000,000.
Four milliseconds remains the scheduling target. Only EIP04 expression progress
is reconciled; paragraph motion and broad functional parity remain pending.

## Files of Interest

- `src/pairs/groups.rs`: shared counted group/expression work.
- `src/pairs/groups/expressions.rs`: profile-specific atom classes.
- `src/session/pairs/navigation.rs`: captured native motion dispatch.
- `tests/pair_navigation/expressions.rs`: endpoints, profiles and hard limits.
- `tests/pair_navigation/expressions/views.rs`: input, macros and projections.
- `examples/factory_shortcuts_acceptance/matrix.rs`: independent additive maps.

## Validation

Focused checks are local; the controller owns the combined delivery gate.
Final clean-HEAD evidence is recorded separately through looper validate after
the PR-reference commit.

- PASS: `cargo fmt --all -- --check`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation expressions -- --test-threads=1`: eleven tests, including atomic byte/depth/retention refusal.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- mate_stale groups_stale nesting_stale expressions_stale --test-threads=1`: four point/mark/focus invalidation tests.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation -- groups nesting mate::catalog_and_focus --test-threads=1`: 25 preserved navigation and mate catalog/focus tests.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- delimited_objects structural_targets --test-threads=1`: two modal percent/object regressions.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example factory-shortcuts-acceptance -- --native target`: 717 commands, six actions, all 28 complete maps in both profiles, seven navigation commands and exactly six new base chords. Historical inputs remain unchanged.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: 717 commands, 332 configuration entries, 112 reachable pages, 792 local links/anchors, ten command and twelve configuration reference tests.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation expressions::resources -- --test-threads=1 --nocapture`: resource measurements below.
- Implementation passed its exact staged workspace formatting, strict all-target/all-feature Clippy and native policy over 1,966 files, two packages and all 79 Cargo roots. The test/report commit uses the same unchanged hook.
- PASS: `git diff --check`.

All 37 frozen expression observations are read without editing their bytes.
The fundamental-mode apostrophe observation maps explicitly to the approved
Plain text-mode endpoint (5, rather than 3), independently recorded by the
text-mode case. Ordinary Lisp-compatible cases use Plain without inventing
reader or semicolon-comment semantics. Additional native cases cover profiles,
quote interiors and escapes, tags/attributes, signed/million counts, Unicode/
CRLF, readonly active/empty regions, macros, overrides, projections and lifecycle.

The first development run passed seven tests and exposed an attribute string
counted before its enclosing tag completed. Tag/attribute levels now remain
distinct. Two assertions were corrected for Markdown heading punctuation and
the cursor advance from queued insertion. The corrected suite passes.
Initial commit attempts rejected pre-format staged source and over-limit
metadata matches. Formatting and cohesive editing/navigation metadata owners
resolve those failures without exemptions, hook bypasses or changed limits.

Fresh unoptimized requests measured each size in both directions using complete
forms, an unmatched outer opener, long atoms and ordinary strings. Allocation
precedes timing. Totals are minimum/maximum microseconds across eight requests;
polling and retained maxima cover all eight.

| Source bytes | Total range | Maximum poll | Inspected bytes | Maximum retained bytes |
| --- | --- | --- | --- | --- |
| 1,024 | 583 / 1,804 | 907 | 1,024 | 8,776 |
| 65,536 | 37,961 / 43,983 | 253 | 65,536 | 8,776 |
| 262,144 | 159,632 / 203,317 | 355 | 262,144 | 8,776 |
| 1,048,576 | 169,389 / 200,112 | 322 | 262,144 | 2,632 |

All 1 MiB shapes refuse at the byte cap. An unmatched outer form refuses forward;
backward from its interior EOF can still reach its qualified atom. Depth 256
completes; 257 returns Limited(Depth). Dense reverse facts refuse within 2 MiB.
Tests assert all hard caps and at most 256 steps per poll. An earlier sample
observed a 12,450-microsecond poll; the four-millisecond scheduling target is
not a wall-clock latency guarantee.

Decoded native C-M-f on 8,198 bytes measured 362 microseconds to dispatch and
31,206 microseconds through completion and queued insertion. Physical terminal
transport remains integrated acceptance.

## Risks and Follow-ups

Slice 5 owns representative physical-input journeys and integrated guidance.
The controller owns integration, the combined gate and navigation closure.
No full delivery gate, guide build, main promotion or publication runs here.
The existing vendored crossterm parentheses warning is unchanged.
Implementation is `f918eb682440e27ca4f0dff6b50de5a1429c6045`; test/report introduction
and the PR reference are recorded with the final documentation commit.
