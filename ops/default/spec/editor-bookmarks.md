# Native Bookmarks

Status: draft
Date: 2026-10-27
Reviewed: 2026-10-31
Format: OpenSpec capability proposal
Source: product-owner discussion; captured for later refinement
Delivery boundary: future capability; no implementation or slicing authorized
Gutter owner: [Editor Gutter](editor-gutter.md)
Model: [Native Bookmarks Model](editor-bookmarks-model.md)
Readiness: native bookmark/gutter direction confirmed; location and lifecycle choices remain open

## Purpose

Capture the selected future bookmark capability: native locations that can be
navigated through Rust commands and represented by real gutter markers. This
is separate from implementing line numbers and from rev's existing transient
mark/selection state.

## Confirmed direction

- Bookmarks are a future feature, not part of the accepted numbering delivery.
- Bookmark semantics need a real native owner; the gutter only presents them.
- User controls follow the TOML/compiled M-x principle where appropriate.
- Naming, persistence, edit anchoring and navigation details remain undecided.

## ADDED Requirements

### Requirement: BMK01 Canonical bookmark ownership

A native bookmark owner SHALL define checked location identity and the meaning
of its references. The gutter SHALL derive marker presentation from that owner
without making rendered cells canonical bookmark state. A bookmark SHALL remain
semantically distinct from the transient mark used for editing selections.

#### Scenario: Hide bookmark presentation

- **GIVEN** an admitted bookmark and its derived gutter marker
- **WHEN** the user hides the gutter
- **THEN** the owner retains the bookmark under its selected lifecycle policy
- **AND** no document text or selection state is changed by hiding presentation.

### Requirement: BMK02 Coherent edits and navigation

Before delivery, the owner SHALL specify insertion/deletion/replacement,
undo/redo, rename and close behavior for bookmark locations. Invalid or stale
locations SHALL NOT silently navigate to an unrelated current position. The
supported bookmark workflow SHALL be available through compiled Rust commands.

#### Scenario: A bookmarked location becomes invalid

- **GIVEN** a bookmark whose location evidence no longer resolves
- **WHEN** the user attempts to navigate to it
- **THEN** the owner reports the selected stale/unavailable outcome
- **AND** it does not claim an unrelated position is the original valid location.

### Requirement: BMK03 Derive current gutter markers

Marker presentation SHALL use the current bookmark/location observation and
the existing gutter geometry. Scrolling, resizing, numbering mode or hiding the
gutter SHALL NOT change bookmark identity. Glyphs, collisions and resource
limits SHALL be specified before delivery; stale locations SHALL NOT produce
functional-looking markers on unrelated source lines.

#### Scenario: Resize a bookmarked view

- **GIVEN** a valid bookmark shown through the gutter
- **WHEN** the view is resized or its numbering mode changes
- **THEN** its marker is derived from the current source/view mapping
- **AND** bookmark identity and document text remain unchanged.

### Requirement: BMK04 Provide native user controls

The accepted bookmark workflow SHALL be available through compiled editor
commands with TOML and runtime controls where appropriate. Setting or removing
a bookmark SHALL NOT insert marker characters into document text. Exact
command names, shortcuts and persistence effects SHALL be decided explicitly.

#### Scenario: Mark a source location

- **GIVEN** a location admitted by the chosen bookmark policy
- **WHEN** the user invokes the native marking operation
- **THEN** the bookmark owner records the admitted location
- **AND** saving the document does not add bookmark glyphs to its source.

## Dependencies and planning readiness

Reuse canonical buffers/locations, native commands and the delivered gutter.
Bookmarks do not require completion, LSP or AI. The modal jump history is a
different capability, even if both reuse checked location primitives.

Before planning, resolve the choices below and specify their concrete
navigation/edit/lifecycle examples in this spec and model. Draft capture is
not implementation approval; no storage format or command catalog ships yet.

## Open decisions

- Named versus unnamed bookmarks; buffer-local, project or global scope.
- Session-only versus persistent storage and its actual XDG ownership/schema.
- Exact edit gravity, deletion, rename, undo and saved/unsaved-buffer behavior.
- Set/remove/list/next/previous commands, ordering and any factory bindings.
- Marker glyph, role, placement and collision behavior in the gutter.
- Limits, cleanup, recovery and native/terminal acceptance evidence.

These questions intentionally remain open. The model is provisional and does
not implement bookmark entities, storage or a generic marker framework.
