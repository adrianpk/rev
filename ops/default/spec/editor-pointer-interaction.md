# Pointer Interaction

Status: accepted
Approved: product owner, 2026-10-27; finalize the selected seven-topic scope and start one combined Looper cycle.
Date: 2026-10-27
Format: OpenSpec capability additions
Baseline: [rev Editor](rev-editor.md), R03, R07, R09, R14-R16, R18 and R27
Panel contract: [Editor Interaction](editor-interaction.md), UI01
Gutter: [Editor Gutter](editor-gutter.md)
Controls: [Editor Configuration](editor-configuration.md)
Ticket: [Mouse, scrolling and pane focus](../ticket/open/20261027190629-mouse-scroll-selection-and-pane-focus.md)
Delivery boundary: selected combined input/navigation/presentation delivery; no implementation started

## Purpose

Support ordinary terminal mouse and trackpad events for scrolling, pane focus,
point placement and linear text selection. Keep these interactions native in
the Rust binary and consistent with existing keyboard commands and regions.

## Selected direction

- Scroll the pane beneath the pointer without changing keyboard focus.
- Change pane focus with a primary-button click.
- Interpret text hits through current panel, gutter and viewport geometry.
- Use the ordinary buffer region for text selection; rendered cells are not
  a separate text store or clipboard source.
- Autoscroll the originating view when an active text drag moves above or below
  its visible text area. Stop on release/cancellation and pause on re-entry.
- Keep keyboard paths available and handle only events the terminal supplies.

