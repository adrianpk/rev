# Delimiter Presentation Delivery

Status: Approved
Date: 2026-11-15
Delivery set: rev-delimiter-presentation
Slice strategy: behavior-first
Planning authority: product-owner continuation to the final delimiter delivery, 2026-11-15
Approved: product owner, 2026-11-15; presentation after accepted editing and navigation
Execution authority: slices 1-6 and controller-owned predecessor-ordered Rebase + Fast-forward integration into dev
Planning baseline: `744497bab434cc38204f091398c89e21d5147476`
Spec: [Delimiter Presentation](../spec/editor-delimiter-presentation.md)
Model: [Pair Preferences and Native State](../spec/editor-delimiter-pairs-model.md)
Concern: [Operations index](../README.md)
Tracker: [Delimiter Presentation Tracker](../tracker/rev-delimiter-presentation.md)
Acceptance: [Frozen contract](../report/rev-delimiter-presentation/acceptance-contract.json)
Active slice: Slice 2 of 6 - Qualified presentation facts
Gate state: pending; controller-owned after all six canonical PRs integrate
Integration target: `dev`

## Outcome and strategy

Complete the third and final delimiter delivery: palette-derived depth colors by
default, restrained active matching, finite pulse and off. The source document,
ordinary editing/navigation operations and terminal configuration keep their
native owners. The only new commands are pair-highlight-mode and
pair-highlight-style, both unbound. No dependency, grammar or permanent service
is added. Main promotion, releases and mirror publication require separate authority.

Behavior-first slices publish independently testable configuration, qualified
facts, visible rendering and pulse behavior, followed by joint continuity and
acceptance. Preparatory slices remain valid and passing; they do not claim a
visible feature or publish a release note before rendering is usable. All styles
share the same qualification, projection and overlay contract from inception.
The final two slices prove integration, not permission to defer known failures.

## Entry, integration and worktree ownership

Editing and navigation are Completed and manually accepted. Navigation's exact
gate at 27957128f7626ee16b5dbdc50af52e9cee27f3c8 has receipt
YEU4CDCDB3RP65D5PG67RRYTYF; its Markdown closure is the planning baseline above.
This is a new delivery set, not reuse of either completed run or conversation.
Publish this specification/model portion, plan, tracker and independent contract
before creating the first runtime branch. There is no project ADR index.

Keep the primary checkout on dev. Every slice uses its exact branch/report below
and a dated worktree under the canonical project worktree root. Each checkout
owns its Cargo target and hook cache; use one build job, disabled incremental
output and no dev/test debuginfo. Preserve existing hooks and active editors,
builds, controllers and unrelated worktrees. No early hook installation or full
set gate. After integration verify fetched ancestry, clean state and no active
processes, then remove that slice's worktree without force before its successor.
This cleanup is mandatory in the same task; do not accumulate six build trees.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Configuration and view controls | `feat/rev-delimiter-presentation-01-config` | `feat(editor): configure view-local delimiter presentation` | `ops/default/report/slices/rev-delimiter-presentation/slice-1-config.md` |
| Slice 2 | Qualified presentation facts | `feat/rev-delimiter-presentation-02-qualification` | `feat(editor): retain bounded source-qualified delimiter spans` | `ops/default/report/slices/rev-delimiter-presentation/slice-2-qualification.md` |
| Slice 3 | Depth colors and active pair | `feat/rev-delimiter-presentation-03-rendering` | `feat(editor): render delimiter depth and active-pair marks` | `ops/default/report/slices/rev-delimiter-presentation/slice-3-rendering.md` |
| Slice 4 | Finite pulse lifecycle | `feat/rev-delimiter-presentation-04-pulse` | `feat(editor): add finite delimiter presentation pulses` | `ops/default/report/slices/rev-delimiter-presentation/slice-4-pulse.md` |
| Slice 5 | Completed-frame continuity | `test/rev-delimiter-presentation-05-continuity` | `test(editor): verify delimiter continuity and input service` | `ops/default/report/slices/rev-delimiter-presentation/slice-5-continuity.md` |
| Slice 6 | Integrated presentation acceptance | `test/rev-delimiter-presentation-06-acceptance` | `test(editor): verify delimiter presentation workflows` | `ops/default/report/slices/rev-delimiter-presentation/slice-6-acceptance.md` |

