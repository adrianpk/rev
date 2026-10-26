# Slice 21: Activate and render linear regions

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-21-regions`
PR: pending

## Purpose

Activate buffer-owned linear selections, move their selected-view endpoint and
render the resulting text consistently across rows, scrolling and shared views.

## Delivered Behavior

- `C-SPC` / `set-mark-command` stores point as mark and activates the buffer's
  region. Reported `C-@` and NUL aliases reach the same handler. Movement extends
  or shrinks the ordered range in either direction; empty ranges remain valid.
- `C-x C-x` / `exchange-point-and-mark` swaps the selected view's point with
  the stored mark and activates the region. An unset mark produces a visible
  refusal without changing point or text. Exchange also works after deactivation.
- Only the selected editing pane highlights its current point-to-mark range.
  Shared views retain independent points while sharing mark and activation.
  Selection follows complete grapheme cells, expanded tabs, partial edge spaces,
  logical rows and horizontal/vertical scrolling.
- A selected line break highlights the remainder of its visible row. Status,
  echo and dividers retain ordinary presentation, and buffer inventory removes
  underlying document selection styles from its own rows.
- `C-g` deactivates selection or cancels the pending interaction while preserving
  the stored mark. Successful ordinary edits and undo/redo deactivate the region
  while preserving the established point/mark mapping. A rejected edit preserves
  active selection.
- Native mark commands cannot steal transient input focus. A change to activation
  after edit preparation rejects publication rather than overwriting newer
  selection state. Selection alone does not modify text, dirty state or history.
- Display attributes reset after selected row spans and before leaving the
  alternate screen, including ordinary failure and unwind cleanup paths.

## Implementation Notes

- T21.1: `1e91e88dc3e8f837d80b4ae987d90a3a02f95f64` adds region ownership,
  implemented command bindings, focused rendering and terminal style restoration.
- T21.2: `25bca45422f59fbd9548ab8017fc0e428c42e4c2` adds native and real
  terminal evidence for shared focus, clipping, cancellation and scrolling.
- The buffer owns stored mark and a separate active flag. Checked native region
  APIs validate buffer revision and grapheme boundaries. Range calculation orders
  endpoints from the shared mark and the supplied live view's qualified point.
- Private prepared edits capture activation along with their existing text,
  points and mark admission. Publication checks it before mutation and deactivates
  only after the accepted transaction is recorded coherently.
- Derived rows retain plain text plus an optional terminal-cell selection range.
  Projection converts qualified endpoints once, clips ranges to the viewport
  and translates only the selected pane's spans during composition. Styling
  never becomes document bytes or embedded escape text in row projections.
- The writer applies reverse-video attributes at complete rendered grapheme
  boundaries and clears them before leaving each selected row. The terminal
  owner separately retains and restores display attributes across exit/failure.
- Input decoding delegates both region commands to the same native handlers and
  command names. Their shared buffer state remains separate from view navigation
  and transient prompt/inventory/decision ownership.

## Contracts Added or Changed

- `Buffer::activate_mark`, `region`, `exchange_mark`, `mark_active` and
  `deactivate_region` expose checked linear selection behavior. Storing an
  inactive mark remains supported through `set_mark`.
- `Command::Region` registers `set-mark-command` and `exchange-point-and-mark`
  with working handlers and their canonical key sequences.
- `Row::selection` exposes derived cell ranges independently of plain row text.
  Pane composition translates selected ranges without styling sibling panes.
- Edit publication rejects changed activation and deactivates successful edits;
  keyboard quit preserves the stored endpoint while clearing activation.
- `Terminal` owns display-attribute restoration alongside screen, cursor and
  raw input restoration. `CHANGELOG.md` describes the delivered selection keys.

## Files of Interest

- `src/buffer/region.rs`, `src/buffer/edit.rs`: selection ownership and admission.
- `src/session/region.rs`: focused set/exchange/cancel orchestration.
- `src/command/region.rs`, `src/input/bindings.rs`: names and reported key aliases.
- `src/display/selection.rs`: bounded cell spans and grapheme-safe styling.
- `src/display.rs`, `src/display/compose.rs`: row projection and pane translation.
- `src/terminal.rs`: display-attribute lifetime and restoration.
- `tests/editing/regions.rs`, `tests/commands.rs`: native selection evidence.
- `scripts/check-pty.py`: actual selection keys, shared focus and scroll journeys.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 202 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test editing`: passed; fifty-one tests, zero skips. Seven
  region cases verify ordered/reversed/empty endpoints, exchange/reactivation,
  multi-row styling, edit deactivation, shared focus and point mapping, clipped
  Unicode/tab/emoji cells, preparation admission and transient cancellation.
- `python3 scripts/check-pty.py --suite terminal`: passed; forty-two cases,
  zero skips. New journeys verify NUL mark reporting, Unicode regions, exchange,
  cancellation/reactivation, focused split styling, shared edits, scrolling,
  clipping and zero-size restoration. Failure fixtures also verify attribute
  reset before screen restoration. Owned fixtures stop after their checks finish;
  terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