The product owner accepted conventional scroll targeting after comparison
with [Emacs wheel behavior](https://github.com/emacs-mirror/emacs/blob/master/lisp/mwheel.el).
This reference selects interaction behavior, not implementation code.

## ADDED Requirements

### Requirement: PTR01 Native terminal input and existing owners

Admitted mouse-wheel, trackpad-scroll, primary-button press, drag and release
events SHALL enter the existing native event/command path. Pointer handling
SHALL reuse the existing focus, view, region and history owners. Physical device
drivers, a scripting layer or editor addons SHALL NOT be required.

Vertical scrolling, pane focus and linear selection SHALL remain usable through
the supported terminal event protocol. Support SHALL NOT imply that a terminal
exposes raw touch gestures, pixel precision or every hardware-specific action.

#### Scenario: Use a terminal-reported trackpad scroll

- **GIVEN** a terminal that reports trackpad movement as scroll events
- **WHEN** the editor receives an admitted vertical scroll over document text
- **THEN** it uses the same native scrolling path as an equivalent wheel event
- **AND** no device-specific runtime component is started.

### Requirement: PTR02 Scroll the pointed pane without taking focus

Vertical wheel/trackpad input SHALL scroll the document view or file tree under
the pointer. It SHALL preserve keyboard focus, including when the target is an
inactive pane. Primary-button clicks SHALL use the ordinary checked pane-focus
transition. Hover alone SHALL NOT select a pane.

Only the targeted view's viewport and its necessary view-local point adjustment
may change during scrolling. Shared text, unrelated views and the active pane's
editing state SHALL remain coherent. Geometry edges and document bounds SHALL
clamp safely. The editor SHALL NOT add simulated momentum or an idle scroll loop.

#### Scenario: Scroll an inactive tree while editing text

- **GIVEN** a document pane has keyboard focus and a visible inactive file tree
- **WHEN** wheel input arrives with coordinates inside the tree
- **THEN** only the tree's view scrolls and keyboard focus remains in the document
- **AND** a later click in the tree performs the ordinary focus transition.

### Requirement: PTR03 Checked point placement through composed geometry

A primary-button text click SHALL resolve the owning live view, focus it and
place point at the represented logical text boundary. Mapping SHALL account for
scroll offsets, tabs, Unicode graphemes, wide glyphs, clipping and the gutter's
selected side and current width. No hit may split an encoded character or use
stale bounds after resize or layout changes.

Gutter labels, borders, status rows, padding and other panels SHALL NOT become
buffer text or invented text positions. A pane-owned non-text hit may focus its
pane without moving point. Separators SHALL NOT initiate an unrequested edit or
resize action. Active prompt/decision owners SHALL retain their existing input
and cancellation guarantees.

#### Scenario: Click tabbed Unicode text beside a gutter

- **GIVEN** nested document panes with relative line numbers and tabbed Unicode text
- **WHEN** a text hit is resolved after a resize and horizontal viewport change
- **THEN** it selects the current owning view and a valid represented text boundary
- **AND** the line-number field and neighboring pane cannot become the destination.

### Requirement: PTR04 Canonical linear selection and finite drag ownership

A primary-button text drag SHALL establish mark and point through the ordinary
linear-region owner. Release SHALL end pointer capture and retain the resulting
region for ordinary copy, kill and other editing commands. A press/release
without a drag SHALL remain point placement rather than an empty selection.

Selection SHALL remain owned by the originating document view. Crossing another
pane, gutter or decoration SHALL NOT select its rendered content or transfer
the drag to that surface. Cancellation, view loss, resize invalidation and mode
disable SHALL end capture without editing text or retaining a stale target.

During an active captured text drag, moving above or below its visible text
area SHALL autoscroll that originating view and extend its canonical region.
Returning inside SHALL pause autoscroll. Release, cancellation, invalidation
or loss of capture SHALL stop it, with no queued or simulated momentum.
Crossing another pane SHALL NOT transfer focus, selection or scrolling to it.

Native editor selection SHALL remain distinct from terminal-owned selection.
Documentation SHALL explain the terminal's bypass gesture where available and
the editor's mouse-disable path, without promising a universal physical modifier.

#### Scenario: Select text across a neighboring pane boundary

- **GIVEN** a drag starts on ordinary document text beside a gutter and another pane
- **WHEN** the pointer crosses those UI areas and the primary button is released
- **THEN** only a valid source-buffer range can remain selected
- **AND** no UI label, other buffer content or history edit is captured.

#### Scenario: Extend selection beyond the visible text

- **GIVEN** a live text drag and more source text beyond the visible rows
- **WHEN** the pointer moves past the top or bottom of its text area while held
- **THEN** bounded autoscroll extends the source buffer's canonical region
- **AND** release stops scrolling while the resulting region remains active.

### Requirement: PTR05 Reversible terminal modes and bounded input work

Only the terminal reporting modes required by the admitted interactions SHALL
be enabled. Disabling support and every normal/error terminal cleanup path
SHALL restore the relevant reporting state and end capture. Mouse support
SHALL add no idle polling, idle repaint timer or unbounded motion queue.
Bounded autoscroll ticks MAY run only while a valid out-of-bounds drag is
captured. Every tick SHALL recheck ownership and document bounds; release,
cancellation or invalidation SHALL remove pending autoscroll work. Autoscroll ticks SHALL be no more frequent than once per 100 ms and each
step SHALL advance at most the lesser of mouse.scroll_lines and the current
text-area height. New motion SHALL NOT accumulate timer work; release/input
cancellation is processed before any due tick.

Scroll/drag work SHALL remain bounded by the current projection and selected
operation. The arrow-inertia ticket's input/rendering investigation SHALL inform
shared regression coverage without treating an unverified backlog as a diagnosis.

#### Scenario: Disable mouse support during a drag

- **GIVEN** a live pointer selection in a normal editing session
- **WHEN** native mouse support is disabled or terminal cleanup runs
- **THEN** reporting/capture ends through the owned cleanup path
- **AND** keyboard operation and document bytes remain valid.

### Requirement: PTR06 Complete startup defaults and native runtime control

Pointer support SHALL have usable factory defaults, typed TOML preferences and
a compiled M-x runtime toggle. Settings SHALL use the existing schema and
startup-only snapshot; runtime overrides SHALL NOT rewrite TOML. Supported
fields SHALL enter the exhaustive sample and packaged bootstrap template.

#### Scenario: Change pointer mode for the current session

- **GIVEN** pointer support was selected by startup settings
- **WHEN** the user invokes its native M-x toggle
- **THEN** the current session changes reporting mode through the same owner
- **AND** saved TOML remains unchanged for the next startup.

## Factory controls

The following concrete native defaults apply to this delivery:

| Concern | Proposal |
| --- | --- |
| Startup support | `mouse.enabled = true` |
| Vertical step | `mouse.scroll_lines = 3`, integer 1..100 logical rows per admitted event |
| Runtime mode | `M-x mouse-mode` toggles the session override |
| Focus | Primary-button click; wheel and hover retain focus |
| Text selection | Primary-button drag; no implicit clipboard write |

## Drag-edge behavior selected from existing editors

[Emacs mouse editing](https://www.gnu.org/software/emacs/manual/html_node/emacs/Mouse-Commands.html)
documents steady autoscroll when a drag moves above or below the window and a
pause when the pointer returns. [Neovim mouse interaction](https://neovim.io/doc/user/gui/)
also supports scrolling while extending selection near window edges, with
`scrolloff` affecting that boundary behavior. Its native
[mouse handler](https://github.com/neovim/neovim/blob/master/src/nvim/mouse.c)
also handles dragging outside the current window.

Following the product owner's instruction to use the conventional behavior,
rev selects bounded drag autoscroll with the existing source-view ownership.
The finite tick and per-step limits above are native implementation bounds.
This does not add new drag gestures.

## Validation obligations

Cover focus/scroll targeting across a tree and multiple shared/different-buffer
panes; empty and large files; absolute/relative and left/right gutters; Unicode,
tabs and horizontal offsets; tiny and resized geometry; cancelled/stale drags;
startup/runtime controls and terminal cleanup. Use native integration checks
and real PTY protocol journeys. Verify responsiveness and idle CPU/memory work
without treating fixture schemas alone as runtime evidence.

## Delivery state

Scroll/focus targeting, drag-edge autoscroll and the native controls above
are selected for the combined delivery. Acceptance of the contract does not
claim implementation or completed runtime validation.
