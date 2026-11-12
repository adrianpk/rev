---
id: TKT-20261108175339
title: Resize the file-tree panel and reveal long nested names
status: in_progress
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: file-tree, panels, navigation
source: chat
reported_at: 2026-11-08T17:53:39Z
commits: [77f96b3b69c959fdeff8f0f9590ac074d0c0a7c6]
pr: https://forge.adrianpk.com/rev/rev/pulls/30
branch: feat/rev-tree-viewport-01-width
started_at: 2026-11-12T20:57:59.810116Z
ready_at: 2026-11-12T20:53:38.391200Z
---

# Resize the file-tree panel and reveal long nested names

## Reported behavior

The file-tree panel can leave too little room to identify entries when a file
name is long, directory nesting consumes the available width, or both occur.
The user needs control over the panel width and a way to see the end of a name
without letting the tree occupy an unlimited part of the editor.

This is a captured product request, not a newly reproduced runtime regression.

## Requested outcome

- Provide native functions to widen and narrow the tree panel in steps of four
  terminal display columns, with associated keybindings. Make them discoverable
  through M-x and the modal command selector with descriptions and effective
  shortcut hints.
- A manual width change is authoritative: suspend automatic fitting and retain
  that chosen width as a session override until another explicit width/mode
  action. Automatic layout logic must not undo the user's adjustment.
- Provide a normalize/reset-width function that clears the manual override and
  returns to the configured width policy, stable automatic mode by default.
  The browser-zoom comparison describes these three actions, not their shortcuts.
  Use the approved tree-local and prefixed width bindings below.
- Use stable automatic width as the requested initial policy. The current
  preferred width is its normal minimum, not a target to shrink below when names
  happen to be short. The inspected factory value is 24 columns; native geometry
  constraints still govern small frames.
- Add tree.auto_width as a boolean preference, defaulting to true. With true,
  tree.width is the automatic minimum; with false, it is the fixed preferred
  width. Reuse tree.max_fraction and existing native geometry constraints.
  Runtime width/mode changes remain session-only and never rewrite TOML.
- Calculate required width from the furthest rendered right edge of the relevant
  expanded tree rows, including rows outside the vertical viewport. Folded
  descendants do not contribute until the user expands their branch. Indentation,
  markers, icons and label width all contribute; measuring only the name is insufficient.
- Selection movement, vertical scrolling and focus changes must not resize the
  panel. A long name entering/leaving the viewport must not cause width oscillation.
  This policy is called stable automatic width.
- Bound automatic growth so source buffers retain useful editing space. Reuse
  the existing tree.max_fraction constraint and its current one-third default;
  keep ordinary native small-frame/layout constraints in effect. No unbounded
  expansion is permitted for an outlier name or nesting depth.
- When the automatic width reaches its limit, allow horizontal navigation to
  reveal the rest of a selected long name, including its end. The capped width
  must not make the remaining name permanently inaccessible.
- Horizontal navigation pans the entire tree content through one shared
  horizontal offset, not only the selected label. Rows retain their common
  alignment while the panel width and selected entry remain unchanged.
- With tree focus, each Left or Right press moves that horizontal viewport by
  one terminal display column, bounded by its admitted range. This is ordinary
  navigation within the panel and requires no command-selector invocation.
  Enter opens a file or toggles a directory; C-f and C-b retain their expansion
  and collapse/parent-selection operations.
- Preserve existing tree selection, focus, expansion/collapse, active-buffer
  following, keyboard navigation and pointer hit-testing while resizing or
  changing the horizontal viewport. Horizontal navigation must not accidentally
  open an entry or change the root. Rebinding Left/Right is intentional;
  directory operations remain available through the approved keys above.
- Retain both minimal and decorated tree presentations, including Nerd Font and
  ordinary Unicode variants. Measure terminal display columns rather than bytes
  or Unicode scalar counts; do not split a displayed glyph when clipping/panning.

## Agreed design

Slice 1 implements the width commands; horizontal scroll commands remain planned for Slice 2:

- tree-widen and tree-narrow for manual step changes.
- tree-reset-width for normalization to the configured width policy.
- tree-auto-width-mode for switching automatic sizing.
- tree-scroll-left and tree-scroll-right for revealing clipped label text.

