---
id: TKT-20261113130719
title: Color Org headings by level
status: reviewing
reviewed_at: 2026-11-13T13:19:31Z
started_at: 2026-11-13T13:07:55Z
branch: feat/ticket-20261113130719-org-heading-colors
ready_at: 2026-11-13T13:07:55Z
kind: task
severity: low
priority: normal
scope: ui
tags: org-mode, colors, presentation
source: chat
reported_at: 2026-11-13T13:07:19Z
pr: https://forge.adrianpk.com/rev/rev/pulls/33
commits: [c61a88c51afebe1cce6e73a3ccb6e3236198644e]
---

# Color Org headings by level

## Requested behavior

Give each Org heading level its own Tokyo Night color treatment, following the
installed Emacs Org behavior. Apply the same level color to the rendered bullet
and heading text. Deliver the change in one pull request targeting dev.

## Reference and scope

The inspected Org configuration uses org-superstar markers
`◉ ○ ✸ ✿ ◆ ▶`, cycling every six levels. Rev already has that default sequence;
the separate personal Nerd Font override was removed without repository changes.
Keep the marker sequence and its configuration unchanged in this ticket.

Org cycles eight heading faces independently of its marker sequence. The
inspected Doom theme uses blue, purple/violet and lighter blue/purple faces.
Adapt those treatments from Rev's Tokyo Night accents rather than importing
another theme's RGB values. Keep per-level colors configurable through the
existing palette owner.

## Acceptance

- Color both bullet and heading text by level, cycling after eight levels.
- Preserve the independent six-marker cycle at deeper levels.
- Keep source stars, source bytes, dirty state and undo history unchanged.
- Preserve indentation, wrapping, clipping, cursor and selection coordinates.
- Preserve selection/search precedence, disabled coloring and monochrome output.
- Leave other Org roles and non-Org syntax highlighting unchanged.
- Validate palette overrides and document their defaults and inheritance.

## Delivery

One bounded ticket and one pull request. No delivery set, tracker or new
specification is required. Do not merge without the owner's explicit action.

## Validation

The focused native gate passed 231 tests: configuration (164), Org headings
(19), Org indentation (18) and syntax (30).

- `cargo test --quiet --locked --offline --test org_headings --test org_indentation --test syntax --test configuration`
- `cargo run --quiet --locked --offline --example documentation-check`: 106 reachable pages, 747 local links/anchors and 307 configuration entries.
- `bash scripts/build-user-guide.sh`: strict link validation and rendered guide.
- Exact staged commit gate: formatting, strict all-target/all-feature workspace Clippy and native source policy passed for all 1843 Rust files and 73 Cargo roots, with zero blocking diagnostics.

Cargo checks use one build job, disabled incremental output and no dev/test
debuginfo. New terminal-cell regressions cover 25 nesting levels, independent
marker/face cycles, source presentation, wrapping, clipping, Unicode, custom
palette overrides and selection/search precedence. Existing geometry/editing
and non-Org syntax regressions remain included in the focused gate.

## Review

PR #33 targets dev and is awaiting maintainer review and merge. Implementation
and native acceptance are recorded in commit `c61a88c51afebe1cce6e73a3ccb6e3236198644e`.
