# File Tree Selection and Active Buffer

Status: Completed
Completed: 2026-11-07
Validated candidate: `27e222f65a81bbdb0d3aece0e7279877faf997b2`
Gate log: `KGGQSFRVEFDYE7B4A3KIT2FGIO`
Date: 2026-11-06
Updated: 2026-11-07
Inspection baseline: `978deacc49f6f5f948cea2070919ce91031f67a2`
Format: OpenSpec capability addition
Approved: product owner; prepare the plan, slicing and tracker and start one Looper delivery on 2026-11-07
Source: [TKT-20261031093408](../ticket/solved/20261031093408-highlight-tree-selection-and-sync-active-buffer.md)
Baselines: [File navigation](editor-file-navigation.md), [Better file tree](editor-better-file-tree.md), [Files, Directories, Projects and Workspaces](editor-projects-workspaces.md)
Model: [Tree Selection Policy](editor-tree-selection-model.md)
Owner decision: active-buffer synchronization is configurable and enabled by default
Plan: [File Tree Selection Delivery](../plan/rev-tree-selection.md)
Tracker: [File Tree Selection Tracker](../tracker/rev-tree-selection.md)

## Purpose

Make the tree's selection visible after focus moves into an editing pane.
Follow the active editing buffer by default, while allowing users to retain
manual tree selection through configuration. Preserve minimal and decorated
presentation, configured click behavior and explicit root ownership.

The owner approved configurability and the enabled default. On 2026-11-07 the
owner approved this complete focus, matching and reveal policy and requested the
plan, slicing, tracker and Looper delivery. One implementation slice is selected.

## Baseline observations

Before this delivery, `src/display/tree.rs` painted the selected row only
while the tree panel was active. `src/tree.rs` owns selection and listing identity. Native file opening
already distinguishes explicit roots from automatic file-parent roots. Workspace
rows also carry member root identity, independently of their displayed name.
Existing `palette.tree_selected` and `palette.tree_selected_text` roles already
define full-row selection colors; the missing behavior is their continued
use outside tree focus and following actual editing-buffer transitions.
This addition reuses those owners instead of adding a second file navigator.

These baseline observations motivated the delivered capability. The completed
implementation and gate evidence are recorded in the
[Slice 1 report](../report/slices/rev-tree-selection/slice-1-follow.md).

## Policy

```toml
[tree]
follow_active_buffer = true
```

Read the boolean at startup through existing configuration admission. It does
not require a project-local file or an additional state directory. Existing
configuration files that omit it adopt true on the next startup; they are not
rewritten. Preserve normal validation, include/override precedence and provenance.

With following enabled, a different active editing file synchronizes the tree
when its entry is in the loaded, expanded listing. The entry need not already
be inside the viewport: an actual follow transition may scroll the tree only
enough to reveal it. It does not change the editing pane's point or viewport.

When the tree owns focus, its keyboard browsing cursor owns the highlight.
When editing owns focus, the active-file indication owns the highlight. A
prompt retains the role of its originating tree or editing owner; opening
M-x is not a file transition. There is at most one highlighted tree row.
Merely moving through unopened entries does not switch buffers or reset the
browsing cursor. Returning to editing resumes its active-file indication.
The retained browsing cursor remains distinct and is not lost when a file
cannot be represented in the tree.

With following disabled, the last selected tree entry remains highlighted
after focus leaves it; changing editing buffers does not move that selection.
This preference changes following, not the availability of persistent selection
feedback. Use existing tree-selection palette roles, full-row selection and
readable NO_COLOR reverse video. Keep the real cursor and focused footer marker
exclusive to actual tree focus. Apply the same behavior to minimal/decorated
trees and ordinary/Nerd Fonts symbols without adding glyphs or palette roles.

