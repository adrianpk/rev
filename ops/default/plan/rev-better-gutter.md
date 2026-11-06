# Better Gutter Delivery

Status: Approved
Date: 2026-11-06
Delivery set: rev-better-gutter
Slice strategy: behavior-first
Approved: product owner; specification, three-slice preparation, push and one autonomous Looper delivery through dev
Planning baseline: `2bd63fe65e76bad468c3627c95bbe0e641ab4b99`
Specification: [Better Gutter](../spec/editor-better-gutter.md)
Model: [Better Gutter Model](../spec/editor-better-gutter-model.md)
Tracker: [Better Gutter Tracker](../tracker/rev-better-gutter.md)
Concern: [Pending Work Index](../README.md)
Active slice: None; all three slices integrated, controller-owned final gate pending

## Outcome and scope

Deliver the approved bookmark presentation in one retained Looper run with three
slices. Markers occupy the outside column, before left-side numbers and mirrored
on the right. The startup boolean `gutter.nerd_fonts` selects the compiled square
`■` when false and the compiled flat bookmark `` when true. Its default is false;
the compiled bookmark foreground becomes `#7aa2f7`. Users do not configure a
character, code point or icon name. No font detection or font installation occurs.

Preserve marker visibility controls, logical numbering, checked cell mappings,
wrapping, completion placement, pointer selection, bookmark identity and storage.
No folding column, dormant folding setting, debugger process or breakpoint
records are delivered. BTG02 and BTG03 protect the present absence of those
features and retain their approved future presentation in the specification.

The strategy is behavior-first: Slice 1 delivers complete outside-column layout,
Slice 2 delivers the fixed-symbol configuration path, and Slice 3 verifies their
joint terminal behavior and completes guidance. Every slice stays compilable and
passing. There is no automatic fourth integration or tuning slice.

## Prerequisites and authority

Native numbering, bookmarks, configuration composition, wrapping, pointer input,
Org visibility, both interaction profiles and Markdown-derived Spore publication
are delivered prerequisites. Rust, Cargo, Bash, the existing PTY utilities and
Spore are sufficient; no package, runtime or dependency change is planned.

The owner's startup instruction authorizes the preparation commits, three named
slice worktrees and canonical PRs targeting dev, controller-owned Rebase +
Fast-forward integration, the selected final gate and documentation closure.
Use one retained Looper controller. Do not wait for separate owner merges of
slice PRs. Main promotion, release, installation and mirror publication are
separate operations.

Native recovery may diagnose six incidents, at most two per family, retry two
verified environmental prerequisites and integrate at most two final-gate
repair PRs within this specification. Keep the same run, consumed budgets,
worker conversation, frozen checks and source authority. Dependency changes,
new product decisions, assertion weakening and additional slices are not
delegated. Use the installed controller's defaults of 200 worker turns and
five stalled turns rather than inventing a tighter execution deadline.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Outside marker layout | `feat/rev-better-gutter-01-layout` | `feat(gutter): place bookmark markers before line numbers` | `ops/default/report/slices/rev-better-gutter/slice-1-layout.md` |
| Slice 2 | Fixed symbols and font flag | `feat/rev-better-gutter-02-markers` | `feat(gutter): select built-in bookmark symbols with a font flag` | `ops/default/report/slices/rev-better-gutter/slice-2-markers.md` |
| Slice 3 | Joint acceptance and guidance | `test/rev-better-gutter-03-acceptance` | `test(gutter): verify marker geometry and document presentation` | `ops/default/report/slices/rev-better-gutter/slice-3-acceptance.md` |

## Slice 1: Outside marker layout

Dependency: clean committed preparation. Requirements: BMK03, BTG02, BTG04.

- T1.1: Repartition the existing `src/gutter` geometry so the marker is outside
  the number field, with mirrored right-side placement. Retain number padding,
  separator behavior, whole-field omission, the three-column source minimum and
  independent number/marker visibility. Allocate no folding column or padding.
- T1.2: Integrate the same geometry in frame rendering, view layout, hit testing,
  cursor/selection and completion anchors. Preserve logical source identity
  across wrap, existing Org folds, panes, resize and horizontal presentation.
  Use the current prepared bookmark-row index; add no IO or row-wide store scan.
- T1.3: Extend existing gutter, bookmark, display, pointer and wrapping tests with
  deterministic geometry and source-byte assertions. Cover both sides, hidden
  fields, narrow panes, multiple names, decimal widths and first fragments.
  Update the gutter layout illustration and a single Unreleased outcome as the
  changed presentation becomes usable. Use `changelog-normalize` for that entry.

## Slice 2: Fixed symbols and font flag

Dependency: Slice 1 integrated. Requirements: BMK03, BTG01, BTG04 and model.

- T2.1: Add `gutter.nerd_fonts` through typed parse, merge, provenance, validation
  and immutable startup settings. False selects the compiled square; true
  selects the compiled Nerd Fonts bookmark. Retire the public arbitrary-glyph
  setting. Accept and discard the legacy `bookmarks.marker` string before
  effective publication so old generated configurations remain loadable.
