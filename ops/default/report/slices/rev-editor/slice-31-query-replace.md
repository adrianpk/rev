# Slice 31: Confirm literal replacements interactively

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-31-query-replace`
PR: `#31`

## Purpose

Deliver literal query replacement through focused arguments and per-occurrence
confirmation, with independent checked edits and meaningful undo of accepted text.

## Delivered Behavior

- M-% and M-x query-replace share one handler. The operation asks for a literal
  query and replacement, then reveals and underlines each candidate in the
  selected pane. Matching uses existing smart case from point to buffer end,
  without wrapping or changing another buffer.
- A fresh y replaces the current occurrence, n skips it and q stops. C-g stops
  either argument or a decision. Return has no affirmative default; pasted,
  held and released answers cannot grant consent or advance another occurrence.
- Only accepted occurrences publish text. Each accepted replacement uses one
  canonical checked transaction, with coherent shared-view/mark mapping and one
  independent undo/redo step. Skipped text remains intact when accepted edits
  are undone, and stopping retains earlier accepted changes.
- Query and replacement drafts are bounded at 4096 scalars each and support
  grapheme editing, literal Tab and bracketed multiline Unicode paste. Empty
  queries and NUL-containing replacements reject visibly. Empty replacements
  delete accepted text; control-looking replacement data remains literal.
- Continuation starts after inserted text or the skipped raw range, so inserted
  query-looking text is not processed again. No matches completes with zero
  replacements and unchanged point/history. Read-only buffers refuse entry.
- Arguments and decisions retain their selected view, buffer and revision.
  Source changes invalidate outstanding authority before an answer can edit
  newer text. Normal editing resumes after completion, cancellation or failure.

## Implementation Notes

- T31.1: `802e14981c2f83f1211786ca16cba39aadf06d45` adds command and
  binding, literal argument/decision contexts, qualified candidate ownership,
  independent transaction publication, three native cases and product reference.
- T31.2: `2231f92989703ee543c68036c7c64ac96fde60f5` expands query-replace
  coverage to eleven native tests and adds two real PTY journeys for consent,
  literal Unicode/line endings, shared-view history and exact save bytes.
- The pending owner has explicit query, replacement and candidate stages. It
  retains a compiled bounded Pattern, literal replacement and qualified Span.
  Decisions check source identity/revision before invoking the existing checked
  Session::replace path; publication refreshes the owner before the next scan.
- Continuation uses original scalar boundaries, not projected display columns
  or folded indices. Empty replacement resumes at the deletion boundary; all
  nonempty matches consume source text. Existing borrowed-rope matching remains
  in use, with no document flattening or new dependency.
- Native cases verify shorten/grow/delete/equal replacements, literal controls,
  smart-case expansions, combining marks, emoji, multiline CRLF, separate buffers,
  shared cursors/marks, every cancellation stage, stale source and argument bounds.
- PTY journeys use existing bounded complete-frame waits and resize handshakes.
  They prove actual M-%/M-x entry, rejection of pasted/default consent, skip/stop,
  independent undo/redo and unchanged skipped bytes in saved files. Only owned
  fixtures close after checks; normal exit verifies terminal restoration.

## Contracts Added or Changed

- The fixed command catalog exposes `query-replace`, bound to reported Meta-%
  and the compatible Escape-percent sequence through the shared input decoder.
- Literal replacement arguments have a focused context separate from path
  arguments. They admit multiline paste without interpreting command bytes;
  occurrence decisions retain fresh y/n/q admission with no implicit acceptance.
- Candidate highlights reuse validated search projection. Raw source ranges
  remain exact even when a match touches part of a grapheme; cursor/display
  repair does not enlarge the text selected for replacement.
- Source-qualified authority is checked on argument acceptance, decision and
  redraw. A stale question closes with `Query replace source changed` and cannot
  become a replacement against the new revision.
- `docs/reference/search.md` documents literal replacement and consent lifetimes.
  Command/prompt references and the index link it; `CHANGELOG.md` describes the
  completed reader-facing replacement workflow.

## Files of Interest

- `src/session/replace.rs`: staged ownership, candidate scan and checked edits.
- `src/input/replace.rs`, `src/input/paste.rs`: literal arguments and fresh decisions.
- `src/command.rs`, `src/input/bindings.rs`: shared named and bound entry.
- `src/session.rs`, `src/session/file.rs`, `src/session/save.rs`,
  `src/session/search.rs`: focused routing, presentation and invalidation.
- `tests/search/replace.rs`: eleven native query-replace behavior tests.
- `scripts/check-pty.py`: actual confirm/skip/stop and multiline file journeys.
- `docs/reference/search.md`, `docs/reference/commands.md`,
  `docs/reference/prompts.md`, `docs/reference/index.md`, `CHANGELOG.md`:
  product contract, command discovery and reader-facing delivery note.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 321 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test search`: passed; thirty tests, zero skips. Eleven
  query-replace tests cover consent, literal progress, lengths, Unicode/CRLF,
  view scope, cancellation, read-only entry, stale source and bounded arguments.
- `python3 scripts/check-pty.py --suite terminal`: passed; sixty-four cases,
  zero skips. New journeys verify real Meta-%/M-x prompts, fresh per-occurrence
  consent, skip/stop, literal replacement, independent undo/redo, resize and
  exact saved Unicode/BOM/CRLF bytes while skipped occurrences remain unchanged.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
