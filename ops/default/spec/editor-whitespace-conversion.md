# Explicit Whitespace Conversion

Status: accepted
Approved: product owner, 2026-10-27; finalize the selected seven-topic delivery using the existing editor and Emacs interaction conventions.
Date: 2026-10-27
Format: OpenSpec capability additions
Baseline: [rev Editor](rev-editor.md), R09, R10, R18, R21 and R22
Configuration: [Editor Configuration](editor-configuration.md), CFG10 and CFG11
Ticket: [Convert tabs and spaces](../ticket/solved/20261027092143-convert-tabs-and-spaces.md)

## Purpose

Expose explicit tabify/untabify commands while keeping display policy, Tab-key
insertion and changes to existing document whitespace separate operations.

## ADDED Requirements

### Requirement: WSP01 Region-scoped conversion with preserved visual columns

M-x tabify and M-x untabify SHALL operate on the marked linear region admitted
by the editor's existing region owner. With no usable region they SHALL report
that condition rather than implicitly convert the whole buffer. An empty region
SHALL be a no-op. Whole-document conversion requires explicitly marking it.

Untabify SHALL expand selected literal tabs to spaces at the buffer's effective
tab stops. Tabify SHALL convert eligible whitespace runs of at least two cells
to a shorter tab/space representation only when all non-whitespace display
columns remain unchanged. Eligibility applies throughout the selected region,
not only indentation; language strings/comments do not receive implicit parser
exceptions. Existing tab stops, Unicode display widths and text before a
mid-line selection SHALL determine the actual starting column.

Only selected whitespace may change. Unicode text, line-ending bytes, BOM and
final-newline state SHALL remain intact. The commands SHALL NOT change settings
or convert other buffers. No factory shortcut is required; existing validated
keymaps may refer to the native command names.

#### Scenario: Expand a selected tab after Unicode text

- **GIVEN** a region begins mid-line after wide/grapheme text and contains tabs
- **WHEN** untabify runs under the buffer's effective tab width
- **THEN** tabs become the correct number of spaces for their actual columns
- **AND** text outside the region and non-whitespace display columns are unchanged.

#### Scenario: Leave an unselected document untouched

- **GIVEN** a document with convertible whitespace and no usable marked region
- **WHEN** the user invokes either conversion command
- **THEN** it reports the missing operation range without a document-wide fallback
- **AND** document bytes, dirty state and history remain unchanged.

### Requirement: WSP02 One coherent edit with ordinary safety and history

A successful conversion SHALL publish one canonical transaction with checked
revision/ranges, ordinary shared-view updates and one undo operation. Redo SHALL
restore the converted result. A no-op SHALL create no edit/history/dirty change.
Read-only admission and failure/cancellation SHALL retain all prior text.

#### Scenario: Undo one multi-line conversion

- **GIVEN** a selected multi-line region with mixed tabs and spaces
- **WHEN** tabify changes it and the user invokes one undo
- **THEN** the exact original bytes and coherent shared-view positions return
- **AND** redo restores the converted bytes through the normal history owner.

## Reference and validation

[GNU Emacs tabs and spaces](https://www.gnu.org/software/emacs/manual/html_node/emacs/Just-Spaces.html)
defines region-based tabify/untabify and preserved text columns. Rev retains its
existing admitted-region policy. Verify both directions, mid-line boundaries,
different buffer tab widths, mixed whitespace, Unicode, CRLF, no-op/read-only
cases, dirty-state identity, undo/redo and actual M-x dispatch.
