# Slice 4: Org projection and bounded wrap lifetimes

Status: drafting
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-04-org-wrap`
PR: pending

## Purpose

Complete WRP04 and WRP05 with shared Org projection geometry, bounded layout
lifetimes and measured long-line control. T4.1 is implemented at `ba7233b763b514be0c0fb0cf205abd2c4ae4c227`; T4.2 is active.

## Delivered Behavior

- Visible Org headings, table fields and source-block lines use the selected
  buffer wrap policy. Native fold requests retain hidden source rows across
  resize and wrap changes without inserting display newlines into source.
- Table-special navigation retains logical field identity, including long
  fields, tabs, combining/wide graphemes and CRLF. Alignment remains an explicit
  ordinary source edit with exact undo.
- Native source drafts and their parent both wrap. Exiting a draft publishes
  one canonical parent edit while preserving delimiters and unrelated text.
- Image rectangles begin at the actual first visible source fragment after
  preceding wrapped text. Reflow changes placement; folded or scrolled-past
  anchors receive no rectangle. Empty preview sets require no wrap traversal.
- Repeated navigation, hit testing and painting reuse checked source and
  fragment observations. Actual split views own independent fragment caches;
  disposing or evicting observations preserves source and visible text.

## Implementation Notes

T4.1 routes optional preview placement through the same current fragments used
by text, hit testing and gutter labels. Existing preview resource admission,
source stamps, clipping, byte/pixel limits and explicit IO ownership remain in
place. Fold, field and draft commands retain their canonical source owners.

The existing folded-Unicode hit fixture now explicitly covers both factory wrap
and the retained horizontal policy. It asserts complete grapheme destinations
and rejects the appropriate padding/partial-glyph cells in each mode.

T4.2 now retains at most 8192 fragments and 1 MiB of projected text per view,
and 1024 coordinate checkpoints per buffer. Coordinates sample every 256
glyphs and remember requested boundaries. A 64-publication journal proves the
unchanged source prefix across edits, undo and redo; missing proof discards the
derived observations. Fragment validity includes the glyph lookahead that
chooses word breaks and the complete CRLF/LF terminator, not just painted text.
Width, tab policy, effective wrap policy and source identity qualify all reuse.

Candidate view snapshots share disposable observations, which do not enter
semantic view equality. A real split starts a separate cache. Fold projection
stays current rather than caching screen rows. Glyph restarts retain the full
source-line context for regional indicators, ZWJ and combining clusters.
Rectangle painting scans the visible column range while copy and kill retain
the full canonical rectangle. Syntax, clipping and pointer conversion use the
same checked coordinate restarts.

## Contracts Added or Changed

Preview anchor rows follow the current visual projection. Pictures neither
create source rows nor acquire another anchor on a continuation. The accepted
buffer policy applies to Org source and tables without an implicit exception.

Retention limits and checked restarts are implemented. Cold misses still scan
synchronously; the required bounded resumable work contract is not complete.

## Files of Interest

- `src/display/preview.rs`: visual-fragment image placement and empty-set fast path.
- `tests/wrapping/org.rs`: native folds, logical fields, source drafts, preview
  reflow and source/history preservation.
- `tests/org_editing/fold_edges.rs`: paired wrap/no-wrap folded Unicode hits.
- `examples/wrap-performance.rs`: finite native API baseline probe with exact
  source restoration and idle-deadline assertions; no terminal gate replacement.
- `src/view/wrap/cache.rs`: bounded, source-qualified fragment retention.
- `src/coordinates/checkpoints.rs`: bounded coordinate and publication-prefix
  observations, independent of view width.

## Validation

- `cargo test --locked --test wrapping org::`: passed, four native Org wrapping cases.
- `cargo test --locked --test org_io previews::`: passed, three existing preview regressions.
- `cargo test --locked --no-fail-fast --test org_editing --test org_tables --test org_source`:
  table/source suites passed (73 and 81 tests). One historical horizontal-only
  folded-hit expectation failed among 226 Org editing tests; its paired
  correction is checked with the focused command below.
- `cargo test --locked --test org_editing fold_edges::unicode_hits`: passed after
  the fixture correction. Final configured checks wait for completed T4.2 and
  PR-reference publication.

No full delivery gate or complete wrap acceptance is claimed by this draft.

The native override lifecycle case also passes: both startup defaults survive
unequal splits, a buffer-local toggle, another buffer, close and revisit under
a new source identity. Source files and the configuration file stay unchanged.

`cargo run --locked --quiet --example wrap-performance` passed its development
baseline assertions on this worktree. It uses the debug profile, one in-memory
buffer per case, eight alternating visual motions, four resizes, eight typed
characters and eight backspaces, with a frame after each event. These are
maximum observed native API latencies, not terminal input/backlog measurements
or accepted final ceilings:

| Case | Bytes / scalars | Cold frame ms | Motion ms | Resize ms | Input ms | Reversal ms |
| --- | --- | --- | --- | --- | --- | --- |
| Token | 65536 / 65536 | 991.930 | 2538.942 | 1002.903 | 999.844 | 1085.815 |
| Words | 62244 / 62244 | 1166.805 | 2282.533 | 855.795 | 789.813 | 811.892 |
| Unicode/tabs | 61440 / 49152 | 848.942 | 1354.143 | 681.149 | 647.380 | 651.291 |

Every case restored the exact source bytes and canonical point, had no remaining
input deadline, and returned Wait on a later expiry. This proves source
restoration and absence of scheduled idle work; it does not measure idle CPU.
The observed prefix costs are baseline failures to resolve under T4.2.

The same development probe after adding fragment and coordinate reuse observed:

| Case | Cold frame ms | Motion ms | Resize ms | Input ms | Reversal ms |
| --- | --- | --- | --- | --- | --- |
| Token | 191.594 | 1.157 | 95.238 | 1.090 | 1.017 |
| Words | 168.992 | 1.195 | 89.385 | 1.143 | 1.063 |
| Unicode/tabs | 119.912 | 1.125 | 61.425 | 0.841 | 0.779 |

These intermediate debug-profile measurements retain the same probe limits;
they establish improved warm paths, not accepted cold-work ceilings. The later
CRLF invalidation correction is checked separately.

Development regressions passed after the cache changes: 24 coordinate tests,
26 wrapping tests and 341 display/pointer/gutter/rectangle/search/window/editing
tests. After bounding rectangle paint traversal, all 42 rectangle tests and the
cross-chunk Unicode restart case passed again. The focused CRLF terminator
invalidation case also passed. All-target Clippy passed before that final
terminator change; fresh clean-HEAD evidence is required for this increment.

## Risks and Follow-ups

T4.2 must make cold source/fragment misses resumable under explicit work
ceilings. Current synchronous misses include geometry changes, far seeks,
early source edits and cache eviction. Bounded retention and fast warm reuse
do not establish an input or cancellation latency bound for those misses.

Complete native cold-work cancellation, input/resize/reversal and idle CPU
measurements, exact successful final validation, and the approved long-line
ticket transition before preparing the canonical PR.
