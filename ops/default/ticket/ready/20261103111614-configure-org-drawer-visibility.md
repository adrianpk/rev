---
id: TKT-20261103111614
title: Configure Org drawer visibility
status: ready
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: org-mode, drawers, visibility
source: chat
reported_at: 2026-11-03T11:16:14Z
ready_at: 2026-11-07T18:58:12Z
commits: []
---

# Configure Org drawer visibility

## Requested behavior

Allow users to open Org buffers with drawers collapsed and reveal their contents
explicitly. Follow Emacs Org visibility conventions where they fit rev's native
configuration and command model. Keep the distinction between drawer visibility
and heading/body visibility clear.

The owner reviewed an Org explanation preview whose answer precedes a REQUEST
drawer containing request metadata. That presentation was easier to read than
the current AI result header. The earlier question about closing every drawer
in a buffer is part of this visibility follow-up.

## Source observations at capture

- Org views start fully visible. The configuration catalog exposes no initial
  drawer visibility setting.
- TAB toggles the drawer at point. org-hide-drawer, org-show-drawer and
  org-toggle-drawer affect the containing drawer.
- There is no drawer-only whole-buffer command. Global visibility cycling also
  changes heading/body visibility; org-show-all currently reveals drawer bodies.
- Visibility belongs to each view, not to source text. Existing folds preserve
  logical source positions and protect incomplete structural observations.

## Emacs reference

The official initial-visibility manual documents org-startup-folded for the
global policy and #+STARTUP directives for individual files. Its default is
showeverything, rather than an unconditional collapsed-drawer policy.

The cycling and drawer references distinguish ordinary visibility cycling,
which keeps drawers collapsed, from explicitly showing everything including
drawers. This is a behavioral reference; it does not establish that rev already
supports those directives or that every Emacs startup state belongs in this
bounded change.

- [Initial visibility](https://orgmode.org/manual/Initial-visibility.html)
- [Global and local cycling](https://orgmode.org/manual/Global-and-local-cycling.html)
- [Drawers](https://orgmode.org/manual/Drawers.html)

## Expected outcome

- Offer a documented TOML startup policy for collapsed or expanded drawers.
  Apply it when initializing an Org view, without repeatedly overriding a
  user's subsequent visibility choices.
- Permit folding or revealing all admitted drawers in the current buffer's
  selected view without folding headings or hiding ordinary explanation text.
  Expose named commands through M-x and modal colon; keep the existing per-drawer
  controls available.
- Cover custom drawers such as REQUEST and property drawers. Literal text that
  resembles drawer delimiters inside source/example blocks must remain literal.
- Preserve independent pane visibility, source identities, cursor admission,
  selection, scrolling and the existing bounded observation behavior. Folding
  must not alter source text, mark the buffer modified, save files or create
  editing undo entries.
- Let a future generated Org explanation initialize REQUEST collapsed using
  this native visibility owner, rather than duplicating folding inside AI code.

## Approved specification and delivery

On 2026-11-07 the owner requested specification, planning and Looper delivery.
[Org Initial Visibility and Drawers](../../spec/editor-org-initial-visibility.md)
and its [model](../../spec/editor-org-initial-visibility-model.md) define the
complete policy. The [plan](../../plan/rev-org-initial-visibility.md) and
[tracker](../../tracker/rev-org-initial-visibility.md) select one vertical slice.

- Preserve showeverything as the ordinary-file compatibility default. Support
  overview, content, showall and showeverything through org.startup_folded.
- Recognized document STARTUP metadata overrides configuration. Ignore unrelated
  words and literal/drawer examples; last recognized token in source order wins.
- Add org-hide-all-drawers, org-show-all-drawers, org-toggle-all-drawers and
  org-cycle-set-startup-visibility with complete descriptions in both selectors.
- Keep initial policy one-time and per-view. Explicit choices suppress pending
  initialization; splits clone current folds. Edits/reload retire stale ranges
  without automatically reapplying startup. Preserve current org-show-all,
  cycling, source navigation and deliberate AI REQUEST initialization.

## Acceptance direction

Use documents with several custom/property drawers and visible prose between
headings. Check collapsed and expanded startup, per-drawer expansion, whole-view
drawer folding/reveal, split panes and explicit show-everything behavior. Include
delimiter-like text inside literal blocks and an incomplete drawer. Verify that
ordinary body text remains visible, source bytes and dirty state do not change,
and navigation/selection continue to use canonical source positions.

## Relationship and delivery boundary

The later Pi acceptance adjustment now presents AI results as native Org with
REQUEST initially folded. That solved presentation change leaves this general
Org startup/whole-view visibility ticket pending; it is outside the heading-marker
delivery.

This is an Org visibility ticket, related to the delivered
[Org Mode specification](../../spec/editor-org-mode.md) and the
[visibility reference](../../../../docs/reference/org-mode.md).
The separate AI presentation delivery is complete. This ready ticket owns only
the approved general visibility behavior. Implementation, validation and later
lifecycle transitions belong to the recorded Looper delivery.
