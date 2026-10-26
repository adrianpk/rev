# Slice 41: Update syntax from exact rope edits

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-41-incremental-syntax`
PR: `#41`
Integrated: `09840bfebb3a4fdeb041a458d7e502790e8e5142`

## Purpose

Advance retained syntax trees from exact accepted rope edits, preserving current
capture coordinates and isolating parser/query failures from text and history.

## Delivered Behavior

- Each admitted transaction prepares Tree-sitter InputEdit records from its
  actual base ranges and inserted fragments. Byte offsets and LF-based row/byte
  columns cover multibyte text, CRLF, removed separators and inserted newlines.
  Disjoint edits publish in the same descending base order as the rope candidate.
- The shared publication owner handles ordinary edits, typing contributions,
  paste, rectangle rows and undo/redo. Typing history grouping does not replace
  the current contribution with a coarse group span. Canceled, read-only and
  stale preparations leave the existing syntax tree intact.
- Publication adjusts retained tree coordinates and invalidates captures.
  Several accepted revisions can accumulate before a parse. Refresh reuses a
  tree only when its coordinate stamp matches current source, and repeated
  redraws do not parse the same revision again.
- A parsed revision has Rust status; obsolete captures cannot color accepted
  new text. Incomplete source remains usable. A failed or canceled parse clears
  its tree and parser continuation and shows Text while preserving all accepted
  text, history, marks and shared points. A later accepted revision can recover
  from current source and resume incremental reuse.
- Queries remain restricted to visible row source intervals, with the existing
  match/result bounds. A request that exceeds its result bound discards partial
  styles without changing source or preventing later small queries.
- Current foregrounds remain aligned through Unicode/CRLF paste and multi-row
  rectangle kill/yank, including undo/redo. Region reverse video, search underline,
  shared-pane points and cursor placement remain independent of syntax styling.

## Implementation Notes

- T41.1: `41d9253f826832cdcabce9a5184922924acfb421` adds prepared
  byte/point edits, retained tree-coordinate revisions, incremental parser seeds,
  real progress-aware cancellation, four native cases and reference/changelog
  updates. All seven ordered checks passed: 425 native tests, 14 focused syntax
  tests and 82 terminal cases.
- T41.2: `6bd57189b9565dd936af46c9104d207419929bfb` adds six native
  failure/surface cases, extends compound newline equivalence and adds two real
  PTY journeys. All seven ordered checks passed: 431 native tests, 20 focused
  syntax tests and 84 terminal cases, with zero skips.
- Input-edit preparation completes before publication. After admission/history
  publication succeeds, safe Tree::edit updates the retained seed without parser
  callbacks or another recoverable operation. Rendering later parses borrowed
  rope chunks; it does not construct a whole-document String.
- Parsed/capture freshness and tree-coordinate freshness use separate stamps.
  Tree coordinates can advance across several mutations while captures stay
  invalid. A source mismatch discards the seed; a current edited tree is passed
  to Tree-sitter for incremental parsing. The public diagnostic reports whether
  the last successful parse actually received that seed.
- A progress-aware refresh accepts the dependency's real ParseOptions. Parser
  cancellation resets continuation state and clears the failed tree. A failed
  revision is not retried on every redraw. No mock parser, fake runtime option,
  unsafe code or extra dependency is introduced.
- Native evidence asserts exact InputEdit fields and descending multi-range
  order, then compares actual incremental captures with an independent full
  parse after hidden revisions, Unicode/newline changes, grouping and history.
- Failure cases trigger a real Tree-sitter progress cancellation after accepted
  shared text, and the actual query result bound on a larger source. They verify
  retained state/points/marks, unstyled fallback, undo recovery and resumed reuse.
  Other cases cover rejected/stale candidates, visible ranges and shared overlays.
- The focused syntax suite completes in about one second locally. Two added PTY
  journeys reuse existing owned fixtures and finite eight-second deadlines. They
  inspect real current foregrounds and exact saved bytes through shared paste,
  region/search layering, rectangle operations and their history replay.

## Contracts Added or Changed

- Prepared mutations expose exact descending base-space syntax edits. Source
  offsets use UTF-8 bytes; row columns count bytes since LF, independently of
  Unicode scalar or terminal-cell coordinates.
- Only accepted publication advances retained tree coordinates. Captures require
  a successfully parsed matching revision; stale or failed seeds cannot color
  unrelated current text.
- Parser cancellation is a presentation failure after accepted text publication.
  It preserves canonical editing/history ownership and cannot become a rejected
  edit or an implicit rollback.
- The syntax reference and existing Unreleased entry describe incremental
  revision behavior, failure isolation and aligned colors through compound edits.

## Files of Interest

- `src/buffer/edit/syntax.rs`: InputEdit preparation and byte/point translation.
- `src/buffer/edit.rs`: prepared syntax changes and admitted publication.
- `src/syntax.rs`: coordinate/parsed stamps, retained seed edits, incremental
  refresh and failure cleanup.
- `src/buffer/syntax.rs`: real progress-aware parser refresh at the buffer owner.
- `tests/syntax/incremental.rs`, `tests/syntax/failures.rs`: ten new native cases
  and independent full-parse equivalence checks.
- `scripts/check-pty.py`: shared Unicode/CRLF and rectangle syntax journeys.
- `docs/reference/syntax.md`, `CHANGELOG.md`: delivered incremental behavior.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 431 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test syntax`: passed; 20 tests, zero skips. New coverage checks
  exact input edits, retained seeds, independent capture equivalence, grouping,
  compound rows, real cancellation, query limits, stale admission and shared views.
- `python3 scripts/check-pty.py --suite terminal`: passed; 84 cases, zero skips.
  New cases drive Unicode/CRLF paste, shared foreground/region/search output,
  rectangle kill/yank, atomic undo/redo and exact file saves.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
