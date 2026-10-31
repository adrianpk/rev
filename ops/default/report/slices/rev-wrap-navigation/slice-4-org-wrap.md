# Slice 4: Org projection and bounded wrap lifetimes

Status: delivered
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-04-org-wrap`
PR: [#177](https://forge.adrianpk.com/adrianpk/rev/pulls/177)

## Purpose

Complete WRP04 and WRP05 with shared Org projection geometry, bounded derived
layout lifetimes and measured native long-line control.

## Delivered Behavior

- Visible Org headings, table fields and source-block lines use the buffer's
  wrap policy. Folds retain hidden source rows through resize and wrap changes.
- Field navigation keeps logical identity across long fields, Unicode, tabs
  and CRLF. Explicit alignment and source-draft publication remain canonical
  source edits with exact undo and preserved delimiters/unrelated text.
- Preview rectangles start at the actual first visible source fragment.
  Continuations, hidden anchors and empty preview sets add no duplicate image.
- The runtime override follows the buffer through unequal splits and changes
  of selected buffer. Closing and revisiting restores the startup preference
  under either configured default, without writing configuration or source.
- Cold native projection, visual motions/kill, coordinate misses and pointer
  preparation yield through native deadlines. Pending surfaces expose no source
  cursor or pointer destination. Far point reversal abandons obsolete reflow.
- Later native input waits in order behind an unfinished pure layout command.
  C-g cancels its uncommitted effect, retains earlier typed input and clears
  queued chord state. Native backpressure leaves unread events at their source.
- Text, gutter, cursor, previews and search share one current row map. Eviction
  never truncates source, including a visible canvas exceeding the cache cap.

## Implementation Notes

Per-view retention is capped at 8192 fragments and 1 MiB of projected text.
Each buffer keeps 1024 coordinate checkpoints, one paused coordinate boundary
and a 64-publication unchanged-prefix journal. Reuse checks source identity,
width, tab/wrap policy, word-break lookahead and the complete LF/CRLF terminator.
An edit, undo or redo without sufficient prefix proof discards observations.
Real splits own independent fragment caches; speculative view copies share
only disposable observations, excluded from semantic view equality.

One resumable line builder per view retains a segmentation cursor, source
coordinates and at most one pane-width of lookahead. A cold projection budget
allows 4096 new glyph reads, shared across visible buffers/views within that
scope. Coordinate scans retain a completed boundary even when visible rows
have evicted their ordinary samples. Local visible-row construction/painting
costs the current canvas area; it does not traverse the rest of the document.
Native event servicing also retains its 32-event/4 ms inter-event allowance.
The deferred FIFO holds 256 ordinary events plus one reserved interrupt.
These are internal work/retention limits, with no new configuration knobs.

Known-boundary segmentation restarts follow the next-boundary premise in
[UAX #29 section 6.3](https://www.unicode.org/reports/tr29/#State_Machines).
Continuous versus restarted/resumed comparisons cover RI parity, ZWJ,
combining, Indic, Prepend, Hangul, tabs and rope seams. Arbitrary public scalar
positions retain checked boundary admission; observed native points use the
qualified forward observations. This evidence is not full Unicode conformance.

Layout commands first compute a pure candidate. Source/point changes reject a
held command before effects; resize recomputes it at the same source point.
Pointer observations additionally require unchanged views and geometry.
Org structural requests first settle their current reveal before pinning the
view; automatic reflow cannot cancel a source-identical operation. Queued
Escape chords retain arrival times even when resize or host expiry intervenes.
Source completion callbacks execute outside projection budgets, so a derived
layout yield cannot interrupt their canonical publication. Existing edit,
undo, kill, Org request and preview resource owners remain authoritative.

`Session::poll_frame` serves native bounded work. `Session::frame` retains
complete synchronous embedding behavior with the same derived owners.
Completion removes the layout deadline; no idle polling scanner is introduced.

## Contracts Added or Changed

Org source and tables follow the accepted buffer policy without implicit
no-wrap exceptions. Images remain projections, never source rows.

Layout observations have explicit retention, work, cancellation and source
qualification boundaries. Source-based rectangle edits remain logical; only
the visible column range is traversed for rectangle painting.

## Files of Interest

- `src/view/wrap.rs` and `src/view/wrap/cache.rs`: checked fragments,
  continuations, retention and bounded lookup.
- `src/coordinates/`: shared work scopes, coordinate checkpoints, observed
  boundary restarts and exact painted/EOL hit admission.
- `src/session/layout.rs` and `src/session/events.rs`: pure preparation,
  finite input retention, cancellation, backpressure and deadlines.
- `src/display.rs`: shared frame map; `src/display/preview.rs`: image placement.
- `tests/wrapping/org.rs`: native folds, fields, drafts, previews and override
  lifetime; `src/session/layout/tests.rs`: cold input and stale operations.
- `examples/wrap-performance.rs` and `scripts/check-wrap-performance.py`:
  finite native API and real PTY measurement drivers using existing owners.

## Validation

T4.1 was committed at `ba7233b763b514be0c0fb0cf205abd2c4ae4c227`.
Intermediate cache and frame increments were validated by controller receipts
`OIPQPYEYATXCWWN3RGGM5JJU4Q` and `CQDNH3ES5XDBYSZEIBO466AJMS`.
Implementation candidate `034b721bb110461e7106cdea970564548934d009` passed
all eleven controller commands in receipt `JNLPDE7N5VZQHLQ33J3U6447EA`:

- `cargo fmt --all -- --check`
- `cargo test --locked --test wrapping --test org_editing --test org_tables --test org_source --test pointer --test configuration`
- `git diff --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --lib coordinates::`
- `cargo test --locked --lib view::wrap::`
- `cargo test --locked --lib session::events::`
- `cargo test --locked --lib session::layout::`
- `cargo test --locked --test display --test coordinates --test rectangles --test search --test gutter --test editing --test windows --test core`
- `cargo run --locked --quiet --example wrap-performance`
- `python3 scripts/check-wrap-performance.py`

The configured integration suites passed 596 tests; focused helper and affected
integration suites passed another 346. Final PR-reference head `279242c0351ce10d2415d97c0aa11f1196b0b63c` passed
the exact three configured commands above in controller receipt
`K6BO6PQAFLAXYANN27HMXLOHZG` (596 tests). PR #177 merged that exact head
through Rebase + Fast-forward into `dev` on 2026-10-31.

Development checks passed for coordinates, continuous/resumed wrap partitions,
queued input and cancellation, native event owners, and display/gutter/search/
wrapping regressions. Added cases cover checkpoint eviction during a batch,
changed source/geometry, and a canvas taller than retained fragment capacity.
The successful controller receipt includes all-target Clippy and the final
EOL-hit, Org admission and queued Escape timing corrections.

`cargo run --locked --quiet --example wrap-performance` uses the debug profile,
in-memory sources, a finite 512-batch driver, eight alternating visual motions,
four resizes, eight typed/reversed characters, 64 queued typed/reversed pairs
and far point reversal. Each native handler, frame and deadline is measured
separately. Controller receipt observations were:

| Fixture | Bytes / scalars | Cold ms | Max resize ms | Queued reversal ms | Largest native step ms |
| --- | --- | --- | --- | --- | --- |
| Token | 65536 / 65536 | 107.022 | 99.695 | 238.424 | 6.879 |
| Words | 62244 / 62244 | 87.342 | 93.553 | 198.153 | 7.175 |
| Unicode/tabs | 61440 / 49152 | 73.977 | 65.286 | 169.197 | 6.708 |
| RI flags | 65536 / 16384 | 25.145 | 20.974 | 148.156 | 10.107 |
| 40000 source lines | 720000 / 720000 | 10.241 | 2.036 | 33.001 | 3.641 |

All restore exact source bytes and point, then have no remaining native
input deadline and return Wait on a later expiry. The initial synchronous
baseline took 848-1167 ms for a cold frame and 681-1003 ms for resize on the
first three fixtures; warm navigation took up to 2539 ms.

`python3 scripts/check-wrap-performance.py` builds the locked release binary
and reuses the repository's real controlling-PTY and idle measurement helpers.
It exercises default visible gutters, a far point, four widths, 128 typed
characters and 128 backspaces, a cold resized C-e/input/reversal/C-g burst,
exact save and restored terminal lifetime. Controller receipt observations were:

| Fixture | Far ms | Max resize ms | 128 input ms | 128 reversal ms | Cold queued reversal ms |
| --- | --- | --- | --- | --- | --- |
| Token | 6.574 | 7.876 | 2.759 | 2.755 | 6.662 |
| Words | 6.249 | 7.151 | 2.721 | 2.898 | 5.693 |

Each 350 ms idle observation emitted zero bytes and consumed zero measured
process CPU ticks. Both saved files matched their original bytes exactly and
both terminal sessions restored their original attributes and exited normally.
The first probe attempt incorrectly expected byte-restoring edits to clear
historical dirty state; the corrected driver requires dirty state until save
and retains the exact source, cursor and exit assertions.

The first configured controller check at
`0fe5431e9ee449986bb56c3aadb669513c941331` failed two Org cases. The tag
observation-limit case exposed a real request/view race, fixed by bounded
reveal before structural admission. The rejection fixture now observes settled
presentation before capturing its baseline. Its original source/view/stamp/
mark/history assertions and the exact 8192/8193 observation limits remain.
Both focused regressions pass, along with queued Escape timing across resize
and delayed expiry. No acceptance or resource limit was weakened.

These are local fixture measurements, not universal latency ceilings. The full
delivery gate and combined file-navigation acceptance remain controller-owned.

## Risks and Follow-ups

One extended grapheme can contain arbitrarily many bytes. Glyph-read budgets
bound derived traversal counts, not its intrinsic segmentation/allocation time
or existing canonical source-edit admission. Visible canvas work also scales
with terminal geometry. The finite measurements do not promise fixed latency
for arbitrary text or hosts.

PR #177 is merged and the wrap ticket is solved. The combined delivery gate
remains owned by the final acceptance phase.