## Frozen controls, palette and source authority

Global pairs.highlight defaults to depth; pairs.pulse_ms defaults to 250 with
50..1000 admission. Optional language-level leaves inherit global values and
native source provenance. Startup defaults initialize a view; runtime overrides
and last non-off style belong to that view and never save settings. The four
optional palette.pairs roles resolve after ordinary palette merging: selection
for match_background, foreground for match_text, warning for mismatch, and the
ordered function/number/keyword/type/string/punctuation syntax roles for the
six-color fallback. Explicit depth_colors has 1..8 opaque RGB entries.

Entry uses a confirmed typed endpoint at point, then immediately before it.
Do not search arbitrary enclosing forms from an ordinary body. Only structural
endpoints get depth color. Eligible quotes preserve their string foreground.
Selection/search foreground and background, cursor and diagnostic attributes
retain priority. Other active endpoint cells add background/underline; match
and pulse use match_text only on structural endpoints. NO_COLOR/capability
fallback never invents color or punctuation. Offscreen mate feedback is a
lower-priority native line/column hint and cannot move point/view or replace
prompts, errors or explicit command feedback.

Reuse the single buffer-owned source qualification/fact owner. Immutable spans
may be shared but retained bytes are charged once; do not create parallel mutable
source caches or let background presentation retire a foreground pair request.
A loose group/expression result is not typed matching proof. Source advancement
requires exact transaction mapping and unchanged lexical/nesting qualification;
otherwise invalidate unproved ranges. Same-line neutral source edits retain
proved colors through every completed frame, including Org/Markdown projections.

## Fixed bounds and workload

The independent JSON contract fixes the controls, literal catalog additions,
resource limits, fixture sources and observations before runtime implementation.
It is an independent oracle, never generated from live application output. Its
SHA-256 is 8a0ccc8e6a9ac4ebb24f4445c2f314b472045f6a426014b53ced065f477da771;
the acceptance script must verify those bytes, not rewrite the expected contract.

- Inspect at most 262144 source bytes per shared request, depth at most 256.
- Retain at most 8192 eligible visible delimiter spans per view; all retained
  pair metadata, shared and per-view, counts within 2097152 bytes per buffer.
- A cooperative poll processes at most 256 tokens under the existing 4 ms
  scheduling target. Do not reset budgets per span, depth, view or counted move.
- Pulse owns at most one generation/expiry per admitted view. Depth/match/off and
  an expired pulse own no pair deadline. Repaint/resize do not extend expiry.
- Exhaustion is limited/uncertain, never a fabricated depth, mate or mismatch.

Resource sketch: 8192 endpoint records at an illustrative 64 bytes each consume
512 KiB before indexes/proof state. The 2 MiB buffer cap therefore requires
explicit admission/accounting, immutable sharing and bounded per-view metadata,
not multiplication of an unaccounted full copy by every split. This estimate is
not a claim about Rust layout; Slice 2 measures real retained bytes, poll duration
and cancellation with the native allocation/work counters. Existing byte/depth
budgets already apply to editing/navigation and cannot be relaxed for rendering.

Freeze 24 completed-frame bursts: Text/Org/Markdown x non-modal/modal x the four
styles, each with 32 neutral ASCII insertions and 32 Backspaces inside leaf.
Observe every completed writer frame, not just the last frame or row text.
Compare source-qualified endpoint identity/foreground and current mapped cells;
unchanged heading symbols/title/prefix geometry remain stable. Delimiter cells
can legitimately move after an insertion; do not equate continuity with stale
absolute terminal columns. Require nonzero observation and independent byte and
point/viewport oracles. Test invalidating delimiter/quote/escape/newline changes,
read-only source, Unicode/CRLF, wraps/folds, split views and reload natively.

