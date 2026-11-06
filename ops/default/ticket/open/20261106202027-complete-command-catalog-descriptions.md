---
id: TKT-20261106202027
title: Complete command catalog descriptions
status: open
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: commands, descriptions, completion
source: chat
reported_at: 2026-11-06T20:20:27Z
commits: []
---

# Complete command catalog descriptions

## Owner decision

On 2026-11-06 the owner selected a three-column command selector: command name,
nearby keybinding and description, using rev's palette. The initial presentation
change should consume optional descriptions and leave the field blank when
metadata is absent.
Populating every description is separate work and must not delay that layout.

This ticket captures the later catalog pass. It is not a new invocation mode,
command-search policy or requirement to reproduce another editor's command set.

## Expected outcome

- Give every shipped canonical command a concise, accurate, single-line English
  description of its user-visible action. Cover bound and unbound commands and
  all native command families, not just common examples.
- Store descriptions with the authoritative command metadata. Aliases reuse
  their canonical action's description; avoid a second command/dispatch catalog
  or presentation-only copies that can drift.
- Keep the initial selector's name/keybinding/description layout and rev palette.
  This pass fills metadata rather than redesigning geometry again.
- Leave the field blank for commands without description metadata; do not add
  dashes or other missing-description placeholders.
  Do not use saved prompts, arguments or source content as inferred descriptions.
- Preserve command-name filtering, ranking, Tab text, canonical invocation,
  effective keybinding discovery and exactly-once usage recording.

## Validation direction

Check the complete admitted built-in catalog for nonempty, single-line
descriptions. Retain a meaningful catalog completeness guard so future command
additions cannot silently omit
their description. Review the text for behavioral accuracy and consistency.

Inspect representative bound/unbound families in both M-x and the modal colon
selector. Verify description clipping, selected-row contrast, NO_COLOR and
user-defined fallback without changing matching, completion or usage identity.
Do not multiply all catalog entries through complete terminal journeys.

## Dependency and delivery boundary

The separate selector-presentation change establishes the optional metadata
accessor and adjacent shortcut column, leaving missing descriptions blank.
Complete the built-in descriptions in one coordinated pass after that accessor exists. This ticket
does not populate the catalog or start a controller during capture.

Related baseline: [Command Keybinding Hints](../../spec/editor-command-keybinding-hints.md).
