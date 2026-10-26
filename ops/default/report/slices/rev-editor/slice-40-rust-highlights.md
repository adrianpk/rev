# Slice 40: Bundle Rust syntax highlighting

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-40-rust-highlights`
PR: `#40`
Integrated: `c2c5e08dad8669ac68b7c5b7e0fae868b376c939`

## Purpose

Render Rust through the pinned bundled grammar/query and a small palette while
keeping document bytes, editing ownership and plain-text fallback intact.

## Delivered Behavior

- Canonical case-sensitive .rs destinations use the compiled Rust grammar and
  HIGHLIGHTS_QUERY. The executable needs no runtime grammar installer or network.
  Unsupported extensions and unnamed buffers remain editable Text.
- The seven fixed foreground groups distinguish keywords, strings/escapes,
  comments, types/constructors, functions, constants/numbers and grammar-provided
  punctuation/operators. Unknown capture groups keep the normal foreground.
- The status shows Rust only for a prepared tree of the current revision.
  Missing, invalid or unusable queries fall back to Text. Incomplete source
  remains editable and retains captures that the grammar can still provide.
- Styles use current source-byte captures mapped through complete graphemes
  into visible cells. Tabs, Unicode widths and horizontal clipping retain their
  ordinary coordinates. Clipped wide-glyph fragments remain unstyled blanks.
- Region reverse video and search underlining remain independent of foregrounds.
  Split/tree composition shifts styles into each document area. Status, prompt,
  tree, divider and inventory text retain their normal styling; foregrounds reset
  at row boundaries. The normal NO_COLOR preference remains respected.
- Opening a symlink follows its canonical referent's extension. Unique display
  labels do not affect recognition. Successful rename or save-as updates mode
  from the actual destination. Missing .rs paths need not exist to use Rust mode.
- Parsing and projection preserve rope bytes, dirty/save state, mark, point,
  history and views. Accepted edits invalidate old coordinates immediately;
  session drawing refreshes rendered buffers before borrowing their projections.

## Implementation Notes

- T40.1: `ad71197b9defa146a2ac02b1a824979465e48c2f` adds syntax
  ownership, safe chunk-based parsing/query predicates, byte-qualified captures,
  foreground projection/composition, extension recognition, four native cases
  and the syntax reference/changelog.
- `07faa9a631bae8aea364cefc939a8ca6044ca74b` corrects the initial
  Clippy trim_split_whitespace failure in a test assertion. All seven ordered
  checks then passed: 415 native tests, four syntax tests and 80 terminal cases.
- T40.2: `a916ffa8507896869e172e3679188bbb3ce868ba` adds six native
  cases and two real color-enabled PTY journeys. All seven ordered checks passed:
  421 native tests, 10 syntax tests and 82 terminal cases, with zero skips.
- Syntax owns the retained parser, query, tree and source stamp. Parse callbacks
  and text predicates borrow rope chunks without a whole-document String copy.
  Setup or parse failure changes only derived presentation.
- Refresh happens at most once per rendered revision outside atomic publication.
  This initial implementation reparses changed source from the beginning; exact
  incremental tree-edit translation remains assigned to approved Slice 41.
- Query cursors inspect each row's visible source interval. In-progress matches
  are limited to 4096 and returned styles to 65536 per request; exhaustion
  discards that request's styles. Projected glyph storage is bounded by visible
  cells, and grapheme fragments do not receive a neighboring token's color.
- Native cases exercise the real grammar/query compiler, incomplete code,
  multi-chunk predicate inputs, Unicode cell edges, canonical association changes
  and selection/search/window/tree/inventory surfaces. Missing and failed query
  setup is injected through the same safe Syntax constructor used by the owner.
- PTY color cases explicitly enable foreground output in their owned subprocesses
  because validation can inherit NO_COLOR. They inspect real ANSI colors and
  exact saved bytes, including Unicode, undo/redo and mode changes through save-as.
  Existing finite fixture deadlines and terminal cleanup remain unchanged.

## Contracts Added or Changed

- Buffer syntax is revision-qualified derived state; stale captures cannot color
  another revision. File associations determine Rust recognition independently
  of labels and window identities.
- Safe Tree-sitter parsing and query predicates consume borrowed canonical text.
  Grammar/query setup failure and unsupported files have a usable Text fallback.
- Foreground spans use display-cell coordinates after source-byte projection.
  Existing selection/search attributes compose independently, and replacement
  inventory rows clear document foregrounds.
- The syntax reference, reference index and Unreleased notes describe available
  Rust captures, palette, path recognition and plain-text behavior.

## Files of Interest

- `src/syntax.rs`, `src/syntax/source.rs`, `src/syntax/style.rs`: retained parser,
  revision-qualified captures, borrowed chunk input and fixed capture palette.
- `src/buffer/syntax.rs`, `src/buffer/visits.rs`, `src/buffer/save.rs`,
  `src/buffer/rename.rs`: canonical association selection and derived refresh.
- `src/display/syntax.rs`, `src/display/selection.rs`: complete-grapheme cell
  projection and independent foreground/reverse/underline output.
- `src/display/compose.rs`, `src/display/tree.rs`, `src/display/inventory.rs`:
  style placement across editing panes and workspace overlays.
- `tests/syntax.rs`, `tests/syntax/edges.rs`: ten native syntax cases.
- `scripts/check-pty.py`: real Rust-color and Text/Rust save-as journeys.
- `docs/reference/syntax.md`, `docs/reference/index.md`, `CHANGELOG.md`:
  reader-facing syntax behavior and palette.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 421 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test syntax`: passed; 10 tests, zero skips. Coverage includes
  bundled capture groups, fallback queries, incomplete source, chunk predicates,
  exact bytes/state, current revisions, Unicode clipping and composed surfaces.
- `python3 scripts/check-pty.py --suite terminal`: passed; 82 cases, zero skips.
  New cases verify actual ANSI foregrounds, Unicode, edit/history/save fidelity,
  unsupported Text and extension-aware Rust/Text save-as.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
