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

## Bracketed paste

The terminal owner enables bracketed paste on entry and disables it during every
restoration path. A complete Crossterm paste event bypasses key decoding and
clears pending chord/Meta prefixes. Pasted C-x, C-g, Escape and similar bytes are
data and cannot invoke editing commands or confirm a decision.

In editing focus, valid Unicode is inserted at point through one checked edit
and one undo step, separate from adjacent typing. LF, CRLF, tabs and final-newline
absence are preserved as supplied, even when the buffer normally uses another
newline style. Controls are displayed using visible notation. NUL-containing
paste is refused as binary input without inserting any part of the payload.

An argument prompt accepts only single-line text within its existing 4096-character
limit, at the prompt cursor. Control-containing or oversized payloads reject wholly.
Decision and inventory focus refuse paste rather than treating it as a response.
Terminal paste framing and UTF-8 decoding belong to Crossterm; literal-event
fidelity applies to the Unicode string it delivers.

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

## Palette and cursor restoration

The renderer supplies semantic RGB foreground/background colors through SGR.
Selections and current search matches override syntax colors; each row resets
colors and attributes before another surface is drawn. `NO_COLOR` suppresses
RGB emission and retains reverse-video selection and search underlining.
True-color fidelity depends on terminal support.

The terminal owner attempts [xterm OSC 12 and OSC 112](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
for cursor color and reset when colors are enabled and `TERM` starts with
`xterm`, `screen`, `tmux`, `rxvt`, `alacritty`, `kitty`, `foot`, `wezterm` or
`ghostty`. Unknown/missing `TERM` and `NO_COLOR` leave cursor color untouched.
A recognized terminal or multiplexer can still ignore or block color controls;
the hardware cursor remains visible using its terminal color. No capability
query, reply wait or blinking timer is introduced.

Restoration resets text attributes/colors and the cursor's terminal default
color before leaving the alternate screen, including output errors and unwind.
This does not reconstruct a cursor color set by another application before
entry. Failed restorations remain owned for Drop to retry; every other cleanup
step, including raw-mode release, is attempted even after an output failure.

## Configured chord identity

The [keymap vocabulary](bindings.md#sequence-vocabulary) uses the same key
normalizer as native input. It deliberately merges traditional Control aliases
and Escape/Meta equivalents, including `C-i`/Tab, `C-m`/Return, `C-h`/Backspace,
NUL/`C-SPC` and the legacy undo reports. Conflicting custom aliases reject before
activation. Enhanced reports do not create separate identities for those aliases.
Named modified keys remain dependent on terminal reports and interception.


Editor, minibuffer and tree maps supplied with an admitted snapshot resolve
aliases and bounded prefixes at the current focus owner. Repetition uses the
resolved action, so a new single-chord motion/navigation alias can repeat while
a completed prefix cannot repeat its suffix. Prompt acceptance and tree mutation
or workspace transitions require a fresh press regardless of their alias.
Decisions keep explicit fresh answers; mapped acceptance cannot grant consent. Phase checks use normalized keys: held Control-m/j cannot bypass the
fresh Enter requirement in prompts. Releases and literal paste retain their
existing event ownership.