Approved default width bindings:

| Binding | Context | Action |
| --- | --- | --- |
| > | File tree | Widen by four display columns. |
| < | File tree | Narrow by four display columns. |
| C-x t > | Editor buffers and file tree | Invoke the same widen action. |
| C-x t < | Editor buffers and file tree | Invoke the same narrow action. |
| C-x t = | Editor buffers and file tree | Restore the configured width policy. |

Unmodified < and > apply only while the tree has focus. They must not replace
ordinary text input in editing buffers. All actions also remain available by
command name through the existing command selectors.

Approved tree navigation bindings:

| Binding | Action |
| --- | --- |
| Left | Move the horizontal viewport one display column toward the beginning. |
| Right | Move the horizontal viewport one display column toward the end. |
| Enter | Open the selected file or expand/collapse the selected directory. |
| C-f | Expand the selected directory. |
| C-b | Collapse the selected directory or select its parent. |

The agreed calculation includes relevant offscreen rows, rather than sizing to
the selected entry or current viewport. The preferred width is a floor and the
selected finite cap is an upper bound. A maliciously long name or extreme nesting
must reach that cap rather than consume the source-editing area.

Use all rows in the expanded tree model, including those below/above the vertical
viewport. Do not inspect folded directories or retain their hidden descendants
as inputs merely to anticipate future expansion. The implementation already
loads direct children when expanding; width measurement uses that admitted model.

Keep an automatic desired-width high-water mark within the current tree context.
Expanding a branch or explicitly refreshing its observed entries can increase
that desired width up to the cap. Folding, selecting another row, vertical
scrolling, focus changes and hiding/showing the tree do not shrink it. Thus
opening and closing a branch does not make the editing area oscillate.

When auto mode owns width, an explicit root/context change starts a fresh
measurement from the preferred minimum. While a manual override owns width,
navigation, branch expansion, refresh and root changes retain the chosen width.
A terminal resize can clamp actual geometry without discarding the manual or
automatic desired width to restore when space returns. It does not scan directories.

Normalize clears the session width/mode override and the old automatic high-water
mark. It restores configured policy: with automatic sizing enabled, measure the
currently expanded model again from the preferred minimum under the cap; with
fixed sizing configured, restore the configured preferred width. Normalization
therefore need not produce exactly 24 columns when current expanded names need
more space or the user's configuration specifies another base.

Runtime adjustments do not silently rewrite TOML preferences. The automatic
minimum is not permission to counteract an explicitly chosen smaller manual
width; native admissible geometry still bounds the actual panel.

Each manual widen/narrow action changes the desired width by four terminal
display columns. A proportional step based on the available display columns is
a possible future refinement, not part of this delivery.

Approved default configuration shape for the planned change:

```toml
[tree]
auto_width = true
width = 24
```

auto_width is the new preference; width and max_fraction already exist. Changing
auto_width to false selects the fixed-width policy. An explicit runtime width
or mode action overrides that configured policy only for the current session;
normalization clears the override and restores the configured policy. Existing
width/fraction validation and small-frame geometry rules still apply.

All calculation is finite over the already observed model. Rendering and cursor
movement never discover the filesystem. This selects expanded/offscreen rows,
not collapsed subtrees, as the automatic fitting scope.

Use a common horizontal viewport for all tree rows. Indentation, markers, icons
and names move together within the fixed panel; do not shift only the selected
filename or resize the panel as a side effect of panning. Preserve the shared
horizontal offset across selection changes, vertical navigation, focus changes
and hiding/showing the tree. A root/context change or explicit width normalization
returns the horizontal viewport to its beginning.

Clamp the offset to the nonnegative range admitted by current expanded content
and actual panel width. If content becomes shorter or the panel becomes wider,
reduce the offset only as needed to avoid an empty horizontal tail. Changing
the selected row alone does not reset it or recenter a shorter label.

## Current source observations

- src/config.rs already defines tree.width and tree.max_fraction. The inspected
  defaults are 24 preferred columns and a maximum share of one third.
- src/tree.rs constrains the preferred width to frame geometry and hides the
  tree on an unusably small frame. Rows are presented within the supplied width.
- src/command/tree.rs contains existing selection, expansion, root and entry
  actions. The inspected catalog has no manual width or horizontal-pan action.