Dense/deep fixtures include depth 128/256/257, 5000 adjacent pairs (10000
endpoints), one oversized token beyond the byte budget and an offscreen mate.
The result beyond admission is conservative; never scan an arbitrary full buffer
to fill colors. Native controlled clocks test exact pulse expiry and cancellation.

Exactly two representative Linux PTYs cover both profiles and all controls in
one journey each. Reuse the owned eight-second/1 MiB boundary and restore/reap
only their own terminal/child. Hold 96 Down then 12 Up inputs on the frozen
1000-line source; first progress and release-to-final frame must each be within
250 ms. No extra movement occurs during a subsequent 350 ms idle observation.
Record the existing input service (32 events or 4 ms target) and no remaining
pair-owned deadline after pulse expiry; idle CPU may consume at most 3 ticks in
the isolated fixture. Disable unrelated asynchronous services in owned fixtures.
These are isolated acceptance bounds, not a physical workstation latency claim.

## Slice tasks and focused checks

Each task uses one conventional commit when practical. Create one concise report
and one canonical PR per slice. Reports start reviewing; only verified canonical
integration makes them delivered. Keep the exact Summary, Changes, Validation
PR headings, command-only successful validation bullets and final immutable
report-introduction link. Do not replace command evidence with process prose.

## Slice 1: Configuration and view controls

Requirements: DPR03
Dependency: approved committed preparation
Branch: `feat/rev-delimiter-presentation-01-config`
Expected PR title: `feat(editor): configure view-local delimiter presentation`
Report: `ops/default/report/slices/rev-delimiter-presentation/slice-1-config.md`

- T1.1: Extend the existing global and optional language pairs parser, merge, origins, candidate validation and effective defaults with highlight=depth and pulse_ms=250 (50..1000). Add the four optional palette.pairs roles, existing RGB parsing and 1..8 depth colors; resolve absent roles only after the editor palette is merged. Keep editing and visual preferences independent.
- T1.2: Add view-local style/last-non-off state and the unbound pair-highlight-mode and pair-highlight-style commands using the existing bounded prompts, argument admission and session/view owners. New views start from resolved global/language defaults; sibling views retain independent overrides. Preserve point, mark, source, history and settings files. Update literal catalog expectations from 717 to 719 and preserve every existing factory binding.
- T1.3: Create tests/pair_presentation.rs with nonempty config cases for defaults, language inheritance/origins, malformed whole-candidate rejection, palette overrides/fallback, command prompts/cancellation and independent views. Document only the admitted controls; omit a release note for an internal layer without visible rendering.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation config -- --test-threads=1
git diff --check
```

## Slice 2: Qualified presentation facts

Requirements: DPR03, DPV01
Dependency: Slice 1 integrated and closed
Branch: `feat/rev-delimiter-presentation-02-qualification`
Expected PR title: `feat(editor): retain bounded source-qualified delimiter spans`
Report: `ops/default/report/slices/rev-delimiter-presentation/slice-2-qualification.md`

- T2.1: Extend the existing buffer-owned typed-pair fact service with bounded immutable presentation spans and proved depth. Reuse lexical profiles and typed confirmation; never promote loose group targets to typed mates. Foreground editing/navigation requests keep their tickets and scheduling priority; background presentation cannot retire or starve them or introduce a parallel mutable source cache.
- T2.2: Advance retained spans through native source transactions only when exact endpoint mapping and unchanged lexical/nesting classification are proved. Validate source/view identities and invalidate unproved ranges after delimiter, quote, escape, newline, language, reload or replacement changes. Preserve bounded neutral same-line edits without a transient fallback to unqualified syntax.
- T2.3: Test source qualification, malformed-versus-uncertain outcomes, lexical exclusions/eligible quotes, Unicode/CRLF and current/stale cancellation. Enforce aggregate 256 KiB inspection, depth 256, 8192 visible spans per view and 2 MiB retained pair metadata per buffer, including shared/per-view charges. Poll at most 256 tokens under the existing 4 ms target; expose limited results without invented depth. Record measured service and retention data in the report.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation qualification -- --test-threads=1
git diff --check
```

