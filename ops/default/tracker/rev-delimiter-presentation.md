# Delimiter Presentation Tracker

Status: Approved
Date: 2026-11-15
Delivery set: rev-delimiter-presentation
Slice strategy: behavior-first
Planning authority: product-owner continuation to the final delimiter delivery, 2026-11-15
Approved: product owner, 2026-11-15; presentation after accepted editing and navigation
Execution authority: slices 1-6 and controller-owned predecessor-ordered Rebase + Fast-forward integration into dev
Planning baseline: `744497bab434cc38204f091398c89e21d5147476`
Integration target: `dev`
Plan: `ops/default/plan/rev-delimiter-presentation.md`
Spec: `ops/default/spec/editor-delimiter-presentation.md`
Model: `ops/default/spec/editor-delimiter-pairs-model.md`
Active slice: Slice 1 of 6 - Configuration and view controls
Completed slices: 0/6
Completed tasks: 0/18
Gate state: pending; controller-owned after all six canonical PRs integrate

## Delivery checklist

- [x] Owner continuation authorizes the final delimiter presentation scope.
- [x] Editing/navigation completed; source baseline and existing receipts verified.
- [x] Presentation spec/model approved; frozen contract, plan and tracker aligned.
- [ ] Six canonical PRs integrated into dev and owned worktrees retired safely.
- [ ] Unchanged combined gate passes on exact clean published dev candidate.
- [ ] All reports delivered and Markdown-only closure records the receipt.

## Slice 1: Configuration and view controls

Requirements: DPR03
Dependency: approved committed preparation
Branch: `feat/rev-delimiter-presentation-01-config`
Expected PR title: `feat(editor): configure view-local delimiter presentation`
Report: `ops/default/report/slices/rev-delimiter-presentation/slice-1-config.md`
State: ready
PR: pending
Task commits: pending
Validation: pending

- [ ] T1.1: Extend the existing global and optional language pairs parser, merge, origins, candidate validation and effective defaults with highlight=depth and pulse_ms=250 (50..1000). Add the four optional palette.pairs roles, existing RGB parsing and 1..8 depth colors; resolve absent roles only after the editor palette is merged. Keep editing and visual preferences independent.
- [ ] T1.2: Add view-local style/last-non-off state and the unbound pair-highlight-mode and pair-highlight-style commands using the existing bounded prompts, argument admission and session/view owners. New views start from resolved global/language defaults; sibling views retain independent overrides. Preserve point, mark, source, history and settings files. Update literal catalog expectations from 717 to 719 and preserve every existing factory binding.
- [ ] T1.3: Create tests/pair_presentation.rs with nonempty config cases for defaults, language inheritance/origins, malformed whole-candidate rejection, palette overrides/fallback, command prompts/cancellation and independent views. Document only the admitted controls; omit a release note for an internal layer without visible rendering.

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
State: pending
PR: pending
Task commits: pending
Validation: pending

- [ ] T2.1: Extend the existing buffer-owned typed-pair fact service with bounded immutable presentation spans and proved depth. Reuse lexical profiles and typed confirmation; never promote loose group targets to typed mates. Foreground editing/navigation requests keep their tickets and scheduling priority; background presentation cannot retire or starve them or introduce a parallel mutable source cache.
- [ ] T2.2: Advance retained spans through native source transactions only when exact endpoint mapping and unchanged lexical/nesting classification are proved. Validate source/view identities and invalidate unproved ranges after delimiter, quote, escape, newline, language, reload or replacement changes. Preserve bounded neutral same-line edits without a transient fallback to unqualified syntax.
- [ ] T2.3: Test source qualification, malformed-versus-uncertain outcomes, lexical exclusions/eligible quotes, Unicode/CRLF and current/stale cancellation. Enforce aggregate 256 KiB inspection, depth 256, 8192 visible spans per view and 2 MiB retained pair metadata per buffer, including shared/per-view charges. Poll at most 256 tokens under the existing 4 ms target; expose limited results without invented depth. Record measured service and retention data in the report.

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
State: pending
PR: pending
Task commits: pending
Validation: pending

