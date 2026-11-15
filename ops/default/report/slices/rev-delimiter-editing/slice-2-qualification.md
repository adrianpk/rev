# Slice 2: Bounded context and pair qualification

Status: delivered
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `feat/rev-delimiter-editing-02-qualification`
PR: [#56](https://forge.adrianpk.com/rev/rev/pulls/56)

## Purpose

Establish native, source-qualified delimiter evidence before automatic typing.
Keep typed mates distinct from the existing modal percent motion and pair objects.

## Delivered Behavior

One canonical buffer owns its pending request, qualified facts and finite work
budget. Queries classify a source point, qualify its typed delimiter, or find its
innermost confirmed enclosing pair. Results distinguish confirmed, unmatched,
mismatched, uncertain and limited outcomes. Unreached context remains Unknown;
no guessed mate, depth or partial edit is published.

Recognized language policy remains available with disabled colors or absent
syntax. Current native trees strengthen literal exclusion only at their exact
revision. Declared fallback profiles respect comments, escapes, admitted quote
kinds, Go raw strings, Org/Markdown literal regions and HTML attribute boundaries.
Unproved Rust lifetime/raw, script slash/template/JSX, triple-string and embedded
contexts remain uncertain or literal.

No source edit, command, binding, new parser dependency or presentation capability
is introduced. Existing modal motion and objects retain their native owners.

## Implementation Notes

- Source and current syntax are borrowed. A bounded tree-cursor observation
  retains no node or tree across revisions; external highlight names do not
  define quote grammar.
- Ticket generations retire cancelled, replaced or stale requests. Stored
  records use native scalar coordinates and acquire the current verified stamp
  when exposed as spans, without rewriting every retained record on each edit.
- Source publication can retain only an identity-mapped prefix whose complete
  lookahead horizon predates the first changed scalar. EOF observations count
  as dependencies. Two bounded lexical checkpoints permit nearby deletions;
  earlier changes discard unproved state. Settings and major-mode application
  retire the owner. Forward typing inspects only new suffix input; deletion
  tests restore a recent prefix rather than scanning the complete source again.
- Counted lookups continue one ticket without resetting inspection or retention.
  Cached fact searches also cooperate. Source reads use a finite 16-scalar window
  and chunk iteration; each newly inspected scalar is charged before admission.
  The native syntax point observation shares that poll's scalar allowance.
- Retention counts actual owned Rust payload: the state/work values, reader
  capacity, structural stack, both checkpoint stacks and fact-vector capacity.
  It excludes general allocator bookkeeping and canonical source owned elsewhere.
  Views retain no duplicate pairing cache. No polling timer or worker is created.

## Contracts Added or Changed

- Policy version 1 qualifies source stamp, query, language profile, provenance,
  source spans and proved nesting depth. Provenance distinguishes current syntax,
  lexical fallback, retained proof and explicitly remapped lexical proof.
- Fallback inspection is at most 262,144 bytes per request; nesting is at most
  256; retained pairing metadata is at most 2,097,152 bytes per buffer.
- Each cooperative poll handles at most 256 lexical/cache steps and 256 new
  scalar observations, with the existing four-millisecond scheduling target and
  an earlier caller deadline. Oversized tokens cannot bypass byte checks.
  Native syntax descent has its own finite 256-node bound.
- Native publication retires old tickets before exposing a new revision.
  Reuse requires the checked prefix mapping; ordinary source/history admission
  continues to own all edits and undo/redo.

## Files of Interest

- `src/pairs.rs`, `src/pairs/work.rs`: typed results, budgets, cooperative work
  and revision-qualified proof.
- `src/pairs/lexical.rs`, `src/pairs/profile.rs`, `src/pairs/reader.rs`:
  conservative language classification and bounded inspection.
- `src/buffer/pairs.rs`, `src/buffer/edit/publication.rs`: canonical buffer
  lifetime, cancellation and checked prefix mapping.
- `src/syntax/pairs.rs`: current bounded native syntax observation.
- `tests/pairs.rs`, `tests/pairs/qualification.rs`: independent matrices,
  lifecycle/resource regressions and reproducible measurements.

## Validation

Successful focused checks before publication:

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs qualification -- --test-threads=1 --nocapture`: 17 tests passed in 36.20 seconds after a 63-second rebuild; the subsequently added native AST-depth case passed in the focused command below.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs -- qualification::native_depth qualification::prefix_horizons --test-threads=1`: 2 passed in 1.65 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs qualification::native_observation_budget -- --exact --test-threads=1`: 1 passed in 0.45 seconds. This proves native syntax observation cannot add a 257th scalar read to a poll. The complete selected suite contains 19 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- local_targets::structural_targets objects::delimited_objects --test-threads=1`: 2 passed in 22.37 seconds; legacy percent motion and inner/around objects retained.
- `git diff --check`: passed.

Coverage includes mixed/nested/malformed and multiline pairs, quote/profile and
comment/escape matrices, unavailable/current/stale syntax, UTF-8/CRLF and combining
characters, every byte/depth/retention boundary, cumulative counted reuse,
cancellation, source replacement, shared views, mode/settings retirement,
forward typing reuse, recent deletion checkpoints and EOF/lookahead dependencies.
Personal configuration bytes remained unchanged. Fixtures use owned temporary
files and finite observable-condition deadlines.

All three source/test commits passed full staged-source formatting, Clippy and native
source policy (1,931 first-party Rust files and 77 Cargo roots after adding the
suite). Initial source-policy feedback required cohesive function extraction;
the first commit attempt then rejected one collapsible checkpoint condition.
That condition was corrected and the unchanged hook passed. No assertion, frozen
command or acceptance bound was relaxed. The vendored crossterm parentheses
warning is pre-existing and non-blocking.

### Native resource measurements

These are local unoptimized dev/test measurements, with one Cargo job, disabled
incremental compilation and no debug information. Each row has 11 cold requests;
source construction and cancellation of the prior request are outside timing.
The p95 is the nearest-rank maximum of those 11 observations. Cold qualification
includes the full bounded lexical/context proof, not only raw delimiter scanning.

| Source bytes | Workload | Median / p95 microseconds | Maximum poll microseconds | Inspected bytes | Retained bytes | Outcome |
| --- | --- | --- | --- | --- | --- | --- |
| 1,024 | Complete | 432 / 467 | 130 | 1,024 | 27,552 | Confirmed |
| 1,024 | Malformed | 473 / 505 | 157 | 1,024 | 19,360 | Mismatched |
| 1,024 | One quoted token | 354 / 390 | 113 | 1,024 | 27,552 | Confirmed outer pair |
| 65,536 | Complete | 33,257 / 35,515 | 216 | 65,536 | 27,552 | Confirmed |
| 65,536 | Malformed | 40,474 / 49,136 | 331 | 65,536 | 19,360 | Mismatched |
| 65,536 | One quoted token | 28,849 / 30,255 | 404 | 65,536 | 27,552 | Confirmed outer pair |
| 262,144 | Complete | 143,427 / 166,254 | 315 | 262,144 | 27,552 | Confirmed |
| 262,144 | Malformed | 138,563 / 159,093 | 301 | 262,144 | 19,360 | Mismatched |
| 262,144 | One quoted token | 114,035 / 118,265 | 265 | 262,144 | 27,552 | Confirmed outer pair |
| 1,048,576 | Complete | 142,003 / 174,812 | 636 | 262,144 | 19,360 | Limited bytes |
| 1,048,576 | Malformed | 139,834 / 174,305 | 720 | 262,144 | 19,360 | Limited bytes |
| 1,048,576 | One oversized quoted token | 115,434 / 120,238 | 232 | 262,144 | 19,360 | Limited bytes |

Each row used at most 256 steps per poll. The 256 KiB cases took 1,024 polls;
limited 1 MiB cases took 1,025. Measured maximum poll time was 720 microseconds,
below the four-millisecond target in this run; this is not a universal latency
guarantee. All 1 MiB cold rows refused after exactly 262,144 inspected bytes.

The actual depth-256 case confirmed its outer pair; depth 257 returned Limited
with maximum admitted depth 256. The native AST-path case stopped at 256 nodes
before inspecting source. Dense-pair metadata exhaustion occurred after 129,872
inspected bytes with exactly 2,097,152 retained payload bytes and returned Limited.
No final apparently balanced fragment was published as a match.

The plan's optimized raw primitive baseline had complete median/p95 values of
5/5, 277/303, 1,057/1,148 and 4,211/4,894 microseconds at the same four source
sizes, from 41 requests each. These debug-build measurements include classification,
strict nesting, budgeting and cooperation and are not an equivalent speed
comparison. They establish bounded cold-work cost. The unchanged-prefix tests
establish one-byte inspection for successive appends after a 64 KiB proof, zero
source inspection for cached pair lookup, and at most 256 inspected bytes for
three nearby deletions. Runtime/PTY input latency remains Slice 7 acceptance.

Final canonical evidence uses exactly the three injected commands, in their
frozen order, at the final clean published HEAD after PR-reference commits.
All three passed at `ed955dac56d22bb623ccec7b62652255af05632f`, including
all 19 qualification tests. Canonical PR #56 integrated that same commit into
`dev`; its title, body, branch, base and immutable report introduction passed
the controller check.
Supplemental focused commands remain recorded here. The combined delivery gate
is controller-owned and has not run for this slice.

## Risks and Follow-ups

Unproved language/literal boundaries deliberately return uncertain; they do not
prevent ordinary literal edits. Automatic typing, commands, deletion, wrapping,
physical-input acceptance and completed-frame continuity belong to later slices.
Caller integration must honor Pending, cancellation and exact source revisions;
this layer does not start a permanent worker or synchronously drain all work in
an editor input handler.