Synchronization does not expand collapsed directories, expose filtered entries,
show a hidden tree or change root ownership. If the active file is not in the
loaded listing, show no active-file highlight while editing owns focus; do not
mark its parent, the previous file or a row that merely occupies the old index.
Explicit expansion or refresh can make the correct entry eligible later.
Manual scrolling is not undone on every repaint; the next actual follow
transition can reveal its target again.

Following itself performs no filesystem reads. Reuse admitted source/listing
identities and observations from ordinary opening, expansion, refresh and
rename operations. Respect automatic file-parent rooting already owned by
ordinary file visits, without adding rerooting on pane focus or buffer switching.

## Matching and context rules

| Situation | Behavior |
| --- | --- |
| Two file panes | Follow the currently active editing pane; moving point within that same file does not trigger another follow. |
| Tree or prompt focus | Retain the originating editing view and allow manual tree navigation without a competing follow. |
| Unnamed or non-file buffer | No active-file highlight; retain the browsing cursor for later tree focus. |
| File outside the chosen roots | No active-file highlight and no root replacement or implicit Workspace membership. |
| Collapsed, filtered or unavailable entry | No fabricated highlight; preserve expansion and filtering choices. |
| Hidden or geometry-suspended tree | Retain state without showing it or reading directories; recompute the eligible indication when it becomes visible. |
| Registered Project | Follow within its existing directory tree; do not register or switch a Project. |
| Workspace with overlapping visible routes | Prefer the retained matching browsing route; otherwise choose the first matching entry in declared root order. Highlight one route only. |
| Rename, Save As, close or source replacement | Resolve the current visiting identity again after the real operation succeeds. A failed operation leaves the prior valid state intact. |

Use existing canonical source and alias rules. A label, basename, clipped path
or stale row number never proves identity. An alias can match only when its
association is established by existing admitted observations; following does
not resolve symlinks or probe paths during drawing. Workspace matches include
root membership identity so overlapping routes cannot silently substitute one
another. If no reliable match exists, leave the active-file indication absent.

## ADDED Requirements

### Requirement: TSV01 Configure default active-buffer following

The editor SHALL admit `tree.follow_active_buffer` as a boolean with a default
of true. False SHALL retain manual tree selection across editing-buffer changes.
It SHALL use existing configuration merging, validation, provenance and complete
sample documentation, without another state file. Selection feedback SHALL
remain available with either value. Omission SHALL adopt true without rewriting
the user's TOML; a non-boolean value SHALL follow existing candidate rejection.

#### Scenario: Default following

- **GIVEN** no override and two visible file rows
- **WHEN** the active editing pane switches to the other file
- **THEN** the tree's active-file indication follows that file.

#### Scenario: Manual preference

- **GIVEN** following is disabled and a selected tree entry
- **WHEN** another editing buffer becomes active
- **THEN** the tree retains its manual selection and visible indication.

### Requirement: TSV02 Retain visible selection across focus changes

A selected or active entry SHALL remain identifiable after focus transfer,
including after activating an already displayed file. Focused tree browsing and
active editing-file identity SHALL remain distinct state even when one highlight
is used for the current focus owner. Reuse the existing selected-row palette
roles and monochrome fallback without changing glyphs or requiring a font.

#### Scenario: Repeated activation

- **GIVEN** the clicked file is already displayed
- **WHEN** the tree activates it and focus returns to editing
- **THEN** a visible highlight confirms the selected file without reopening it.

#### Scenario: Browse without activation

- **GIVEN** following is enabled and the tree owns focus
- **WHEN** keyboard navigation selects an unopened file
- **THEN** its browsing cursor remains usable until an actual editing-context
  transition occurs, without changing the displayed buffer.

#### Scenario: Selection-only click

- **GIVEN** a configured selection-only click policy
- **WHEN** a click selects a different tree entry
- **THEN** that entry is highlighted under tree focus without opening it
- **AND** returning to editing shows the actual active file when following is enabled.

### Requirement: TSV03 Follow the relevant editing pane without false identity