- T2.2: Wire the fixed glyph choice and compiled blue through existing view and
  renderer owners. Retain explicit palette precedence, independent marker
  visibility, split inheritance and source/history invariants. Introduce no new
  font command, icon-name resolver, font probing or configurable glyph path.
- T2.3: Verify both choices, default omission, composed includes, invalid boolean
  types, startup reset behavior and legacy input. Update the complete current
  sample, value catalog and reference to describe the boolean; keep legacy-input
  compatibility in migration guidance. Revise the existing Unreleased bookmark
  outcome rather than add duplicate implementation notes.

## Slice 3: Joint acceptance and guidance

Dependency: Slice 2 integrated. Requirements: BMK03 and BTG01-BTG04 together.

- T3.1: Add a bounded native Rust driver at
  `examples/better-gutter-acceptance.rs`, its owned helpers under
  `examples/better_gutter_acceptance/` and the Bash entrypoint
  `scripts/check-better-gutter.sh`. Reuse existing session/frame and PTY helpers.
  Run one representative journey per profile: set/show/hide a bookmark, resize,
  wrap, select source text, verify source-only copy and close with TTY restored.
  Native cases cover the glyph/configuration variants without multiplying PTYs.
- T3.2: Complete concise Diataxis guidance: bookmark how-to, gutter/configuration
  reference, ordinary User Guide examples and migration from old sample values.
  Build Spore from the Markdown source. Reconcile the named gutter, bookmark and
  configuration baseline/model descriptions without replacing historical gate
  evidence. Keep future folding and debugger semantics explicitly future.
- T3.3: Complete each real slice report, task/commit/PR traceability and evidence
  limits. Hand off the exact integrated candidate to the controller's frozen
  gate. Close the set only after that gate passes; update only authorized
  documentation during closure. Do not manufacture reports while planning.

Nerd Fonts coverage proves emitted code points, geometry and renderer color,
not the appearance of a physical terminal's font. New Rev playgrounds use an
isolated configuration with `gutter.nerd_fonts = true` for screenshots. This
fixture choice does not change the product's false factory default.

## Slice validation

All Cargo checks other than formatting use the existing low-debug profile:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
The local controller configuration records this prefix literally.

| Slice | Required commands in order, with the Cargo prefix above |
| --- | --- |
| Slice 1 | `cargo fmt --all -- --check`; `cargo clippy --locked --lib --bin rev -- -D warnings`; `cargo test --locked --test gutter --test bookmarks --test display --test pointer --test wrapping -- --test-threads=1`; `git diff --check`. |
| Slice 2 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test configuration --test bookmarks --test gutter -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 3 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test gutter --test bookmarks -- --test-threads=1`; `bash -n scripts/check-better-gutter.sh`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |

Update only assertions whose old defaults or coordinates have intentionally
changed; retain explicit legacy preferences when a test is about another owner.
Do not disable tests or substitute weaker assertions to accommodate the layout.

## Final gate

Execute these commands once against the clean integrated candidate, in order.
The environment line applies to all subsequent Cargo and script invocations.
The acceptance script is created in Slice 3; these commands are prospective
delivery obligations, not claims that planning ran the implementation gate.

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test gutter --test bookmarks --test display --test pointer --test wrapping --test configuration --test org_indentation -- --test-threads=1
bash scripts/check-better-gutter.sh
cargo run --locked --quiet --example documentation-check
bash scripts/build-user-guide.sh
git diff --check
```

The new acceptance script builds the editor and its driver with the same
low-debug profile, uses owned temporary HOME/XDG fixtures, checks bounded PTY
output and shutdown, and restores terminal state. It needs no Python and does
not install or invoke a global editor. Do not chain historical Git, terminal,
LSP, Pi, Org or bookmark resource scripts into this focused set's gate.

Existing gate reports identify compilation as the dominant cold cost. The
selected existing suites and two short PTY journeys should remain minutes-scale
with cached builds; this is an estimate, not measured evidence for the new set.
Record actual durations and diagnose unexpected cost without silently raising
timeouts or rerunning unrelated exhaustive suites.

## Closure and publication boundary

Use Conventional Commits without task-number suffixes and the established
Summary, Changes, Validation PR body. The final Changes bullet links each
report through its immutable introduction commit. Only successful commands
appear as Validation bullets. Every slice uses one canonical PR to dev.

Closure may change the plan, tracker, pending index, three delivered reports,
this specification/model, and the directly reconciled gutter, bookmark and
configuration specifications/models. These are exact documentation paths in
the local configuration. No product source or user-facing guide changes occur
after the final gate; such corrections must precede a new validated candidate.

Completion requires all three canonical PRs integrated, reports delivered,
BMK03/BTG01/BTG04 implemented, BTG02/BTG03's current absence protected, the frozen
gate green on the exact candidate and completed artifacts. Future debugger and
folding functionality are not marked delivered merely because their visual
conventions are recorded here.
