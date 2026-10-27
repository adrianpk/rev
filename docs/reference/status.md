# Pane status

Each editing pane reserves one status row above its separator or the bottom
prompt/echo area. Its left group shows buffer identity, clean/modified state,
write access and logical position. The language mode aligns with the pane's
right edge, separated by flexible space:

```text
> config.rs -- RW -- L37 C3                                      Rust
```

`>` identifies the active editing pane; inactive panes retain their own position
and use the inactive status background. `--` immediately after the name means
clean and `**` means modified. `RW` means writable and `RO` means read-only.
The second `--` separates access from position. The right label is the current
syntax mode or `Text` when the buffer uses plain text.

Line and column are one-based logical document coordinates. Columns count
display cells with the buffer's tab stops and Unicode glyph widths; horizontal
scrolling and relative line numbers do not change them. Shared-buffer panes
share state and access but retain independent points. The status spans its
original pane width regardless of gutter side or width.

When the row cannot contain both groups, it omits the language first. It then
compacts position to `L37:3` and clips the name using terminal-cell boundaries.
At smaller widths it omits position and keeps complete state/access indicators
as space permits. Partial access tokens or truncated numeric positions are not
shown. Unusable geometry suspends presentation; complete groups return when
space becomes available. Prompts and tree rows keep their own data and bounds.

The existing `palette.status`, `palette.status_inactive` and
`palette.status_text` roles control status colors. Rendering observes the live
buffer and view without changing text, focus, history or saved bytes.
