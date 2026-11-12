# Tree Viewport Policy

Status: Accepted
Date: 2026-11-12
Specification: [File Tree Width and Horizontal Navigation](editor-tree-viewport.md)
Source: [TKT-20261108175339](../ticket/in_progress/20261108175339-improve-file-tree-panel-width-and-horizontal-navigation.md)

## Configuration contract

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `tree.auto_width` | boolean | `true` | New startup automatic/fixed policy |
| `tree.width` | existing integer, 1..=512 | `24` | Automatic preferred minimum or fixed preferred width |
| `tree.max_fraction` | existing finite float, 0.1..=0.5 | one third | Maximum frame share for actual tree geometry |

Admit the new leaf through partial documents, inherited/root precedence,
resolved defaults, source provenance, configuration catalog and complete sample.
Preserve existing width/fraction validation and atomic candidate rejection.
Runtime width/mode changes never modify stored configuration bytes. No new
database, state file, migration, public SDK/wire field or stored session record
is introduced.

## Session ownership

| Value | Owner and invariant |
| --- | --- |
| Configured policy | Existing admitted startup snapshot; reset restores it |
| Session width/mode override | Tree viewport owner; explicit commands alone replace it |
| Automatic desired-width high-water mark | Tree context; finite width of admitted expanded content, from preferred minimum |
| Desired width | Independent of temporary actual terminal geometry |
| Expanded content extent | Existing retained model plus shared sanitized display-column presentation; includes offscreen rows |
| Actual width | Existing layout owner, bounded by frame, maximum fraction and small-frame rules |
| Horizontal offset | One nonnegative display-column value shared by all rows; bounded by current content extent and actual width |
| Selected/active route and pointer hit | Existing tree/follow/pointer owners; never inferred from clipped text |

Keep width and horizontal state cohesive with the tree. Use checked/saturating
display-column arithmetic at native representation limits; do not accumulate
unbounded width or scan collapsed descendants. Measurement and rendering must
agree on the unpanned row shape, sanitizing and glyph cell widths. Clipping at
a partial wide cell retains alignment without drawing half a glyph.

## Transitions

| Event | Width policy | Horizontal viewport |
| --- | --- | --- |
| Initial observation | Configured automatic fit or fixed preference | Beginning |
| Expand / explicit refresh / admitted listing publication | Automatic mark can grow; manual choice retained | Retain, clamp if needed |
| Fold / content removal | Automatic mark does not shrink | Retain, clamp excess to current content |
| Selection / vertical scroll / focus / hide-show | Retain | Retain |
| Root or Project/Workspace context change | Fresh automatic mark; preserve manual desired width | Beginning |
| Terminal resize | Clamp actual geometry; retain desired width, no directory reads | Retain or clamp excess |
| Widen / narrow | Four-column desired-width step; manual ownership | Retain or clamp excess |
| Enable automatic mode | Fresh fit of current model from configured minimum | Retain or clamp excess |
| Disable automatic mode | Freeze current desired width as session fixed choice | Retain or clamp excess |
| Reset width | Clear overrides/old mark; restore configured policy | Beginning |
| Pan left / right | Unchanged | One bounded display-column step |

Same-context membership publication is an admitted listing change. Opening a
different Project/Workspace/root resets automatic measurement and horizontal
position but must carry a manual desired width across the transition. Existing
context persistence remains unchanged.

## Identity and failure rules

Panning/width changes do not open files, discover directories, mutate source,
alter history or substitute roots. Retain stale-hit rejection by row/path/root
identity and native content bounds. Expected listing/configuration failures
continue through existing owners. A hidden/unusable panel must not underflow
offsets or discard the desired width needed when space returns.
