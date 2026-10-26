# Forward search

`C-s` or `M-x isearch-forward` opens `I-search:` in the minibuffer. Type a
literal query to find its first occurrence at or after the starting point.
Matching is case-sensitive and does not wrap at the end of the buffer.

- Query edits update the result immediately. Prompt cursor movement, Home/End,
  Backspace/Delete and their Emacs equivalents edit only the query. Tab inserts
  a literal tab; bracketed paste can supply multiline Unicode or CRLF text.
- C-s continues after the current match. Further query edits search from that
  continuation anchor. An empty query returns to the starting point.
- A match moves point to its end and reveals it in the selected pane. Underline
  identifies the match independently of the region's reverse video. Source
  matches may include part of a grapheme; cursor and highlighting cover whole
  graphemes, including CRLF pairs. Status, other panes and echo remain unstyled.
- `No match` appears beside the retained query when no result exists. Point
  stays at its last valid position, and removing query text can recover a match.
- Return accepts the current position and closes search. C-g cancels the query
  and returns to the starting view. Neither action inserts query text.

Queries are limited to 4096 Unicode scalars. A rejected insertion preserves the
existing query. Matching scans borrowed rope chunks and carries literal prefix
state between them; it copies only the bounded query and its fallback table.
It does not create a whole-buffer copy on each query key.

Each result retains the original buffer identity, revision and half-open scalar
range. If the live source changes, the stale result cannot be highlighted or
restored; search closes with `Search source changed`. Search itself changes
view and transient input state, leaving text, modification state and undo/redo
transactions intact.