## Slice 3: Depth colors and active pair

Requirements: DPR03, DPV01
Dependency: Slice 2 integrated and closed
Branch: `feat/rev-delimiter-presentation-03-rendering`
Expected PR title: `feat(editor): render delimiter depth and active-pair marks`
Report: `ops/default/report/slices/rev-delimiter-presentation/slice-3-rendering.md`

- T3.1: Project current qualified structural spans through native source/grapheme/fold/wrap mappings. Depth style colors only delimiters with the resolved cyclic palette; off leaves ordinary rendering intact. Reuse existing cells/paint ownership rather than a retained row cache or whole-buffer scan per frame. Preserve Org/Markdown prefix and table projections.
- T3.2: Add active-pair marks using the existing at-point then immediately-before entry policy. Match style affects only the two endpoints; depth adds background/underline while retaining depth foreground. Keep eligible quote foregrounds, selection/search cells, cursor and diagnostic attributes authoritative. Confirmed mismatches may use warning; unfinished/limited source does not. Add lower-priority offscreen mate line/column feedback without moving point/view or replacing prompts/errors. Honor NO_COLOR and supported attributes.
- T3.3: Add rendering cases for independent palette inputs/overrides, repeated depths, lexical exclusions, mismatch/uncertainty, quotes, split views, read-only source, wrapping, wide/combining graphemes, folds and overlay precedence. Assert source/history/point/viewport preservation and complete command/map inventory. Use changelog-normalize for one concise product entry when this visible outcome becomes usable.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation rendering -- --test-threads=1
git diff --check
```

## Slice 4: Finite pulse lifecycle

Requirements: DPR03, DPV01
Dependency: Slice 3 integrated and closed
Branch: `feat/rev-delimiter-presentation-04-pulse`
Expected PR title: `feat(editor): add finite delimiter presentation pulses`
Report: `ops/default/report/slices/rev-delimiter-presentation/slice-4-pulse.md`

- T4.1: Implement pulse style through the existing monotonic run/deadline owner. An accepted input or successful explicit pair motion that identifies an active endpoint starts one generation for the resolved pulse_ms. Rendering, resize, polls and unsuccessful/no-op motion never extend it. Keep key handling nonblocking and do not add a timer thread, service or terminal blink attribute.
- T4.2: Retire or explicitly requalify source/view/point generations on edits, motion, focus/view changes, reload and cancellation. Expired, replaced and off pulses release their marks and pair-owned deadlines; steady depth/match/off own none. Respect the existing admitted view count and preserve unrelated native deadlines.
- T4.3: Add controlled-clock pulse cases for 50/250/1000 ms, replacement, exact expiry, resize, source/point/view changes, cancellation and late results. Check no obsolete mark/deadline, no style-triggered caret/scroll movement and no ordinary keystroke waiting for pulse duration. Do not sleep through production-length durations in deterministic tests.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation pulse -- --test-threads=1
git diff --check
```

## Slice 5: Completed-frame continuity

Requirements: DPV01, DPR03
Dependency: Slice 4 integrated and closed
Branch: `test/rev-delimiter-presentation-05-continuity`
Expected PR title: `test(editor): verify delimiter continuity and input service`
Report: `ops/default/report/slices/rev-delimiter-presentation/slice-5-continuity.md`

- T5.1: Exercise native completed-frame observation for the frozen Text/Org/Markdown forms in both profiles and all four styles. Across 32 neutral body insertions and 32 deletions, retain proved delimiter foregrounds and correct mapped endpoints; preserve headings, prefix geometry, source/selection/cursor mappings and active-pair eligibility. If a real seam fails, correct only its presentation/proof/scheduling owner.
- T5.2: Batch exactly 24 native completed-frame bursts (3 documents x 2 profiles x 4 styles), asserting nonzero observed frames and independent source/style oracles. Add invalidating edits, redraw/resize, split-view divergence, source reload and NO_COLOR cases at the lowest-cost native boundary without multiplying every case into a PTY.
- T5.3: Measure the frozen dense/deep/large-token and held-navigation workloads with existing timing/resource helpers. Assert cooperative limits, cancellation, input/frame scheduling and quiescent retirement. Reuse the completed-frame observer and native input-repeat owners; preserve frame service after 32 events or the 4 ms target. Record observations and limits, not inferred physical-key acceptance.

