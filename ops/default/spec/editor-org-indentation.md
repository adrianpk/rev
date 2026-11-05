# Org Visual Indentation

Status: Completed
Completed: 2026-11-05
Date: 2026-11-05
Format: OpenSpec capability specification
Source: owner request for indented or unindented Org presentation, startup TOML and runtime commands, in no more than six slices
Approved: product owner, 2026-11-05; specification approved and explicit plan, slicing and one Looper startup requested
Model: [Org Visual Indentation Model](editor-org-indentation-model.md)
Plan: [Org Indentation Delivery](../plan/rev-org-indentation.md)
Tracker: [Org Indentation Tracker](../tracker/rev-org-indentation.md)
Related owners: [Heading Presentation](editor-org-heading-presentation.md), [Soft Wrapping](editor-soft-wrap.md), [Org Mode](editor-org-mode.md), [Configuration](editor-configuration.md)

## Purpose

Align Org body text with its enclosing heading through visual indentation.
The owner requested two choices, indentation enabled or disabled, available in
TOML and through native commands. This concerns heading hierarchy and its body,
not changing list-item nesting or inserting spaces into files.

The behavioral reference is [Org Indent mode](https://orgmode.org/manual/Org-Indent-Mode.html).
The approved rev profile uses two cells per heading level, defaults to enabled
and keeps the runtime choice local to a buffer, like Visual Line mode.
Approval defines the delivery contract; it is not implementation evidence.

## Presentation examples

For the same source, using the existing default heading markers:

```org
* Project
Project notes.
** Tasks
TODO Review the route.
*** Details
Check the distance.
```

Enabled, with heading text and body aligned:

```text
◉ Project
  Project notes.
  ○ Tasks
    TODO Review the route.
    ✸ Details
      Check the distance.
```

Disabled, retaining the current heading presentation:

```text
◉ Project
Project notes.
 ○ Tasks
TODO Review the route.
  ✸ Details
Check the distance.
```

Disabled removes the new virtual indentation. It does not remove original star
cells or rewrite the existing pretty-heading contract into a flat outline.
Glyph selection and pretty/source presentation remain independent.

## Requirements

### Requirement: OIN01 Indent heading bodies without editing text

When enabled, rev SHALL indent recognized headings and their body lines by
structural level. For level N, add N-1 virtual cells before the original heading
prefix and 2N before body lines, subject to the narrow-pane rule in the model.
With pretty headings and hidden leading stars, the marker is at column 2(N-1)
and the title and unindented body text begin at column 2N. Existing source
whitespace SHALL remain additional source whitespace. Content before the first
heading SHALL receive no structural margin.

Native Org recognition SHALL own enclosing-heading identity through the next
heading, including a return to a shallower level. Tables, lists, drawers and
source/example blocks SHALL retain their internal source layout as a unit;
literal stars inside a block SHALL not create headings. Blank lines SHALL not
acquire source spaces. Disabling the mode SHALL restore existing geometry.

#### Scenario: Follow the enclosing heading

- **GIVEN** the three-level source in the examples and indentation enabled
- **WHEN** it is displayed in a sufficiently wide Org pane
- **THEN** body lines begin at columns 2, 4 and 6 respectively
- **AND** a subsequent level-1 heading restores a two-cell body margin.

#### Scenario: Preserve source and internal formatting

- **GIVEN** a heading body containing a table, a nested list and a source block
- **WHEN** indentation is enabled, disabled, copied or saved
- **THEN** virtual cells never enter copied text, saved bytes or undo history
- **AND** relative source indentation, table fields and code remain intact.

### Requirement: OIN02 Configure and control the same buffer policy

`org.indent` SHALL be a boolean startup preference, default `true`,
admitted through existing composition, provenance and whole-candidate failure.
The complete catalog, packaged sample and reference SHALL include it together
when delivered. Runtime controls SHALL use the following shared command names:

| Command | Result |
| --- | --- |
| `org-indent-mode` | Toggle the effective value. |
| `org-indent-enable` | Enable it; repeated enablement is idempotent. |
| `org-indent-disable` | Disable it; repeated disablement is idempotent. |

M-x, modal colon and configured keybindings SHALL invoke the same handlers,
without new default shortcuts. Commands SHALL affect every view of the active
Org buffer, including read-only Org result buffers, and report the resulting
state. Other buffers SHALL retain their choices. Outside an active Org editing
view, report unavailability without enabling Org or changing another buffer.
Runtime choices SHALL not write TOML or mark the document modified.

#### Scenario: Override startup for one buffer

- **GIVEN** `org.indent = true` and two distinct Org buffers
- **WHEN** `org-indent-disable` is invoked in one buffer
- **THEN** its views use current unindented presentation
- **AND** the other buffer and startup configuration remain unchanged.

### Requirement: OIN03 Share one checked source and screen mapping

Rendering, wrapping, cursor placement, navigation, pointer hits, selections,
search highlighting and completion/result anchors SHALL use the same effective
projection. Continuations SHALL retain their logical line's structural margin;
heading continuations SHALL align with heading text. Visual Line commands SHALL
operate on the resulting visual rows. Source line numbers, bookmark locations,
Org folding and modal source operations SHALL keep their logical identities.

Virtual cells SHALL not become selectable text or fictitious source positions.
Clicking the virtual margin SHALL resolve to the start of that fragment's real
source span. Ordinary source-star positions remain addressable. Horizontal
scrolling with soft wrap disabled SHALL use the same checked projection.
An outdated layout SHALL not authorize an edit or pointer action.

#### Scenario: Wrap and select an indented paragraph

- **GIVEN** a long level-2 paragraph, a narrow pane and soft wrap enabled
- **WHEN** the user moves vertically, clicks a continuation and selects text
- **THEN** continuation rows retain the structural margin
- **AND** point, selection and copied text refer to the intended source only.

### Requirement: OIN04 Preserve policy through the buffer lifecycle

Splitting a buffer SHALL share its indentation policy while retaining each
view's geometry and independent pretty-heading choice. Switching buffers SHALL
restore each buffer's policy. Edits, undo/redo, structural changes, folding,
resize and external reload SHALL invalidate affected derived layout without
resetting an explicit runtime choice or moving canonical selection anchors.
Closing the buffer SHALL discard its override; reopening SHALL use startup
preferences. Disabling Org SHALL suspend indentation without losing its choice.

#### Scenario: Reflow two views after a heading edit

- **GIVEN** two differently sized views of one buffer and an explicit override
- **WHEN** a heading level changes and is subsequently undone
- **THEN** both views recompute affected body and continuation margins
- **AND** each keeps valid source anchors and the same buffer-local choice.

### Requirement: OIN05 Keep layout bounded and document the delivered behavior

Projection SHALL reuse native Org observations and bounded layout work. A frame
SHALL not scan an entire document, allocate padding proportional to unchecked
heading depth or schedule a new idle poller. Missing/stale structural evidence
SHALL use coherent literal presentation until current observations are ready;
stale margins and current text SHALL not be combined. Existing cache and work
ceilings SHALL remain enforced, including under repeated toggles and resize.

Delivery SHALL verify native geometry, one representative real-terminal journey
in each interaction profile and resource bounds. The Markdown Org guide SHALL
show the two presentations and commands; its Spore build SHALL derive from
that content. References and Unreleased SHALL describe only delivered behavior.

#### Scenario: Deep outline in a small pane

- **GIVEN** a deeply nested document and a pane narrower than its ideal margin
- **WHEN** indentation, resize and navigation are exercised repeatedly
- **THEN** layout remains finite, preserves source and permits forward progress
- **AND** cache retention and per-pass work remain within existing bounds.

## Scope and baseline reconciliation

This capability adds visual body/continuation indentation to the completed
heading and wrap features. Their source-prefix preservation remains true before
the added margin; width and wrap invariance apply to comparisons at the same
indentation policy. The delivered heading specification's "Reference comparison
and settled profile" and OGP04, its model's derived-view records, and the
soft-wrap specification/model's continuation and layout identity are reconciled
with this policy. Their historical receipts retain their original candidates;
the new indentation default is covered by the separate completed gate below.

No file-local STARTUP directive, configurable indentation width, source
reindentation command, glyph/font change or drawer-visibility change is added.
The three commands and single boolean are the complete new user surface.

## Completed delivery evidence

OIN01-OIN05 are delivered through four canonical slices, PRs #276-#279. The
controller passed the unchanged selected full gate at immutable dev candidate
`61e6e6a1b73d94577c7851aae6f200f6101d35c7`, receipt
`YE523W7IGTDKXEUNGCMA3NNJBY`. The [Slice 4 closure](../report/slices/rev-org-indentation/slice-4-acceptance.md#delivery-set-closure)
records 569 selected integration tests, eight native cache/work cases,
both-profile native/PTY journeys, three bounded resource cases and strict local
Spore validation derived from the canonical Markdown guide. The paired model,
plan and tracker are Completed. Earlier heading and wrapping receipts remain
historical evidence for their original layouts.
