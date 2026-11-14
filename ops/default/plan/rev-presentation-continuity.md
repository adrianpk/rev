# Presentation Continuity Delivery

Status: Approved
Date: 2026-11-14
Delivery set: rev-presentation-continuity
Slice strategy: layered
Approved: product owner, 2026-11-14; approved continuity specification and explicit clarification that approval continues the established plan, slicing and autonomous Looper workflow; this five-slice map is derived under that authority
Planning baseline: `5e21c0bb73f21afddf6fcd73be9c5a30a139673e`
Spec: [Org and Markdown Presentation Continuity](../spec/editor-presentation-continuity.md)
Tracker: [Presentation Continuity Tracker](../tracker/rev-presentation-continuity.md)
Active slice: Slice 3 - Markdown presentation continuity
Readiness: approved preparation; controller startup follows publication and native preflight
Full gate: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view::indentation:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test org_indentation --test markdown --test wrapping --test pointer --test display --test core --test coordinates --test editing --test windows -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-presentation-continuity.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`

## Outcome, strategy and authority

Deliver PC01-PC12 and the three modified baseline requirements through five
reviewable layers: checked publication/mapping, Org integration, Markdown
integration, shared lifecycle/resource acceptance, and terminal/documentation
acceptance. Layered slicing is selected because the reserved publication and
coordinate contract must pass before either format consumes it. The first
slice is an internal passing foundation; no intentionally failing or skipped
acceptance assertions may be committed.

The owner's approved scope and clarified established workflow authorize the
preparation commit/push on dev, the exact slice branches and PRs below,
controller-owned Rebase + Fast-forward merges into dev, integrated validation
and closure. Looper is the single execution and merge owner; slice PRs do not
wait for another approval prompt. Main integration, release, binary installation
and mirror publication are outside this delivery.

The spec's same-line scope, eight-row ceiling, charged 32 KiB admission budget,
exact structural authority, conservative lexical fallback and no redraw hold
are fixed. Do not widen Markdown syntax, add dependencies, change fold policy or
rewrite historical delivery evidence. Unknown safety cases reject carry; the
mandatory positive examples must still be admitted.

## Worktree and execution hygiene

Keep the primary checkout on dev. Create implementation worktrees only below
the canonical project worktree root with YYYYMMDD-HHMM-task-slug names.
Each checkout owns its Cargo target and hook cache; use one Cargo job, no
incremental build output and no dev/test debuginfo. Preserve installed hooks,
pre-existing worktrees, ignored data and unrelated uncommitted work.

After each merge, fetch dev and verify the old worktree HEAD is its ancestor,
the worktree is clean/unlocked, no process/build uses it and ignored output is
classified as generated. Remove it with non-forced Git worktree removal before
the next slice. Preserve branches and doubtful or active checkouts. Never stop
a build or editor to enable removal. Do not accumulate five release/debug/hook
target trees until final closure.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Contracts and checked carry | `refactor/rev-presentation-continuity-01-carry` | `refactor(presentation): prepare bounded display carry` | `ops/default/report/slices/rev-presentation-continuity/slice-1-carry.md` |
| Slice 2 | Org presentation continuity | `fix/rev-presentation-continuity-02-org` | `fix(org): preserve headings during ordinary typing` | `ops/default/report/slices/rev-presentation-continuity/slice-2-org.md` |
| Slice 3 | Markdown presentation continuity | `fix/rev-presentation-continuity-03-markdown` | `fix(markdown): retain verified heading presentation` | `ops/default/report/slices/rev-presentation-continuity/slice-3-markdown.md` |
| Slice 4 | Shared lifecycle and resource acceptance | `test/rev-presentation-continuity-04-lifecycle` | `test(presentation): cover carry lifecycle and bounds` | `ops/default/report/slices/rev-presentation-continuity/slice-4-lifecycle.md` |
| Slice 5 | Terminal acceptance and guidance | `test/rev-presentation-continuity-05-acceptance` | `test(presentation): verify continuous Org and Markdown frames` | `ops/default/report/slices/rev-presentation-continuity/slice-5-acceptance.md` |

## Slice 1: Contracts and checked carry

Requirements: PC02, PC03, PC06, PC07, PC11; OGP08, OIN05, MD06 contract reconciliation.
Dependency: approved preparation published and native preflight passed.

- T1.1: Reconcile the approved OGP08, OIN05 and MD06 deltas and the six named model clauses before runtime work. Preserve every inherited scenario, completed status and historical receipt; identify the new continuity scope and its separate pending delivery.
- T1.2: Prepare a private bounded carry transition from the canonical single-change before/after ropes, with exact/base/current identity, at most eight rows, a charged 32 KiB scan budget, checked row/byte deltas and infallible reserved installation. Retain existing owner data rather than cloning whole observations or introducing a second index.
- T1.3: Add passing parameterized mapping/publication regressions under the buffer unit-test namespace for repeated same-row and two-row edits, multibyte source, rejected shapes, overflow and budget exhaustion. Keep existing literal behavior until each format's display integration is delivered; report this internal layer without a changelog entry.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test core --test coordinates --test editing -- --test-threads=1`
4. `git diff --check`

