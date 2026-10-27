# Line Navigation

Status: accepted
Approved: product owner, 2026-10-27; finalize the selected seven-topic delivery and start one Looper cycle using the requested Emacs behavior.
Date: 2026-10-27
Format: OpenSpec capability additions
Baseline: [rev Editor](rev-editor.md), R07, R09, R16, R18 and R22
Related: [Editor Gutter](editor-gutter.md)
Ticket: [Emacs-style goto-line](../ticket/open/20261027191322-add-emacs-style-goto-line.md)

## Purpose

Provide the requested goto-line behavior through the native command and prompt
owners. Preserve C-g cancellation and existing text, view and history contracts.

## ADDED Requirements

### Requirement: NAV01 Familiar command and checked destination

M-g g, M-g M-g and M-x goto-line SHALL open the same native line-number prompt.
M-g alone SHALL remain a prefix; C-g SHALL retain cancellation in every context.
The prompt SHALL accept a bounded signed decimal integer. Malformed or
overflowing input SHALL produce recoverable feedback without changing point.
This capability does not introduce a general numeric-prefix command language.

For an existing positive logical line, point SHALL move to its beginning. Line
numbers SHALL be one-based and independent of relative gutter labels. Zero or
negative input SHALL clamp to buffer start; a request beyond the last logical
line SHALL clamp to buffer end, matching the observed Emacs goto-line behavior.
An empty buffer SHALL remain at its sole valid position.

#### Scenario: Visit an admitted logical line

- **GIVEN** a multiline buffer and relative gutter numbering
- **WHEN** the user submits line 37 through either binding or M-x
- **THEN** the selected view reaches the beginning of logical line 37
- **AND** the gutter's relative labels do not change the requested coordinate.

#### Scenario: Clamp an out-of-range request

- **GIVEN** a two-line buffer whose last line has no terminating newline
- **WHEN** the user submits a line after the document's last line
- **THEN** point reaches the exact buffer end, not an invented line
- **AND** zero or negative requests instead reach buffer start.

### Requirement: NAV02 Cancel safely and preserve ordinary view ownership

No point movement SHALL occur while entering the number. C-g SHALL cancel and
retain the original point, viewport and document state. Successful navigation
SHALL use the ordinary checked movement/region rules and reveal the destination
in the selected view. Other views SHALL retain their independent points.

Navigation SHALL work in read-only buffers and SHALL NOT create a text edit,
undo node, save, configuration write or dirty-state change. Shared buffer
selection follows the existing mark/region contract rather than a second owner.

#### Scenario: Cancel a line prompt with unsaved text

- **GIVEN** an unsaved Unicode buffer and a pending line-number prompt
- **WHEN** the user types a number and cancels with C-g
- **THEN** its original point, viewport, text and history remain intact
- **AND** normal editing focus returns without a navigation side effect.

## Reference and validation

[GNU Emacs movement](https://www.gnu.org/software/emacs/manual/html_node/emacs/Moving-Point.html)
defines the bindings. Boundary behavior was checked with the installed Emacs
using temporary buffers, including zero, negative and beyond-EOF requests.
Verify both key sequences, M-x dispatch, invalid input, CRLF, final empty lines,
Unicode, cancellation, independent views and real terminal interaction.
