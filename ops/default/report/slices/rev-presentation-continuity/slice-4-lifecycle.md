# Slice 4: Shared lifecycle and resource acceptance

Status: reviewing
Delivery set: rev-presentation-continuity
Plan: `ops/default/plan/rev-presentation-continuity.md`
Tracker: `ops/default/tracker/rev-presentation-continuity.md`
Branch: `test/rev-presentation-continuity-04-lifecycle`
PR: `#44`

## Purpose

Verify shared carry retirement, current geometry and bounded owner state under
PC02, PC03 and PC06-PC12. Terminal acceptance and guidance remain Slice 5.

## Delivered Behavior

Eighteen new native regressions verify both formats through undo/redo, reload, mode and syntax
policy changes, explicit cancellation and re-enablement, close and fresh-base
recovery. Current incomplete/canceled results retire presentation; obsolete
publications preserve a newer admitted chain. Markdown worker polling rejects a
previous generation and a closed buffer, and clears its owned slot/deadline.
The existing Org worker accepts an explicit retry queued before canceled work
is reaped. No runtime behavior or production storage is changed by this slice.

Source publication still clears folds. A carried Org heading supplies display
only: a new fold request waits for exact native structure. Existing Markdown
list/table and stale-publication authority regressions remain intact.

Three panes with three widths preserve separate pretty choices and current
points through five pending edits on two rows in each format. Shared Org
indentation changes immediately. Wrapped and horizontally clipped projections,
Unicode hit boundaries, native drag/copy and selection foreground all use current
source. Offscreen opening and closing delimiters on either viewport edge retire
the entire uncertain cross-line lexical capture while structural headings carry.

## Implementation

T4.1, T4.2 and T4.3 are delivered in `7f1f834fe8de4dde5c650039e3eafeece338430f` as one cohesive
cross-owner test commit. Five new buffer cases cover common retirement and fixed
mapping storage. Two session cases exercise Markdown's actual result polling;
two cache cases verify prefix reuse, exact replacement and eviction. Nine new
integration cases extend the existing heading, indentation, Markdown, pointer,
window, wrapping and display owners. No new fixture framework is introduced.

## Bounds and Positive Evidence

- The existing format admission matrices pass with Org 4 accepted / 10 rejected
  and Markdown 7 accepted / 21 rejected. Plain body edits and recognized-prefix
  title edits remain positive cases; definitions, literal blocks, structural
  delimiters and uncertain contexts reject.
- Generated full-owner comparisons admit 64 revisions per format. The new
  sustained case admits 512 Unicode insertions/deletions per format on two rows;
  zero-net rows remain dirty and keep one base stamp and two fixed slots.
- Native row exhaustion admits eight distinct body rows per format (16 accepted),
  rejects each ninth row (2 rejected), and refuses both following gap edits.
  Editing/history continue; fresh complete evidence permits carry again.
- Measured `Mapping` and `Option<Mapping>` are each 304 bytes on this build, with
  capacity eight. This is buffer-level storage, independent of the key count;
  it is separate from per-view cache accounting. The charged 32 KiB inspection
  ceiling and checked-overflow regressions remain enforced.
- Both format caches preserve four unchanged-prefix fragments after the edit and
  all six populated fragments across an equal exact replacement. A structural
  edit rejects old hits and a different exact layout starts with zero retained
  entries. Source stamps remain mandatory throughout.
- Four carried edits over 9,000 short body rows at widths 1 and 512 reach, without
  exceeding, 8,192 retained fragments and 1,048,576 retained text bytes. Virtual
  padding is included in the existing text accounting. These measurements cover
  retained cache text/mapping, not process RSS or an input-latency guarantee.
- Pointer regressions compare 24 wrapped/clipped projections and every native
  cell hit before/after equal exact replacement. Drag/copy checks both native
  formats against their current canonical scalar ranges, including punctuation
  and Unicode. No generated display marker enters copied source.

All production edits are absent: only test modules and test functions are added.
The single retained base, existing parser dependencies, worker slots, deadlines,
fold policy, event scheduling and cache ceilings remain the existing owners'.
Tests use native jobs and projections; they add no shipping worker or timer.

## Validation

Local pre-submit required checks passed in canonical order:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: formatting, complete Clippy and native source policy passed across 1,898 first-party Rust files, two packages and 75 Cargo roots; zero blocking/deferred diagnostics. Cold Cargo check took 1 minute 40 seconds; final warm check 0.55 seconds.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib buffer:: -- --test-threads=1`: all 29 passed, including five new shared lifecycle/bounds cases; no ignored tests. Final warm Cargo preparation 0.06 seconds, native execution 2.93 seconds. Initial cold test build took 2 minutes.
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test org_indentation --test markdown --test wrapping --test pointer --test display --test windows continuity -- --test-threads=1`: all 23 selected cases passed, no failures or ignored tests (8 Org headings, 7 Markdown, 4 pointer, and one each indentation, wrapping, display and windows). Final warm Cargo preparation 0.05 seconds; native execution totals 23.86 seconds across the seven executables.
4. `git diff --check`: passed.

Focused checks also passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib continuity -- --test-threads=1 --nocapture`: four new native job/cache cases, 6.59 seconds execution; records the retained prefix and cache measurements above.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test syntax current_revision -- --test-threads=1`: the existing unrelated Rust source-stamp/style refresh regression passed, 0.35 seconds execution; canonical bytes, state and history remain unchanged by redraw.

Development checks corrected test-only private API calls, a borrowed/owned row
collection type, wide-cell decoder sampling and the exact three-width geometry
expectation for an odd pane split. No runtime fix, limit increase, ignored test
or reduced acceptance assertion was needed. Vendored crossterm retains its
existing unused-parentheses warning.

The installed hook passed formatting, full Clippy, complete source policy and
index-integrity checks against the immutable staged implementation tree
`f3afdbad77bae1a75cc2b0fa7babe0bead500edd`, with an exclusive private cache.
Its cold Cargo check took 1 minute 48 seconds. Final clean-HEAD evidence will be recorded once through
the controller validator after report/PR-reference commits and the push.
Native fixtures finish and release their own resources; targets and hook output
remain exclusive to this owned worktree.

## Risks and Follow-ups

This slice deliberately observes pending display by projecting without starting
replacement analysis, and also exercises real owned cancellation/retry and poll
publication. It does not claim sustained real-terminal input timing or graphical
acceptance. Slice 5 owns the PTY matrix, larger-file input observations and guide
updates. The controller owns integration and the exact delivery-set gate.
