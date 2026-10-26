# Search and replacement

## Incremental search

`C-s` or `M-x isearch-forward` opens `I-search:` in the minibuffer. `C-r` or
`M-x isearch-backward` opens `I-search backward:`. Type a literal query to find
the first occurrence at or after point, or the last occurrence ending at or
before point for backward search. Search does not wrap at either buffer edge.

Queries without uppercase characters use Unicode case-insensitive comparison.
Any uppercase query character makes the entire query case-sensitive. Editing
the query recalculates that choice immediately. Case-insensitive search uses
[Unicode 17.0.0 full default case folding](https://www.unicode.org/Public/17.0.0/ucd/CaseFolding.txt),
without locale-specific Turkic mappings or normalization. For example,
`strasse` matches `Straße`, while `STRASSE` requires those exact capitals.
Matches consume complete scalar expansions: `ss` can match `ß`, while `s`
cannot match half of that expansion. Composed and decomposed accents stay distinct.

- Query edits update the result immediately. Prompt cursor movement, Home/End,
  Backspace/Delete and their Emacs equivalents edit only the query. Tab inserts
  a literal tab; bracketed paste can supply multiline Unicode or CRLF text.
- C-s continues after the current match; C-r continues before it. Switching
  direction finds the adjacent result in the newly chosen direction. Further
  query edits search from that continuation anchor. With no current match, a
  direction change uses the retained point. An empty query returns to the
  original starting point while retaining the chosen direction.
- A forward match moves point to its end; a backward match moves point to its
  start. Normal scrolling reveals that point in the selected pane. Underline
  identifies the match independently of the region's reverse video. Source
  matches may include part of a grapheme; cursor and highlighting cover whole
  graphemes, including CRLF pairs. Status, other panes and echo remain unstyled.
- `No match` appears beside the retained query when no result exists. Point
  stays at its last valid position, and removing query text can recover a match.
- Return accepts the current position and closes search. A fresh C-g cancels
  from any query, direction or failure state and restores the starting point,
  scrolling, intended column and recenter cycle in its original pane. Normal
  reveal then accommodates current terminal geometry. Neither action inserts
  query text, and a held Return cannot insert a second newline after acceptance.

Queries are limited to 4096 Unicode scalars. A rejected insertion preserves the
existing query. Matching scans borrowed rope chunks and carries literal prefix
state between them. Unicode folding expands each scalar to at most three
scalars; matching keeps a bounded query/fallback table and a query-sized ring
of original source positions. Backward search reverses its bounded query and
traverses borrowed source characters backward across chunks. Neither direction
creates a whole-buffer or folded-file copy on each query key.

Each result retains the original buffer identity, revision and half-open scalar
range. If the live source changes, the stale result cannot be highlighted or
restored; search closes with `Search source changed`. Search itself changes
view and transient input state, leaving text, modification state and undo/redo
transactions intact.

## Query replace

`M-%` or `M-x query-replace` asks for `Query replace:` and then `Replace with:`.
Return accepts each argument. The query must be nonempty; an empty replacement
deletes accepted text. Both arguments support grapheme editing, literal Tab and
bracketed multiline Unicode paste, bounded at 4096 scalars each. A replacement
containing NUL is rejected. Query input and replacement drafts do not edit text.

Matching uses the same literal smart-case policy as incremental search, from
the starting point to the active buffer's end, without wrapping. The selected
pane reveals and underlines the current original source range. Replacement text
is inserted exactly, with no case adjustment, backreferences or expression
interpretation. A raw match may touch part of a grapheme; only its exact scalar
range is replaced, while cursor and underline projection retain whole graphemes.

At `Replace occurrence? [y/n/q]`, a fresh y replaces the displayed occurrence,
n skips it and q stops. C-g stops at either argument or a decision. Return and
other keys have no affirmative default. Paste and held/released answers cannot
grant consent. Accepted replacements remain when the operation stops; undo
reverses each accepted occurrence independently, leaving skipped text intact.

Continuation starts after the inserted text or skipped source range, so inserted
query-looking text is not processed again. Empty replacements continue at the
deletion boundary. Completion reports the accepted count; no matches reports
zero without changing text, point or history. Read-only buffers refuse entry.

The operation retains its selected view, buffer, revision and current source
range. A later source change invalidates its outstanding arguments/decision with
`Query replace source changed`; a stale answer cannot edit another occurrence.
Each accepted change uses the normal checked transaction, mapping shared views
and marks and preserving literal LF/CRLF bytes. Normal editing remains available
after completion, failure or cancellation, with no deferred replacement.