Required commands:

```sh
cargo fmt --all -- --check
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation continuity -- --test-threads=1
git diff --check
```

## Slice 6: Integrated presentation acceptance

Requirements: DPR03, DPV01; shared editing/navigation and document projection
Dependency: Slice 5 integrated and closed
Branch: `test/rev-delimiter-presentation-06-acceptance`
Expected PR title: `test(editor): verify delimiter presentation workflows`
Report: `ops/default/report/slices/rev-delimiter-presentation/slice-6-acceptance.md`

- T6.1: Add scripts/check-delimiter-presentation.sh and one focused acceptance example using the existing owned controlling-PTY/observer infrastructure. Batch the complete new native presentation suite, existing pairs and pair_navigation regressions, relevant configuration/palette, macro-input and document-continuity selections plus one independent complete catalog/factory-map acceptance. Explicitly reject every empty test filter; do not invoke historical feature scripts or the whole repository test suite.
- T6.2: Run exactly two owned Linux PTYs, one per interaction profile, covering default depth and named match/pulse/off controls, bounded prompt/cancellation, effective command hints, source/point/viewport invariants, save/reopen and held-input stop. Observe completed frames including foreground/mark attributes and the frozen progress/release/idle bounds. Update reference/default examples and the progressive User Guide. Normalize the existing concise Unreleased outcome; do not add delivery chronology or incidental fixes as product entries.
- T6.3: Finish all eighteen task records and six concise reports with exact ordered clean-HEAD check receipts, immutable introduction commits and canonical PR links. Preserve frozen catalog/reference/contract digests and independent expectations of 719 commands with unchanged factory maps. After integration let the controller run the unchanged final gate; only a successful exact-candidate receipt authorizes Markdown-only set/model/spec closure.

Required commands:

```sh
cargo fmt --all -- --check
bash scripts/check-delimiter-presentation.sh
bash scripts/check-documentation.sh
bash scripts/build-user-guide.sh
git diff --check
```

## Controller-owned delivery gate

Run the following unchanged command only after all six canonical slice PRs are
integrated on clean synchronized dev:

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-delimiter-presentation.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'
```

The presentation script executes nonempty selected native batches and one
complete independent map/catalog check, then exactly two owned PTYs. Shared
pair fact changes justify both existing pair suites, not their historical
acceptance scripts. Configuration/palette and macro/document projection cases
protect the boundaries changed here. Do not run all command dispatch tests,
append every previous feature gate or execute a repository-wide test suite by
habit. Workspace fmt/Clippy/source policy and real hook fixtures remain required.
Docs/reference/guide checks and git diff finish the combined gate.

Expected final-gate cost is 20-35 minutes, based on the preceding navigation
gate's 20m48s and comparable workspace/hook compilation. That is an estimate,
not an increased timeout or claim of completion. Reuse checkout-owned caches;
report actual compilation/check progress. Slice workers run only their frozen
focused checks; the controller owns this integrated gate and any permitted
validation repair path. Do not dispatch nightly integration or promote main.

## Reports, integration and closure

Preserve all existing editing/navigation reference data, original factory
catalog/mapping digests and prior receipts. The independent inventory ends at
719 names (717 existing plus the two unbound visual commands) with identical
factory maps. Extend literal expected data, never infer it from Command::ALL.

On a successful clean dev candidate, record exact full-gate command, result,
receipt and source identity in Slice 6 acceptance. Only then mark presentation,
shared model, plan, tracker and six reports Completed/delivered and update the
operations index through the configured exact Markdown closure paths. Distinguish
the tested runtime candidate from later Markdown-only closure. No runtime,
fixture, assertion, frozen contract or gate changes belong in that closure.
Stop after this delivery; main promotion and later work remain separate.
