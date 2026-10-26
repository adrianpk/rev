# Slice 29: Search forward across rope chunks

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-29-forward-search`
PR: `#29`
Integrated: Rebase + Fast-forward into dev at `9e7decac0237df6877707fee780801b858487fa2`.

## Purpose

Deliver incremental forward literal search with a visible query, qualified source
results and selected-pane presentation while preserving canonical rope content.

## Delivered Behavior

- C-s and M-x isearch-forward share one search handler. Query edits immediately
  find the first literal occurrence from the search anchor; C-s continues after
  the current result. Matching is case-sensitive and does not wrap.
- The minibuffer owns query input, including grapheme editing, literal Tab and
  bracketed multiline Unicode/CRLF paste. Queries are bounded at 4096 scalars;
  rejected insertions preserve the draft and display the existing limit message.
- Matches retain exact buffer/revision/scalar ranges. Point moves to the match
  end, adjusted to a full grapheme, and normal reveal brings it into view.
  Underline marks the selected pane's result independently of region reverse
  video, with complete grapheme edges and clipped multiline projection.
- A missing match clears result styling, retains a valid point and shows
  `No match` beside the query. Editing can recover a result. Return closes
  search at point; C-g drops the interaction and restores its starting view.
- Search does not publish text or alter logical state, modification metadata,
  stored marks or undo/redo transactions. A later source publication invalidates
  its old result and closes search with `Search source changed` on redraw/update.

## Implementation Notes

- T29.1: `65df69161fa93616a191f50f2c70bfdcad22cf98` adds the bounded
  literal matcher, focused query owner, command/binding, session result state,
  selected-pane underline projection, three native behavior tests and reference.
- T29.2: `3ba1af483b66be0caf17f6d676d0c9ca8157a9bc` expands coverage
  to eleven native tests and two real PTY journeys for chunk/grapheme seams,
  multiline queries, editing/recovery, stale results, bounds and larger ropes.
- The KMP scan walks borrowed RopeSlice chunks, retaining prefix progress across
  seams. Its only source-independent allocations are the bounded query and
  fallback table: O(query) storage and O(scanned scalars + query) matching work.
  Inspection confirms no source flattening in the search owner or matcher.
- Larger fixtures exercise a 1 MiB long line and approximately 1.25 MiB of
  Unicode/tabbed lines, repeated incremental scans near EOF and a 4096-scalar
  repeated-prefix miss. Exact source content, stamps and history remain intact.
- PTY checks use actual key reports, bracketed paste, complete-frame waits and
  resize handshakes. Owned temporary fixtures close after their assertions;
  accepted exit verifies terminal restoration and unchanged disk bytes.

## Contracts Added or Changed

- `search::Pattern` compiles a bounded exact literal and returns a qualified
  Span, rejecting invalid starts and over-limit queries without source mutation.
- Search focus routes C-s, Return and query editing through its owner; pasted
  command-looking control data remains query text. Held Return cannot accept
  search twice or insert a newline after focus returns to editing.
- Search presentation validates source freshness before using a result. Raw
  source ranges remain exact; only cursor/highlight projection expands graphemes.
- The command catalog now exposes `isearch-forward`. The reference documents
  query anchoring, failure, bounds, matching and presentation; the changelog
  records the usable forward-search behavior.

## Files of Interest

- `src/search.rs`: bounded literal compilation and borrowed chunk scan.
- `src/input/search.rs`, `src/input/prompt.rs`: focused query editing and paste.
- `src/session/search.rs`: source-qualified transient search state and reveal.
- `src/display/search.rs`, `src/display/selection.rs`: independent match styling.
- `tests/search.rs`, `tests/search/edges.rs`: eleven native behavior cases.
- `scripts/check-pty.py`: forward continuation and multiline split-pane journeys.
- `docs/reference/search.md`: durable forward-search contract.
- `docs/reference/commands.md`, `docs/reference/index.md`, `CHANGELOG.md`:
  command discovery, reference navigation and reader-facing delivery note.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 302 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test search`: passed; eleven tests, zero skips. Coverage includes
  exact Unicode and CRLF seams, raw/grapheme range distinctions, no-match
  recovery, query bounds/ownership, source freshness, redo and large ropes.
- `python3 scripts/check-pty.py --suite terminal`: passed; sixty cases, zero
  skips. New journeys verify real C-s and M-x entry, next-match reveal across
  many lines, no-match deletion recovery, multiline Unicode query paste,
  pane isolation, tiny/zero geometry, cancel and exact unchanged disk bytes.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
