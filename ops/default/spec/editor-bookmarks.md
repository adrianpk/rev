# Native Bookmarks

Status: draft
Date: 2026-10-27
Format: OpenSpec capability proposal
Source: product-owner discussion; captured for later refinement
Delivery boundary: outside rev-configuration-ui; no implementation or slicing authorized
Gutter owner: [Editor Gutter](editor-gutter.md)
Model: [Native Bookmarks Model](editor-bookmarks-model.md)

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

## Open decisions

- Named versus unnamed bookmarks; buffer-local, project or global scope.
- Session-only versus persistent storage and its actual XDG ownership/schema.
- Exact edit gravity, deletion, rename, undo and saved/unsaved-buffer behavior.
- Set/remove/list/next/previous commands, ordering and any factory bindings.
- Marker glyph, role, placement and collision behavior in the gutter.
- Limits, cleanup, recovery and native/terminal acceptance evidence.

These questions intentionally remain open. The model is provisional and does
not implement bookmark entities, storage or a generic marker framework.
