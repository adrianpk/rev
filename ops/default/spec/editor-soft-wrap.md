# Word-Aware Soft Wrapping

Status: accepted
Date: 2026-10-31
Approved: product owner, 2026-10-31; selected five-topic delivery, Emacs interaction criterion and explicit Looper start request
Plan: [Wrap and Navigation](../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../tracker/rev-wrap-navigation.md)
Format: OpenSpec capability proposal
Source: approved visual direction in the [long-line ticket](../ticket/solved/20261028224018-define-comfortable-long-line-wrapping.md)
Model: [Soft Wrap Model](editor-soft-wrap-model.md)
Baselines: [rev Editor](rev-editor.md), [Editor Gutter](editor-gutter.md), [Org Mode](editor-org-mode.md)
Readiness: accepted behavior and finite delivery scope; implementation is owned by rev-wrap-navigation

## Purpose

Let long lines remain readable by continuing them across display rows, with
word-aware breaks, without changing the file. Keep an explicit no-wrap
alternative. Emacs Visual Line mode is the interaction reference for both
display and the selected line commands. Preserve maximum similarity to Emacs
in keybindings and behavior when an equivalent interaction exists.

## Confirmed decisions

- Wrapping is native Rust presentation, not a formatter or external package.
- Prefer word boundaries; source newlines, indentation and file bytes remain
  canonical. Hard wrapping/filling is a different editing operation.
- Enable word-aware wrapping by default at the available pane text width.
  Ordinary long text must remain readable without the user enabling a mode or
  routinely scrolling horizontally. The same factory policy
  applies to normal text buffers, including Org source and table lines.
- Keep absolute/relative numbering based on logical source lines. Display
  continuations do not acquire their own document line numbers.
- With wrapping enabled, follow Emacs Visual Line behavior: Up/Down and
  C-n/C-p move by visible rows; C-a/C-e reach the visible row's beginning/end;
  C-k kills to its visible end through the normal kill/history owners.
- Configuration and native M-x commands allow disabling and enabling wrapping.
  Use editor.soft_wrap = true and M-x visual-line-mode. The runtime override
  is buffer-local, shared by its views and discarded when the buffer closes.

## ADDED Requirements

### Requirement: WRP01 Preserve canonical text

Soft wrapping SHALL derive display rows from existing source without inserting
newlines or changing the canonical buffer, dirty state or history. The user
SHALL be able to choose an explicit no-wrap presentation. Saving, copying and
source-consuming features SHALL continue to use canonical text.

#### Scenario: Toggle presentation and save

- **GIVEN** a long logical line in an unchanged buffer
- **WHEN** the user enables wrap, disables it and saves
- **THEN** file bytes match the original text
- **AND** the presentation changes create no edit or undo transaction.

### Requirement: WRP02 Respect real view geometry

Wrapped layout SHALL use each view's admitted text width after panel, gutter
and other reserved cells. It SHALL prefer permitted word boundaries and keep
graphemes intact. It SHALL define a finite fallback for unbroken tokens, tabs,
wide characters and extremely narrow panes before delivery. Resize and split
changes SHALL invalidate obsolete layouts without changing source positions.

#### Scenario: Show one buffer at two widths

- **GIVEN** two views of one document with different text widths
- **WHEN** both display a long line with wrapping enabled
- **THEN** each uses its own checked display layout
- **AND** both refer to the same source line and buffer revision.

### Requirement: WRP03 Keep source and display coordinates distinct

Gutter numbering and goto-line SHALL identify logical source lines. Pointer
placement, linear/rectangular selection, search reveals and cursor presentation
SHALL resolve through the shared source/display projection. A continuation row
SHALL NOT become a new source line or an independent gutter marker location.
The selected visual-line commands SHALL follow WRP07. Logical source identity
SHALL remain authoritative for line numbers, goto-line and source ranges.

#### Scenario: Locate a wrapped source line

- **GIVEN** one logical line occupying several display rows
- **WHEN** goto-line targets that source line or search reveals text within it
- **THEN** the editor resolves the real source position and reveals it
- **AND** relative line distances still count source lines, not continuations.

### Requirement: WRP04 Compose with Org projections

Wrapping SHALL compose with the existing Org fold, heading, table and source
projections through their shared geometry owner. Hidden text SHALL stay hidden;
table/source edits SHALL remain source edits. Visible source lines, including tables and source blocks, SHALL use the
selected buffer wrap policy without an automatic region-specific no-wrap
exception. Source alignment remains canonical; a wrapped narrow table need
not look like a wider unwrapped table. Native Org commands still address
logical source rows and fields through the shared mapping.

#### Scenario: Reflow around a folded subtree

- **GIVEN** a folded Org subtree and visible long heading/body text
- **WHEN** wrapping or view width changes
- **THEN** only admitted visible text is projected
- **AND** the subtree's fold state and canonical source remain unchanged.

### Requirement: WRP05 Bound layout work and retain editing control

Layout storage and work SHALL have explicit bounds for very long logical
lines. Local movement SHALL NOT require an unbounded whole-document reflow.
Input, cancellation and rendering SHALL remain responsive. Recomputed layouts
SHALL be tied to current source, folding, settings and geometry observations.

#### Scenario: Edit a very long line