- [ ] T3.1: Project current qualified structural spans through native source/grapheme/fold/wrap mappings. Depth style colors only delimiters with the resolved cyclic palette; off leaves ordinary rendering intact. Reuse existing cells/paint ownership rather than a retained row cache or whole-buffer scan per frame. Preserve Org/Markdown prefix and table projections.
- [ ] T3.2: Add active-pair marks using the existing at-point then immediately-before entry policy. Match style affects only the two endpoints; depth adds background/underline while retaining depth foreground. Keep eligible quote foregrounds, selection/search cells, cursor and diagnostic attributes authoritative. Confirmed mismatches may use warning; unfinished/limited source does not. Add lower-priority offscreen mate line/column feedback without moving point/view or replacing prompts/errors. Honor NO_COLOR and supported attributes.
- [ ] T3.3: Add rendering cases for independent palette inputs/overrides, repeated depths, lexical exclusions, mismatch/uncertainty, quotes, split views, read-only source, wrapping, wide/combining graphemes, folds and overlay precedence. Assert source/history/point/viewport preservation and complete command/map inventory. Use changelog-normalize for one concise product entry when this visible outcome becomes usable.

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
State: pending
PR: pending
Task commits: pending
Validation: pending

- [ ] T4.1: Implement pulse style through the existing monotonic run/deadline owner. An accepted input or successful explicit pair motion that identifies an active endpoint starts one generation for the resolved pulse_ms. Rendering, resize, polls and unsuccessful/no-op motion never extend it. Keep key handling nonblocking and do not add a timer thread, service or terminal blink attribute.
- [ ] T4.2: Retire or explicitly requalify source/view/point generations on edits, motion, focus/view changes, reload and cancellation. Expired, replaced and off pulses release their marks and pair-owned deadlines; steady depth/match/off own none. Respect the existing admitted view count and preserve unrelated native deadlines.
- [ ] T4.3: Add controlled-clock pulse cases for 50/250/1000 ms, replacement, exact expiry, resize, source/point/view changes, cancellation and late results. Check no obsolete mark/deadline, no style-triggered caret/scroll movement and no ordinary keystroke waiting for pulse duration. Do not sleep through production-length durations in deterministic tests.

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
State: pending
PR: pending
Task commits: pending
Validation: pending

- [ ] T5.1: Exercise native completed-frame observation for the frozen Text/Org/Markdown forms in both profiles and all four styles. Across 32 neutral body insertions and 32 deletions, retain proved delimiter foregrounds and correct mapped endpoints; preserve headings, prefix geometry, source/selection/cursor mappings and active-pair eligibility. If a real seam fails, correct only its presentation/proof/scheduling owner.
- [ ] T5.2: Batch exactly 24 native completed-frame bursts (3 documents x 2 profiles x 4 styles), asserting nonzero observed frames and independent source/style oracles. Add invalidating edits, redraw/resize, split-view divergence, source reload and NO_COLOR cases at the lowest-cost native boundary without multiplying every case into a PTY.
- [ ] T5.3: Measure the frozen dense/deep/large-token and held-navigation workloads with existing timing/resource helpers. Assert cooperative limits, cancellation, input/frame scheduling and quiescent retirement. Reuse the completed-frame observer and native input-repeat owners; preserve frame service after 32 events or the 4 ms target. Record observations and limits, not inferred physical-key acceptance.

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
State: pending
PR: pending
Task commits: pending
Validation: pending

- [ ] T6.1: Add scripts/check-delimiter-presentation.sh and one focused acceptance example using the existing owned controlling-PTY/observer infrastructure. Batch the complete new native presentation suite, existing pairs and pair_navigation regressions, relevant configuration/palette, macro-input and document-continuity selections plus one independent complete catalog/factory-map acceptance. Explicitly reject every empty test filter; do not invoke historical feature scripts or the whole repository test suite.
- [ ] T6.2: Run exactly two owned Linux PTYs, one per interaction profile, covering default depth and named match/pulse/off controls, bounded prompt/cancellation, effective command hints, source/point/viewport invariants, save/reopen and held-input stop. Observe completed frames including foreground/mark attributes and the frozen progress/release/idle bounds. Update reference/default examples and the progressive User Guide. Normalize the existing concise Unreleased outcome; do not add delivery chronology or incidental fixes as product entries.
- [ ] T6.3: Finish all eighteen task records and six concise reports with exact ordered clean-HEAD check receipts, immutable introduction commits and canonical PR links. Preserve frozen catalog/reference/contract digests and independent expectations of 719 commands with unchanged factory maps. After integration let the controller run the unchanged final gate; only a successful exact-candidate receipt authorizes Markdown-only set/model/spec closure.

Required commands:

```sh
cargo fmt --all -- --check
bash scripts/check-delimiter-presentation.sh
bash scripts/check-documentation.sh
bash scripts/build-user-guide.sh
git diff --check
```

## Delivery-set closure

Pending. Record the exact clean runtime candidate, complete successful command,
controller receipt and any subsequent Markdown-only closure separately. A passing
slice check or a current installed binary is not the combined delivery gate.
