# Slice 1: Confirmed mate jump

Status: delivered
Delivery set: rev-delimiter-navigation
Plan: [Delimiter Navigation Delivery](../../../plan/rev-delimiter-navigation.md)
Tracker: [Delimiter Navigation Tracker](../../../tracker/rev-delimiter-navigation.md)
Branch: `feat/rev-delimiter-navigation-01-mate`
PR: `#62`

## Purpose

Deliver the unbound matching-delimiter command under DPR02 using confirmed
typed pairs and the existing source-jump owner.

## Delivered Behavior

M-x matching-delimiter prefers a delimiter at point, then immediately before it,
and jumps to the other confirmed structural or quote endpoint. It never searches
later on the line. Read-only source and disabled automatic/quote editing remain
admitted. Explicit numeric prefixes refuse before movement.

Successful jumps preserve source, revision, undo history and the ordinary linear
mark, including an empty active region. Unmatched, mismatched, literal, uncertain,
limited, cancelled or stale requests publish no destination. The existing modal
percent command retains its current-line search and native jump traversal.

## Implementation Notes

The buffer selects one direct-entry lookup and owns its existing typed proof,
budget, retained facts and revision/generation ticket. The session retains only
the captured point/view/mark/editing state, ticket and prepared view. Native
layout admission polls the request and destination projection cooperatively;
C-g and stale ownership retire its ticket. Later input remains in the existing
finite FIFO and resumes only after resolution.

Confirmed results use the native source-jump owner and fold reveal. Scalar
endpoints inside a grapheme normalize in movement direction. Native prepend
cases explicitly verify forward 0 -> 3 for (U+0600), and backward 3 -> 0 for
U+0600(a), without splitting the affected cluster.

Argument admission and deferred publication have separate cohesive helpers to
retain the existing complexity ceiling. No lexical grammar or modal target
implementation changes.

## Contracts Added or Changed

- One unbound command, matching-delimiter, extends the catalog from 710 to 711.
- The independent factory expectation adds that literal identity while preserving
  all original 707 commands, three editing commands, six actions and 28 maps.
- Confirmed typed facts remain distinct from future group/expression outcomes.
- Byte/depth/retention/poll limits remain 262,144 / 256 / 2,097,152 / 256.
- No factory binding, configuration leaf, dependency or persisted format changes.

## Files of Interest

- `src/buffer/pairs.rs`: direct-entry qualification.
- `src/session/pairs/navigation.rs`: captured mate request and source jump.
- `tests/pair_navigation.rs`: approved mate behavior and lifecycle checks.
- `src/session/layout.rs`: existing cooperative owner and cancellation.
- `src/session/arguments.rs`: unsupported-prefix admission.
- `src/session/pairs/navigation/tests.rs`: pending point/mark/focus invalidation.
- `examples/factory_shortcuts_acceptance/matrix.rs`: independent catalog addition.
- Command/binding/delimiter references and Unreleased describe delivered behavior.

## Validation

Focused checks are local; the controller owns the combined delivery gate.
Final clean-HEAD evidence is recorded separately with looper validate after
the PR-reference commit.

- PASS: `cargo fmt --all -- --check`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation mate -- --test-threads=1`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib mate -- --test-threads=1`: one native invalidation test.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example factory-shortcuts-acceptance -- --native target`: independent complete maps/catalog and effective-hint routes.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: 711 catalog entries, 112 reachable pages, 792 local links/anchors, 10 command and 12 configuration reference tests.
- Both implementation and test commits passed their exact staged hooks:
  formatting, strict all-target/all-feature workspace Clippy and native policy
  for 1,958 and 1,960 files respectively. The preserved hook uses its exclusive
  cache; no hook was installed or bypassed.
- PASS: `git diff --check`.

The 11-test native suite reads all 19 approved mate cases without changing their bytes.
Additional cases protect escapes/raw/literal refusal, adjacent precedence,
endpoint round trips, prefix forms, active/empty marks, jump traversal, undo
barriers, custom hints, read-only Insert/Replace, modal percent, cancellation,
source/view replacement, queued input and grapheme normalization.

Fresh unoptimized native requests measured one complete parenthesized atom and
one mismatched closer at each size. These are observations, not latency bounds.
Source allocation precedes timing.

| Source bytes | Complete total / maximum poll us | Malformed total / maximum poll us | Inspected bytes | Retained bytes complete / malformed |
| --- | --- | --- | --- | --- |
| 1,024 | 493 / 126 | 460 / 119 | 1,024 | 27,632 / 19,440 |
| 65,536 | 33,418 / 157 | 34,152 / 155 | 65,536 | 27,632 / 19,440 |
| 262,144 | 140,471 / 168 | 157,762 / 579 | 262,144 | 27,632 / 19,440 |
| 1,048,576 | 148,293 / 268 | 139,618 / 200 | 262,144 | 19,440 / 19,440 |

The 1 MiB sources safely return Limited(Bytes). Depth 256 confirms, while 257
returns Limited(Depth); oversized ordinary atoms and strings also refuse without
source/history/mark changes. Tests assert all hard resource caps and at most 256
lexical steps per poll.

An 8,194-byte source through an actual configured native key event measured
301 us dispatch, 5 us queued-key admission and 27,319 us completion including
queued insertion. This measures native input processing, not a physical PTY.

## Risks and Follow-ups

PR #62 integrated validated commit `295aa89a6ac9db401784fbdd972d1624216a392d`
into dev. Its three ordered required checks passed through looper validate.
The clean inactive integrated worktree was removed without force.
The later group/expression slices and physical PTY acceptance are delivered.
The controller's unchanged combined gate passed at
`27957128f7626ee16b5dbdc50af52e9cee27f3c8`, receipt
`YEU4CDCDB3RP65D5PG67RRYTYF`. The
[Slice 5 closure](slice-5-acceptance.md#delivery-set-closure) records the exact
command, integrated outcomes and later Markdown-only closure relationship.
The existing vendored crossterm unused-parentheses warning remains unchanged.