- **GIVEN** a document containing a long unbroken token
- **WHEN** the user edits, navigates and resizes its wrapped view
- **THEN** admitted layout work stays within its selected limits
- **AND** stale projections cannot redirect a pointer hit or text edit.

### Requirement: WRP06 Useful factory behavior and explicit overrides

Rev SHALL enable word-aware wrapping by default at each pane's available text
width. An ordinary text buffer SHALL NOT require configuration or a manual
toggle to keep long text visible. TOML preferences and compiled M-x commands
SHALL allow an explicit no-wrap choice without changing source bytes. This
default applies to visible document source, including structured Org text.
WRP07 owns the selected movement and line-editing behavior.

#### Scenario: Write a long line with factory preferences

- **GIVEN** an ordinary text buffer with no wrap preference or runtime override
- **WHEN** the user writes beyond the pane's available text width
- **THEN** the display continues the text using word-aware wrapping
- **AND** no manual wrap activation or inserted source newline is required.

### Requirement: WRP07 Follow Emacs Visual Line interaction

When wrapping is enabled, Up/Down and C-n/C-p SHALL traverse visual rows, and
C-a/C-e SHALL move point to the beginning/end of the current visual row. C-k SHALL
follow Emacs kill-visual-line behavior through the canonical source edit,
kill-ring and undo owners. With wrapping disabled, retain the existing logical
line behavior. Document numbering and goto-line SHALL remain source-based.
Consult the Emacs reference for boundary behavior rather than inventing another
line-editing convention.

#### Scenario: Navigate and kill within a wrapped logical line

- **GIVEN** one logical line displayed across multiple visual rows
- **WHEN** the user moves through it with C-n/C-p and uses C-a/C-e
- **THEN** motion addresses the visible rows and their boundaries
- **AND** C-k from inside a row kills through its visible end using normal
  source editing and undo, without treating display continuations as newlines.

## Accepted interaction profile

These choices apply the owner's Emacs criterion to the selected native editor
scope. They complete this proposal without adding another wrap mode or runtime.

- Factory/startup: editor.soft_wrap = true, validated through the existing
  whole-candidate TOML loader and included in both complete sample copies.
  False selects ordinary no-wrap/horizontal presentation. Startup remains the
  only settings-file load; runtime controls do not rewrite settings.
- Runtime: M-x visual-line-mode toggles the current buffer's effective policy.
  All views of that buffer share the choice, each at its own available width.
  Splitting preserves the choice; buffer close discards its override. Newly
  created buffers use the frozen startup default. Do not add a global toggle,
  per-language mode table or fixed-column wrap mode to this delivery.
- Breaks: prefer space/tab boundaries like ordinary Emacs Visual Line; if a
  token cannot fit, continue at checked grapheme boundaries. Keep existing tab
  expansion and unsupported/wide-cell fallback rules; never split UTF-8 or a
  grapheme. Add no automatic continuation prefix or extra indentation.
- Commands: ordinary arrows, C-n/C-p, C-a/C-e and C-k follow WRP07. Preserve
  goal-column and paging intent through the current source/display mapping.
  When wrapping is disabled, existing logical-line behavior returns. Contextual
  minibuffer, tree and Org-special command admission keeps its current owner.
- Gutter: label the first visible fragment of a logical source line; subsequent
  continuation rows have blank number cells. Number width remains based on
  source line count, and relative distances remain logical source distances.
- Selection: linear pointer selection uses mapped source positions. Existing
  rectangles still select logical source rows and columns, even when those
  rows have several visual fragments; never duplicate an edit per fragment.
  Search and goto-line reveal their checked source targets in the new layout.
- Org: apply the buffer's policy to visible source; folded text stays hidden.
  Table fields, source editing, links, capture and task actions keep canonical
  logical identities. Wrap alone does not realign tables, alter source blocks,
  insert whitespace or produce undo. Preview images retain their existing
  native projection contract.
- Bounded implementation: retain at most 8192 derived fragments per view;
  evict/recompute rather than truncate source or hide text after that count.
  Build large layouts in bounded resumable batches, with no whole-document
  scan on movement/render and no idle project scanner. Record the actual
  batch/work ceilings and measured long-line input/resize cost in the owning
  reports before handoff. These are internal safeguards, not new TOML knobs.

## Baseline reconciliation and validation

The current editor deliberately uses no wrap. Before implementation, prepare
complete modified clauses for rev Editor's text-display default and Editor
Gutter GUT03, including the copy in the consolidated editor specification.
Keep historical delivery records intact. These accepted proposals describe future changes,
not an existing toggle or current configurable key.

Validate exact byte preservation, undo/dirty state, split/resize, word and
token breaks, Unicode/tabs, gutter numbering, pointer/selection/search/goto,
Org folds/structured regions and worst-case long-line input work. The modal
draft must use the accepted motion semantics when it is planned; it is not a
prerequisite for delivering a coherent Emacs-like wrapping feature.

## Behavioral references

- [Emacs Visual Line mode](https://www.gnu.org/software/emacs/manual/html_node/emacs/Visual-Line-Mode.html).
- [Emacs continuation lines](https://www.gnu.org/software/emacs/manual/html_node/emacs/Continuation-Lines.html).