## Slice 2: Org presentation continuity

Requirements: PC01-PC04, PC06-PC11; reconciled OGP08 and OIN05.
Dependency: Slice 1 integrated and closed on dev.

- T2.1: Reuse the Org owner's actual bounded classifier for before/after body admission and recognized-prefix title admission. Preserve ASCII-space recognition and literal ownership; reject boundary/definition changes and unknown cases, with full-analysis comparison and mandatory positive carries.
- T2.2: Integrate the current facade with Org indentation, concealment, marker rendering and palette-derived structural heading faces. Keep exact structural APIs unchanged, current source/hit mappings coherent and geometry identity stable through a proven carry and equal exact replacement.
- T2.3: Extend Org heading/indentation fixtures with all four policy combinations and intermediate-frame continuity assertions, title/body changes, disabled colors, cancellation, stale-hit retirement, Unicode and source fidelity. Preserve existing lifecycle-test intent and add one concise release-worthy changelog outcome when the Org correction is usable.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view::indentation:: -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test org_indentation -- --test-threads=1`
4. `git diff --check`

## Slice 3: Markdown presentation continuity

Requirements: PC01-PC03, PC05-PC11; reconciled MD06.
Dependency: Slice 2 integrated and closed on dev.

- T3.1: Retain admitted complete Markdown base data behind the private adapter. Implement conservative before/after same-line body and ATX-title guards with stable indentation and non-blankness; reject fences, definitions, structural and uncertain contexts. Validate accepted cases against full pulldown-cmark output without introducing another parser or dependency.
- T3.2: Use the current facade for heading/list symbols, structural heading colors and checked capture mappings. Preserve width/indentation, active palette and view policy; dirty changed rows and uncertain lexical context conservatively. Keep list/table actions exact and cancellation literal.
- T3.3: Extend the Markdown fixture suite with mandatory positive body/title carries, unchanged neighboring list markers, definitions and literal-code refusals, monochrome/disabled-color behavior and intermediate-frame/source-fidelity checks. Extend the existing concise changelog outcome to the delivered Markdown correction.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown -- --test-threads=1`
3. `git diff --check`

## Slice 4: Shared lifecycle and resource acceptance

Requirements: PC02, PC03, PC06-PC12; cross-format integration.
Dependency: Slice 3 integrated and closed on dev.

