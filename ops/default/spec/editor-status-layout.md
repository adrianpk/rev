# Pane Status Layout

Status: accepted
Approved: product owner, 2026-10-27; finalize the selected seven-topic scope and start one combined Looper cycle.
Date: 2026-10-27
Format: OpenSpec change
Baseline: [rev Editor](rev-editor.md), R05 and R07
Panel owner: [Editor Interaction](editor-interaction.md), UI01
Related geometry: [Editor Gutter](editor-gutter.md)
Ticket: [Group pane status left and right](../ticket/reviewing/20261027194421-group-pane-status-left-and-right.md)
Delivery boundary: selected combined input/navigation/presentation delivery

## Purpose

Make the existing status facts readable through two aligned groups, preserving
a compact native presentation. Position remains beside the resource identity
on the left; the existing language/syntax context sits on the right.

## Selected presentation

```text
config.rs -- RW -- L37 C3                                      Rust
config.rs ** RW -- L37 C3                                      Rust
```

The first two-character marker is the existing clean/modified state, not an
arbitrary separator. RO/RW retain their current meaning. The separator before
position is presentation. Preserve the current documented position convention
and focus indicator; relative gutter numbering does not change logical line
numbers in status. This change adds no status fields.

## MODIFIED Requirements

### Requirement: R05 Screen layout and status

Source: sections 5.1-5.2. Acceptance: A01, A05, A20.

The frame MUST contain editing windows, a per-window mode/status indication
or a clearly equivalent indication, a bottom minibuffer/echo area, and an
optional left file tree. Initially the tree is hidden and no prompt is active.
The selected editing window MUST be visually identifiable.

Status MUST show buffer/file name, dirty state, current line and column,
syntax mode or `Text`, and read-only state when applicable. Errors MUST appear
without erasing document content. The column convention MUST be documented
and remain consistent with the explicit coordinate domains.

Status SHALL use two aligned groups within each editing pane's own status
rectangle. The left group SHALL contain the existing buffer name, clean/modified
marker, write policy and line/column. The right group SHALL contain its syntax
mode or Text label. Flexible space SHALL separate the groups when they fit.
The existing active-pane indication and palette roles SHALL remain available.
The single-row reservation and separate prompt/echo owner SHALL remain unchanged.
When width cannot admit the complete status, STA02 defines the explicit
compact/omitted-field exception; no overlapping or false label is permitted.

#### Scenario: Save error remains visible with document intact

- **GIVEN** an active buffer and its status indication
- **WHEN** an ordinary save error is reported
- **THEN** the echo area identifies the failure
- **AND** the document and its dirty indication remain visible.

#### Scenario: Show ordinary status in two aligned groups

- **GIVEN** an editing pane with enough status width and a Rust buffer at line 37, column 3
- **WHEN** its status is rendered
- **THEN** the left group shows its identity, state and L37 C3 position
- **AND** Rust is right-aligned within that pane, separated by flexible space.


## ADDED Requirements

### Requirement: STA01 Preserve pane-local observations

Each editing pane SHALL show the buffer and position belonging to that view.
Shared buffer dirty/write policy remains shared, while view-local point and
active styling remain independent. Rendering SHALL NOT switch focus, mutate
text, query external processes or make status text canonical state.

The change SHALL reuse the existing one-row status and palette owners. A
programmable segment registry, extra status rows and new status subsystems are
not required. Tree and transient prompt/status surfaces retain their own data
and bounds rather than inheriting an unrelated document's status.

#### Scenario: Shared buffer, independent positions

- **GIVEN** two editing panes show one buffer at different stored points
- **WHEN** focus changes or one point moves
- **THEN** each status reports its own current line and column
- **AND** shared dirty/write state remains truthful in both panes.

### Requirement: STA02 Deterministic narrow-width composition

Composition SHALL measure terminal cells through the existing checked text
projection, including Unicode and wide glyphs. Both groups SHALL remain inside
the owning pane's status rectangle and SHALL NOT overlap or cross a separator.
The gutter's side or width SHALL NOT become an independent status offset.

When width is insufficient, remove the language group first, compact position
next and clip the buffer label as required while prioritizing state/access
indicators. Tiny geometry SHALL retain bounded output rather than fabricate a
state or write outside the row. Omitted groups SHALL NOT leave orphan separators.
The complete form SHALL return when width is available again.

#### Scenario: Shrink and restore a split pane

- **GIVEN** two status groups and a wide Unicode buffer label in nested panes
- **WHEN** the terminal shrinks and later grows
- **THEN** composition degrades within each pane without overlap or invalid text
- **AND** the complete truthful two-group presentation returns after growth.

## Validation obligations

Cover clean/modified and RW/RO states, Text and syntax modes, shared/different
buffers, independent points, nested splits, left/right gutters, Unicode names,
tiny geometry and restoration. Verify active/inactive palette roles and a
representative terminal journey with prompt/completion surfaces. Status work
must remain bounded and event-driven. No runtime validation is claimed here.

## Review state

The selected two-group presentation and compact-width policy belong to the
combined delivery. This behavior-only change introduces no new settings schema
or persisted model. No implementation result is claimed.
