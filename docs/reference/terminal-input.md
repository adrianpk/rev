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

## Ordered repeat and visible progress

Vertical arrows use the ordinary logical-line and reveal owners. Native input
work yields to presentation after at most 32 events or 4 ms while retaining
handler order, reversal, text-command ordering, Escape and C-g. Release reports
do not repeat movement. The release collector checks first update and stop
tail within 150 ms at its declared input rate, then requires no drift or idle
output. See [bindings](bindings.md) for column and viewport behavior.

## Pointer reporting ownership

Native startup enables button-event tracking (DEC 1002) and SGR mouse
coordinates (DEC 1006) together when `mouse.enabled` is true (the default). It requests no all-motion hover or RXVT
reporting. The existing decoder supplies primary press/drag/release and vertical
wheel reports as zero-based terminal cells; these are not document positions.
Focused prompts retain their input. Wheel and unsupported reports preserve
keyboard prefixes and the existing Escape deadline. An admitted primary press
completes its focus transition and clears pending keyboard prefixes.

Primary presses resolve against the current composed tree and editing panes.
An editing text hit selects its live view and places point through checked
movement. Tab, wide, combining, control and emoji cells select a complete source
grapheme; scroll offsets and either gutter side use the rendering geometry.
The exact end-of-line boundary is selectable, including an empty final row.
Gutters, status rows, trailing fill, beyond-EOF rows and clipped glyph fragments
may focus their owning pane without moving point. Shared separators do not
select a pane. Tree entry hits select and activate through the native tree-open
owner by default; `mouse.tree_activate_on_click = false` selects/focuses only.
Blank tree rows and the footer focus without entry activation. Hits retain the
current viewport and full entry identity; stale rows cannot redirect an action.
Prompts, completion,
inventory and decisions retain ownership; clicks cannot accept or cancel them.
Movement preserves an existing active mark/region and creates no undo item.

A primary press arms its checked text point. Moving to a different represented
text boundary while held starts the ordinary linear mark/region. Release ends
capture and leaves that region available to copy, kill and other native editing
commands. A click or same-cell motion creates no new selection, and gestures
never copy implicitly to the kill ring or terminal clipboard.

Capture stays with its source pane across gutters, status, gaps and neighboring
panes. Non-text cells within the source's visible rows retain the last checked
endpoint; returning to source text continues from the same anchor. Above or
below its visible text, a captured drag scrolls only its originating view and
extends the canonical region. Each step waits at least 100 ms and advances at
most the lesser of `mouse.scroll_lines` and the source text-area height.
Returning inside pauses the deadline; release, cancellation, invalidation and
document bounds remove pending work. Late wakes perform one step without
catch-up or simulated momentum. Wheel reports do not transfer scrolling
during capture. Keyboard input, C-g, resize, reported focus loss, source/view
changes and host mode disable end capture. C-g keeps its normal region
cancellation; the mode-owner hook retains the canonical region. Orphan motion
and release cannot revive capture. There is no idle selection timer.

Vertical wheel and terminal-reported trackpad events scroll the document pane
or loaded file tree under the pointer by `mouse.scroll_lines` logical rows
(default three, integer 1..=100). They retain
keyboard focus, pending prefixes and other panes' points. A still-visible point
or tree selection remains unchanged; crossing the new viewport edge clamps it
to the nearest visible logical row. Source bytes and undo history are unchanged.
Tree scrolling uses its loaded entries without reading or expanding directories.
Native bounds include the final logical row, and suspended layouts have no
scroll target. Wheel handling schedules no inertia, timer or idle work.

`M-x mouse-mode` toggles native pointer support for the entire session, from
editing or tree focus. Disabling ends capture and pending drag work, retains the
native region, restores the acquired reporting pair and ignores late pointer
reports. Keyboard editing and other terminal owners stay active. Enabling
restores native gestures using the same owner. The startup snapshot is frozen:
neither toggle reloads or writes TOML, and a new process uses `mouse.enabled`
again. Set that Boolean to false to start without reporting.

Mouse and trackpad input depends on reports delivered by the terminal. Trackpads
work when the terminal translates them into vertical wheel/pointer events;
physical gesture or modifier support is terminal-specific. While reporting is
active, the terminal may reserve a bypass gesture (often holding Shift) for its
own text selection and copy behavior. Check the terminal's configuration; no
physical modifier is universal. `M-x mouse-mode` is the native disable path when
the terminal provides no usable bypass. Native selection copies only through
ordinary editor commands such as M-w, never implicitly to a system clipboard.

Disable, normal/error cleanup and unwind attempt every acquired reset. Failed
resets remain owned for retry. A later enable clears uncertain modes before
acquiring a confirmed pair. Only a valid captured outside drag owns a deadline;
ordinary idle, clicks, inside drags, disabled mode and released/cancelled gestures
add no polling, repaint timer or motion queue.

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

The release interaction collector exercises pointer reports at 120 reports per
second. An instantaneous multi-kilobyte report burst can stall the pinned
terminal decoder before subsequent keyboard input. The native event-work bound
does not establish a bound for that dependency's byte buffer. The measured
rate and idle results do not claim instantaneous burst support.
