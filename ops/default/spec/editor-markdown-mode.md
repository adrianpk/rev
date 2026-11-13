# Markdown Editing and Presentation

Status: Approved
Date: 2026-11-13
Approved: product owner, 2026-11-13; specification, slicing and the established autonomous Looper workflow
Format: OpenSpec capability addition
Source: product-owner request for Markdown presentation and editing comparable to Org
Inspection baseline: `47c8b2210d8e043640f488e1024ea7c7a2389265`
Authorization: preparation, slice PRs and controller-owned integration through dev
Model: [Markdown Policy and Actions](editor-markdown-mode-model.md)
Plan: [Markdown Delivery](../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../tracker/rev-markdown.md)
Readiness: approved parser dependency verified; ready for autonomous delivery

## Purpose

Provide native Markdown headings, list presentation, syntax colors and practical
table editing through Rev's existing source editor. Include table creation,
alignment, field navigation and insertion/deletion of rows and columns.
The scope is document presentation and structural editing; Org task management,
table formulas and a separate HTML preview are outside this capability.

## Baseline and references

Rev recognizes `.md` and offers managed Marksman integration, but its bundled
grammar catalog has no Markdown entry. Heading projection and native table
commands currently belong to Org. Reuse shared coordinates, colors, transactions
and worker lifecycle while keeping Markdown syntax and actions format-owned.

