# Terminal input

The input owner receives Crossterm events. Linux controlling-PTY checks exercise
the byte reports below; native event checks cover explicit modifier and key-phase
reports. A physical chord works when the terminal and any multiplexer deliver a
distinguishable supported report.

## Meta and Escape

- Reported Alt or Meta modifiers use the same Meta bindings. Shift on a reported
  character preserves that character, including `<` and `>`.
- A separately reported plain Escape starts a **100 ms** Meta deadline. A plain
  chord received before that deadline gains Alt and follows the active keymap.
  An existing `C-x` or `C-x r` prefix remains the owner of its suffix.
- Reported Control chords retain their meaning while Escape is pending. In
  editing focus, `C-x C-c` can still request guarded exit; `C-g` immediately
  cancels the prefix or active interaction.
- At or after the deadline, standalone Escape clears pending key prefixes and
  displays `Escape`. It preserves the focused prompt or decision; use `C-g` to
  cancel that interaction. A later chord is handled normally and is not lost.
- A second separately reported Escape clears the pending prefix immediately.
  Escape press/release repeats do not extend its deadline.
- Input polling waits only for the remaining deadline. Resize events remain
  actionable, and terminal errors return through normal guard restoration.
  There is no timer thread or sleep-based input wait.
- Staging a Meta modifier preserves kill/yank continuity until its chord is
  resolved. Cancellation, expiry and an unbound chord end that continuity.

Escape bytes received together with another chord may already be decoded by
Crossterm as one Alt event. That event dispatches immediately. The application
deadline applies to separately reported Escape events; it does not reinterpret
partial control sequences buffered by the terminal library. Delivery grouping
cannot distinguish every physical Escape press from Alt.

## Required reports

| Chord | Supported event/report | Behavior |
| --- | --- | --- |
| `C-SPC` | Control-space, Control-`@`, or normalized NUL key | Set mark and activate the linear region. |
| `C-/` | Distinguishable Control-`/`, including a CSI-u report | Undo. |
| `C-_` | Distinguishable Control-`_` or legacy byte `0x1f` | Undo. |
| `C-x u` | Control-`x`, then plain `u` | Undo without a punctuation-modifier report. |
| `M-<`, `M->` | Alt/Meta character or compatible Escape prefix | Move to buffer start/end. |
| `C-x r` | Control-`x`, then plain `r` | Retain the nested rectangle prefix for its suffix or `C-g`. |

Crossterm's Unix legacy decoder reports byte `0x1f` as Control-`7`. The input
owner maps that report to undo because the byte cannot identify its physical
`C-/`, `C-_` or `C-7` origin. A terminal that intercepts a chord cannot supply it
to the editor; `C-x u` is the reportable alternative.

## Phases, focus and invalid input

Only presses and allowed repeats dispatch. Releases are inert. Prefix,
confirmation and completed command suffix repeats cannot dispatch twice or
become document text. Prompt, inventory and decision focus continues to own
modified input; Escape never transfers that input to the underlying document.

Invalid nested sequences clear their prefix and show a concise message. An
unsupported rectangle suffix is rejected without invoking a linear command.
Unknown library-decoded modified keys and control characters are rejected;
malformed terminal control sequences discarded by Crossterm do not enter text.
Printable characters received after a standalone Escape has expired are ordinary
new input, rather than an indefinitely pending Meta suffix.
