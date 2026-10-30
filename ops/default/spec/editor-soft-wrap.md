# Word-Aware Soft Wrapping

Status: draft
Date: 2026-10-31
Format: OpenSpec capability proposal
Source: approved visual direction in the [long-line ticket](../ticket/open/20261028224018-define-comfortable-long-line-wrapping.md)
Model: [Soft Wrap Model](editor-soft-wrap-model.md)
Baselines: [rev Editor](rev-editor.md), [Editor Gutter](editor-gutter.md), [Org Mode](editor-org-mode.md)
Readiness: source-preserving word-aware display approved; motion semantics and defaults remain open

## Purpose

Let long lines remain readable by continuing them across display rows, with
word-aware breaks, without changing the file. Keep an explicit no-wrap
alternative. Emacs Visual Line mode is the display reference; its changes to
editing commands are not implicitly adopted.

## Confirmed decisions

- Wrapping is native Rust presentation, not a formatter or external package.
- Prefer word boundaries; source newlines, indentation and file bytes remain
  canonical. Hard wrapping/filling is a different editing operation.
- Keep absolute/relative numbering based on logical source lines. Display
  continuations do not acquire their own document line numbers.
- Configuration and native runtime commands expose the admitted controls.
  Exact defaults, names and override lifetime still require a decision.

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
The accepted navigation matrix SHALL specify visual versus logical behavior
of every affected line motion and editing command.

#### Scenario: Locate a wrapped source line

- **GIVEN** one logical line occupying several display rows
- **WHEN** goto-line targets that source line or search reveals text within it
- **THEN** the editor resolves the real source position and reveals it
- **AND** relative line distances still count source lines, not continuations.

### Requirement: WRP04 Compose with Org projections

Wrapping SHALL compose with the existing Org fold, heading, table and source
projections through their shared geometry owner. Hidden text SHALL stay hidden;
table/source edits SHALL remain source edits. The wrap policy for structured
regions SHALL be decided explicitly rather than silently breaking alignment
or using a separate cursor/selection engine.

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

## Decisions before planning

1. Factory mode, any per-language distinction, startup settings and runtime
   override/reset/split inheritance. No specific default is selected here.
2. Viewport-only versus an optional column limit; break rules, continuation
   indentation/markers, unbroken-token fallback and tabs/wide graphemes.
3. Logical or visual semantics for Up/Down, C-n/C-p, C-a/C-e, C-k, paging and
   desired columns; retain explicit command identities for either behavior.
4. Horizontal scrolling, rectangles and pointer mapping across continuations;
   continuation gutter labels and coexistence with optional heading glyphs.
5. Whether structured Org regions such as tables and source blocks wrap;
   projection ordering with folds, and the future modal motion matrix.
6. Measured finite layout/cache/input budgets and acceptance fixtures.

## Baseline reconciliation and validation

The current editor deliberately uses no wrap. Before implementation, prepare
complete modified clauses for rev Editor's text-display default and Editor
Gutter GUT03, including the copy in the consolidated editor specification.
Keep historical delivery records intact. These drafts describe future changes,
not an existing toggle or current configurable key.

Validate exact byte preservation, undo/dirty state, split/resize, word and
token breaks, Unicode/tabs, gutter numbering, pointer/selection/search/goto,
Org folds/structured regions and worst-case long-line input work. The modal
draft must use the accepted motion semantics when it is planned; it is not a
prerequisite for delivering a coherent Emacs-like wrapping feature.