- Syntax: [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/).
- Tables: [GFM 0.29-gfm, section 4.10](https://github.github.com/gfm/#tables-extension-).
  Only this extension is adopted here; this is not a full-GFM conformance claim.
- Existing behavior: [syntax](../../../docs/reference/syntax.md),
  [heading presentation](editor-org-heading-presentation.md),
  [heading colors](../../../docs/reference/org-mode.md#heading-colors) and
  [table editing](../../../docs/reference/org-tables.md).

## Visual defaults

These defaults form the approved visual acceptance profile.
Colors come from Rev's existing Tokyo Night palette and its heading tints.

| Heading level | ATX marker | Heading color |
| --- | --- | --- |
| 1 | ◉ | `#7aa2f7` |
| 2 | ○ | `#9d7cd8` |
| 3 | ✸ | `#c0caf5` |
| 4 | ✿ | `#9bb9f9` |
| 5 | ◆ | `#b59ce1` |
| 6 | ▶ | `#bcd0fb` |

Unordered list markers cycle `•`, `◦`, `▪` by logical nesting depth, independently
of heading markers. Ordered lists retain their source numbers and delimiters.
Setext titles use their level's color and retain their source underline;
marker substitution applies to ATX prefixes. Source presentation reveals the
original markers and keeps syntax colors. Font coverage belongs to the terminal.

## ADDED Requirements

### Requirement: MD01 Recognize Markdown structure and literal contexts

Rev SHALL recognize `.md` and `.markdown` through existing association precedence,
show Markdown mode and honor explicit Text mode. Recognition SHALL follow the
references for ATX/Setext headings, nested lists, quotes, inline markup, fenced
and indented code, and tables. Styling and structural actions SHALL use current
context; Markdown-looking content inside literal code SHALL remain literal.

#### Scenario: Distinguish headings from literal source

- **GIVEN** the fixture below in a recognized Markdown buffer
- **WHEN** its current structure is observed
- **THEN** the six ATX headings and two Setext headings have their correct levels
- **AND** the seven-hash paragraph and fenced content are not headings or tables.

### Requirement: MD02 Present headings and semantic colors

Pretty presentation SHALL apply the heading markers and colors to the
bullet and ordinary heading text. Emphasis, strong emphasis, links, inline code,
code blocks, quotes and table structure SHALL have palette-based treatments.
Inline semantic colors SHALL override an enclosing heading/table color;
selection SHALL precede search, which SHALL precede syntax. NO_COLOR SHALL retain
readable monochrome selection/search cues. Disabling syntax colors SHALL leave
independently enabled marker presentation available.

#### Scenario: Resolve the visual profile

- **GIVEN** factory colors and pretty presentation
- **WHEN** the fixture is displayed, then switched to source presentation
- **THEN** its six ATX markers/colors match the profile above before switching
- **AND** switching restores the source prefixes without an edit or point change.

### Requirement: MD03 Present and edit lists by their actual nesting

Rev SHALL decorate unordered markers by logical depth, preserving indentation,
continuation paragraphs and ordered numbering. Continue-item SHALL insert a
sibling after a nonempty item, preserving its bullet style or advancing its
number. On an empty item it SHALL exit one list level. Indent/outdent SHALL
preserve the selected item's child/continuation structure and reject an invalid
move without partial edits. Each explicit list edit SHALL be one undo unit.

#### Scenario: Continue a nested item

- **GIVEN** point at the end of `Child` in the fixture
- **WHEN** Continue-item is invoked
- **THEN** a sibling with the same source `+` marker and indentation is inserted
- **AND** one undo restores the exact source and canonical point.

### Requirement: MD04 Edit GFM tables without losing cell source

Rev SHALL create a table on a blank line from explicit column/body-row counts,
including its required header and delimiter. It SHALL align source padding,
honor `:---`, `---:` and `:---:`, navigate next/previous field while skipping the
delimiter, and move to the same field in the next body row. Forward navigation
at the final field/row SHALL append a body row. Previous-field at the first
header cell SHALL keep point there. Insert/delete actions SHALL operate on body
rows and whole columns, preserving the header/delimiter and at least one column.

Cell bytes, escaped pipes, inline markup and Unicode SHALL survive alignment and
structural edits. Already aligned navigation SHALL add no history. Each source
change SHALL be one checked undo unit; retained cell points in other panes SHALL
follow their text. Actions SHALL reject ambiguous or lossy shapes, including
extra body cells that GFM rendering would ignore, rather than discard content.

#### Scenario: Align and traverse a table

- **GIVEN** the fixture's two-column table
- **WHEN** Align-table and Next-field from its last body cell are invoked
- **THEN** columns retain their left/right alignment and the escaped pipe remains in one cell
- **AND** navigation creates a new body row; undo restores the source before that insertion.

#### Scenario: Create and change table structure

- **GIVEN** a blank line and requested dimensions of two columns and one body row
- **WHEN** Create-table, Insert-column, Insert-body-row and their delete actions run
- **THEN** every intermediate table retains a valid header/delimiter and its cell values
- **AND** deleting the final column or the header as a body row rejects unchanged.

### Requirement: MD05 Keep presentation separate from source edits

Rendering, toggling presentation and resizing SHALL preserve source bytes,
BOM/newline style, dirty state and history. Copy SHALL return canonical Markdown.
Cursor, mark, pointer hits, wrapping and clipping SHALL use checked source/cell
mappings, including concealed markers and wide/combining graphemes. Presentation
controls SHALL work in read-only buffers; source-changing actions SHALL respect
read-only authority. Both interaction profiles SHALL expose the same named actions.

#### Scenario: Select, copy and save decorated source

- **GIVEN** the fixture with Unicode and a selection crossing a decorated prefix
- **WHEN** the view wraps, scrolls, splits, toggles presentation, copies and saves
- **THEN** copy/save contain original syntax and the same byte/newline identities
- **AND** source positions remain valid and presentation adds no undo entry.

### Requirement: MD06 Bound analysis and preserve existing editor behavior

Observation SHALL respect configured syntax byte limits, finite work budgets,
cancellation and revision checks. Pending, canceled or stale observations SHALL
never authorize structural edits; ordinary source editing SHALL remain available.
Retain the existing Org table bounds for the initial Markdown table profile:
8 MiB source/output, 10,000 rows, 256 columns, 100,000 cells and 4,096 displayed
columns per cell. Native Markdown SHALL work offline without an LSP. Existing
Org behavior, other language highlighting and configured LSP behavior SHALL remain intact.

#### Scenario: Retire stale structural work

- **GIVEN** a table action pending on a particular buffer revision
- **WHEN** source changes, cancellation occurs, or a table bound is exceeded
- **THEN** the action publishes no partial edit or stale cursor destination
- **AND** ordinary text editing and other buffers remain usable.

## Reproducible acceptance

Save this UTF-8 fixture as `sample.md`, using LF and no BOM for the first run.
The accented `é` contains `e` followed by U+0301; `界` is a wide character.

````markdown
# Level 1
## Level 2
### Level 3
#### Level 4
##### Level 5
###### Level 6
####### Plain paragraph

Setext one
==========

Setext two
----------

- Parent
    + Child
        * Leaf

3. Ordered
4. Next

> A quote with *emphasis*, **strong**, [guide](https://example.com/guide) and `code`.

| Name | Value |
| :--- | ---: |
| 界é | `a\|b` |

```text
# Literal heading
- Literal item
| Literal | pipes |
| --- | --- |
```
````

Use named actions from the final command catalog; record resolved names and keys
with the tested commit and configuration. Verify terminal cells, not only ANSI
escape presence. The acceptance receipt SHALL cover this compact matrix:

| Case | Required evidence |
| --- | --- |
| Presentation | Six heading faces/markers, Setext levels, three list depths, ordered source numbers and literal-code protection. |
| Table syntax | Left/center/right delimiters; optional outer pipes; escaped pipes including inline code; reject mismatched header/delimiter and lossy extra cells. |
| Editing | Create/align/navigate/insert/delete; empty and ordered list continuation; indent/outdent; exact one-action undo/redo and read-only refusal. |
| Geometry | Representative 80x24 and 32x12 frames, wrapping on/off, horizontal clipping, wide/combining characters and shared-pane points. |
| Fidelity | Copy/save byte comparison, then repeat representative cases with BOM+CRLF, source presentation, disabled colors and NO_COLOR. |
| Lifecycle | Edit/undo/reload invalidation, stale/canceled/over-limit rejection, both input profiles, offline operation and focused Org/non-Markdown regressions. |

## Delivery decisions

The linked model records the command, preference and ownership contract derived
from this approved scope. The owner accepted `pulldown-cmark` 0.13.6 subject to
license verification. Its MIT license and the selected transitive graph were
verified compatible with Rev's GPL-3.0-only license. Default features remain
disabled; only the table parsing option is enabled. Retain required notices.
A private adapter preserves the option of a separately approved owned parser
later; this delivery does not create that successor project.