Following SHALL use the active editing pane, or the most recently active editing
pane while a tree or command prompt owns input. Prompt focus alone SHALL NOT
substitute a minibuffer or result buffer for that editing origin. Existing
canonical path/buffer rules SHALL own identity, including aliases. A deliberately
selected editing buffer that has no visiting file SHALL clear the active-file
indication rather than retaining a stale source-file highlight.

#### Scenario: Two editing panes

- **GIVEN** two panes display different files whose tree rows are visible
- **WHEN** mouse or keyboard focus changes between those editing panes
- **THEN** the indication follows the newly active editing file.

#### Scenario: No matching visible row

- **GIVEN** an unnamed buffer, an out-of-root file or a collapsed matching row
- **WHEN** that editing buffer becomes active
- **THEN** no unrelated entry is presented as its active file
- **AND** the explicitly chosen root and expansion choices are preserved.

#### Scenario: Overlapping Workspace roots

- **GIVEN** two loaded routes represent the same canonical file
- **WHEN** its editing pane becomes active
- **THEN** one retained matching route is highlighted, or the first match in declared root order
- **AND** following neither duplicates a buffer nor changes Workspace membership.

### Requirement: TSV04 Keep synchronization within native UI ownership

Selection changes SHALL NOT edit text, create undo or usage records, open files
or change root policy. Paint SHALL perform no filesystem I/O. Resolve identities
on existing buffer/focus/listing transitions, not on every frame or idle poll.
Preserve scrolling, refresh, resize, input profiles and both tree presentations.
Match only the existing loaded listing; add no recursive scan, polling worker
or new capacity limit. Actual file/focus changes and listing publication may
recompute the match; moving point or repainting the same file SHALL NOT do so.

Source mutations and effects retain their existing owners. Following SHALL NOT
save or revert a buffer, create a history entry, invoke a file-opening command,
alter context registrations or affect save/discard/cancel decisions.

#### Scenario: Refresh and resize

- **GIVEN** a highlighted entry with stable canonical identity
- **WHEN** the listing refreshes or the terminal resizes
- **THEN** the appropriate selection remains associated with that entry
- **AND** rendering uses admitted data without scanning the filesystem.

#### Scenario: Reveal an offscreen loaded entry

- **GIVEN** the active file has an expanded listing entry outside the viewport
- **WHEN** a real file/pane transition synchronizes its indication
- **THEN** the tree scrolls only enough to reveal that entry
- **AND** collapsed directories, editing point and source viewport remain unchanged.

#### Scenario: Preserve a manual collapse

- **GIVEN** the user collapsed the active file's ancestor
- **WHEN** editing regains focus or the frame is redrawn
- **THEN** the ancestor remains collapsed and no other entry impersonates the file
- **AND** later explicit expansion can restore its eligible highlight.

#### Scenario: Pointer target changes before release

- **GIVEN** a tree press has captured a row identity
- **WHEN** refresh or a follow transition changes its projection before release
- **THEN** the existing checked hit policy rejects a stale target
- **AND** the release cannot activate the new occupant of that row.

## Review and acceptance

The approved reveal boundary preserves collapsed directories: following reveals
only entries already present in the loaded listing, not hidden descendants.
Slice 1 delivered TSV01-TSV04 through canonical PR #308. The controller passed
the selected gate on `27e222f65a81bbdb0d3aece0e7279877faf997b2`; the linked report
records its exact command and evidence. Requirements and scenarios retain
the approved acceptance contract.

Cover configured true/false and omission/invalid values, already displayed files,
buffer switching, two panes, tree browsing, click preferences, missing rows,
overlapping Workspace routes, aliases, refresh, scroll, resize, NO_COLOR and
decorated/minimal rendering. Extend existing native tree/pointer/display and
configuration tests; use representative cases instead of a Cartesian matrix.
Assert unchanged source bytes, dirty state, point, mark and history, and keep
matching/drawing free of extra filesystem operations and idle polling.
Document the setting in configuration reference/sample and ordinary tree use
in the Markdown User Guide, then project it to Spore.