- T4.1: Complete and verify common retirement rules for cancellation, incomplete/limited results, rejected publication, ninth row, reload, undo/redo, buffer close and relevant mode/policy changes. Preserve exact worker/generation checks and current fold-clearing behavior.
- T4.2: Add focused continuity cases in existing geometry/view owners for independent panes, policy transitions, wrapped and horizontally clipped projections, pointer/selection correspondence and equal/different exact replacements. Keep source stamps and unchanged-prefix cache reuse; record actual cache/mapping bounds and no additional timers/workers.
- T4.3: Exercise repeated and two-row pending edits, lexical-context uncertainty beyond viewport edges, resource ceilings and native action-authority refusal. Report accepted/rejected admission counts with positive examples. Any runtime refinements are confined to the approved continuity contracts.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test org_indentation --test markdown --test wrapping --test pointer --test display --test windows continuity -- --test-threads=1`
4. `git diff --check`

## Slice 5: Terminal acceptance and guidance

Requirements: PC01-PC12; joint terminal acceptance and delivery closure.
Dependency: Slice 4 integrated and closed on dev.

- T5.1: Add scripts/check-presentation-continuity.sh as a narrow entrypoint over the existing native/PTY fixture, terminal ownership and cell-decoder infrastructure. Extend the existing Org/Markdown acceptance owners rather than building a new framework. Verify both formats and both input profiles with all intermediate states, real worker completion, colors/source controls, cleanup and exact source preservation.
- T5.2: Run minimal, approximately 50 KB and 500 KB body/title cases, bounded heading-density cases and at least five keys within 180 ms on one/two rows. Record whether work actually remained pending, every observed marker/title-column/foreground state, accepted/rejected counts and measurement limits. Keep graphical claims separate from emitted-terminal evidence.
- T5.3: Update the existing Org/Markdown reference and User Guide text, derive the Spore guide, preserve a concise changelog outcome and complete the slice report. Leave final set completion pending the controller's exact integrated full gate; afterward close only authorized Markdown artifacts and remove this set's integrated inactive clean worktrees.

Ordered slice checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-presentation-continuity.sh`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
5. `git diff --check`

## Validation selection and cost

The final gate protects canonical publication, retained mapping, both format
owners, heading/indentation/Markdown behavior, wrapping, native pointer/display,
basic coordinates/history, shared views and the delivered reference/guide.
It is not the complete repository suite. Unchanged configuration catalogs,
all command families, LSP/provider flows and historical acceptance journeys
are not multiplied into this gate.

New Slice 4 integration cases in the listed native targets must have a
continuity module/name so the frozen focused filter runs real cases. Reports
record counts; zero selected new cases is not acceptance. Mapping unit cases
belong under buffer so their frozen filter also protects the new code.
Do not replace required commands with broader expensive historical gates.

Planning estimate: roughly 15-35 minutes for the combined warm native/terminal
gate, with additional cold compilation cost depending on the checkout.
This is an estimate, not a timeout or a measured result. Record actual build,
native and PTY costs in reports. Previous broad Markdown delivery included
whole command/configuration suites; this set avoids that unrelated repeat
cost. The controller runs the full gate once against the exact integrated
candidate after Slice 5, not during preparation or each earlier slice.

Ordered full gate:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib view::indentation:: -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test org_indentation --test markdown --test wrapping --test pointer --test display --test core --test coordinates --test editing --test windows -- --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-presentation-continuity.sh`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
8. `git diff --check`

## Reports, closure and failure handling

Use one report per canonical row, the global slice-report sections and the
exact Summary / Changes / Validation PR body. The last Changes item links to
the immutable commit containing that slice report. Validation lists only
successful executed commands, required ordered checks first and whitespace
last. Track task commits and actual PRs in the tracker.

Each slice report becomes delivered only after its canonical integration is
verified. The spec, plan and tracker become Completed only after the
controller's exact-candidate full gate succeeds. Preserve unrelated baselines'
historical completed records; their amended contracts point to this delivery
for continuity evidence.

Before freezing configuration, include only the exact plan, tracker, continuity
specification, five slice reports, ops index and the six directly reconciled
baseline/spec-model files in closure_paths. Closure changes are Markdown-only;
guide/changelog/runtime edits must precede the tested candidate. Do not loosen
a frozen gate, reset a checkpoint or silently grant repair budgets after a
failure. Use the installed controller's retained stop and supported recovery
path within actual authority. No successor delivery is queued.