- src/command/window.rs provides focus, close/keep and balance actions; source
  splits and the dedicated tree area have their existing layout ownership.
- ops/default/spec/editor-better-file-tree.md and its model own the established
  minimal/decorated presentation baseline.

Related contracts: [Better File Tree](../../spec/editor-better-file-tree.md) and
its [model](../../spec/editor-better-file-tree-model.md).

These observations guide later design; they do not justify changing the current
defaults or implementation during capture.

## Acceptance direction

- Exercise a shallow long file name, deeply nested short names, and a nested
  long name. Each can be inspected fully through the admitted width/pan controls.
- Manual widen/narrow actions move by four display columns and stop safely at
  geometry limits. Their keybindings work in the intended tree context without
  conflicting with native movement or cancellation.
- Verify tree-local < and > and the three approved C-x t width bindings. The
  prefixed widen/narrow bindings invoke the same actions from an editing buffer;
  unmodified angle brackets still insert text normally there.
- After a manual change, navigate, expand/collapse, refresh and change roots:
  automatic fitting cannot replace the chosen width. Normalize restores the
  configured policy using the current tree, without changing persisted settings.
- Verify the default and explicit tree.auto_width = true policies and an explicit
  false configuration. tree.width acts as the automatic minimum or fixed preferred
  width respectively. Runtime adjustments leave the TOML file unchanged, and
  normalization restores the admitted startup policy in either case.
- Automatic mode accounts for nesting and presentation, respects its cap and
  keeps useful editing space. Long outliers cannot claim the entire terminal.
- With only short labels, auto mode retains the normal preferred minimum instead
  of tightly compacting the panel. Narrow frames still follow native constraints.
- Put a long nested entry below the vertical viewport and navigate between short
  and long entries. The calculated content width accounts for the relevant
  offscreen row, and selection/vertical scrolling cannot change panel width.
- Put an extreme name inside a folded directory. The panel does not inspect or
  size for that descendant before expansion. Expanding can grow to the cap;
  folding again retains the established width without collapsing the source area.
- Hide/show the tree and change focus without losing its width. A root/context
  change resets the measurement; a small terminal clamps actual geometry safely.
- At the cap, horizontal navigation reveals the selected entry's ending and
  permits returning to its beginning without changing its identity.
- Pan horizontally with several rows visible: every row uses the same offset,
  preserving their relative alignment without changing panel width or selection.
- Each Left/Right press pans one display column, stopping at the admitted
  horizontal limits without opening files or expanding/collapsing directories.
  This step is independent of the four-column width adjustment. Enter and
  C-f/C-b retain their admitted file/directory operations.
- After panning, change rows, scroll vertically, change focus and hide/show the
  tree: retain the shared horizontal position. Change roots or normalize width:
  return to the beginning. Shorter content or a wider panel clamps excess offset
  without otherwise resetting the user's position.
- Exercise spaces, combining characters, wide characters and Nerd Font icons in
  minimal/decorated presentations. Clipping, selection and mouse targeting agree.
- Resize the terminal, hide/show the tree, change roots and navigate between
  entries. Apply the selected state/reset policy consistently and avoid jitter.
- Demonstrate that fitting does not expand directories, perform unbounded scans,
  discover the filesystem from rendering/cursor movement or mutate source text,
  history, roots or open buffers.

## Design decisions

- Width bindings, the four-column manual step, preferred-width floor, existing
  max_fraction cap, stable automatic initial policy, tree.auto_width preference
  and session-only overrides are settled.
- The one-column horizontal step, Left/Right bindings, whole-tree panning,
  common alignment, fixed panel width and viewport reset/clamping rules are
  settled. No product questions remain open from this capture round.

## Delivery boundary

Product alignment is complete. On 2026-11-12 the owner authorized the
specification and established implementation workflow. This ticket retains its
identity and is in progress for the two-slice Tree Viewport delivery.

- [Specification](../../spec/editor-tree-viewport.md)
- [Model](../../spec/editor-tree-viewport-model.md)
- [Plan](../../plan/rev-tree-viewport.md)
- [Tracker](../../tracker/rev-tree-viewport.md)

The native Looper controller owns implementation, canonical PR integration,
the selected exact-candidate gate and documentation/ticket closure.
