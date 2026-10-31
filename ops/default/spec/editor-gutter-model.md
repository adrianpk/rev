# Editor Gutter Model

Status: Completed
Completed: 2026-10-28
Validated candidate: `dedd85064cb5565a5382a9d8e2e2902daae24d88`
Gate validation: `PZ4OIPXA6GUBSWDH5ZS7MJUOGX`
Approved: product owner, 2026-10-27; specification and model accepted.
Date: 2026-10-27
Specification: [Editor Gutter](editor-gutter.md)
Configuration owner: [Editor Configuration Model](editor-configuration-model.md)
View owner: [rev Editor Model](rev-editor-model.md)
Delivery: [Input and Navigation](../plan/rev-input-navigation.md); integrated gate passed

## Control model

All gutter behavior has usable factory defaults. User preferences persist in
the existing TOML file; native M-x commands change active-view session state.
No configurable command language, provider registry or second settings store is
introduced. The input/navigation delivery implements this schema extension; the earlier
configuration/UI gate remains historical evidence for its original surface.

| Concern | TOML startup default | Native M-x control | Runtime scope |
| --- | --- | --- | --- |
| Visibility | `gutter.visible = true` | Toggle | Active editing view |
| Calculation mode | `gutter.mode = "absolute"` | Set absolute/relative, cycle | Active editing view |
| Current-line form | `gutter.current_line = "absolute"` | Cycle absolute/zero/hidden | Active editing view |
| Side | `gutter.side = "left"` | Toggle left/right | Active editing view |
| Minimum number digits | `gutter.minimum_digits = 4` | Startup setting | Global default used by views |
| Left padding | `gutter.left_padding = 0` | Startup setting | Global default used by views |
| Right padding | `gutter.right_padding = 1` | Startup setting | Global default used by views |
| Palette | Number and bookmark `palette.gutter` roles below | Startup setting | Renderer palette snapshot |
| Bookmark visibility | `bookmarks.markers = true` | `bookmark-markers-mode` | Active editing view |
| Bookmark glyph | `bookmarks.marker = "●"` | Startup setting | Immutable view startup glyph |
| Effective policy/source | Derived from resolved settings and overrides | Inspect | Active editing view |
| Restore configured defaults | Existing immutable startup settings | Reset | Active editing view |

### Configuration shapes

| Key | Type and bounds | Default |
| --- | --- | --- |
| `gutter.visible` | Boolean | `true` |
| `gutter.mode` | `absolute`, `relative` | `absolute` |
| `gutter.current_line` | `absolute`, `zero`, `hidden` | `absolute` |
| `gutter.side` | `left`, `right` | `left` |
| `gutter.minimum_digits` | Integer, 1..20 | `4` |
| `gutter.left_padding` | Integer, 0..4 terminal cells | `0` |
| `gutter.right_padding` | Integer, 0..4 terminal cells | `1` |
| `palette.gutter.background` | Six-digit RGB | `#1a1b26` |
| `palette.gutter.foreground` | Six-digit RGB | `#565f89` |
| `palette.gutter.current_line` | Six-digit RGB | `#7aa2f7` |
| `palette.gutter.separator` | Six-digit RGB | `#3b4261` |
| `palette.gutter.bookmark` | Six-digit RGB | `#e0af68` |
| `bookmarks.markers` | Boolean | `true` |
| `bookmarks.marker` | One printable non-control grapheme, terminal width one | `●` |

Colors reuse the pinned Tokyo Night night palette selected by the existing
configuration model. Omitted leaves inherit through built-ins, includes and
root under the existing partial-value rules. These fields are global/view
preferences; the existing language editing tables do not acquire new gutter
keys implicitly. Invalid preferences reject the whole startup candidate.

Number labels always fit the complete admitted buffer line index. The fixed
implementation limit of 20 digits is not a configurable truncation maximum.
The default separator is a checked one-cell `│` with printable `|` fallback.
Arbitrary separator glyphs and lane declarations are not part of this scope.

### Startup TOML

```toml
[gutter]
visible = true
mode = "absolute"
current_line = "absolute"
side = "left"
minimum_digits = 4
left_padding = 0
right_padding = 1

[palette.gutter]
background = "#1a1b26"
foreground = "#565f89"
current_line = "#7aa2f7"
separator = "#3b4261"
bookmark = "#e0af68"

[bookmarks]
markers = true
marker = "●"
```

These keys and alternatives are documented in the exhaustive reference sample
and packaged template alongside their native handlers.
Startup loading remains unchanged: editing TOML affects a later startup, while
M-x operations change view-local overrides immediately.

## Compiled command identities

Every listed command must have a real Rust handler and be discoverable through
M-x. Aliases use existing context-keymap validation; no executable config body
is evaluated. The explicit names retained here define the native control surface.

| Command | Active-view outcome |
| --- | --- |
| `gutter-toggle-line-numbers` | Hide/restore numbers, retaining mode. |
| `gutter-cycle-line-number-mode` | Alternate absolute and relative; retain visibility. |
| `gutter-set-line-numbers-absolute` | Select absolute mode and show numbers. |
| `gutter-set-line-numbers-relative` | Select relative mode and show numbers. |
| `gutter-cycle-current-line` | Cycle absolute, zero and hidden; retain mode/visibility. |
| `gutter-toggle-line-number-side` | Move the number field left/right. |
| `gutter-inspect-numbering` | Report effective policy and startup/view override source. |
| `gutter-reset-numbering` | Clear overrides and restore the resolved startup gutter policy. |

Commands do not alter TOML or immutable startup defaults. Each changed leaf is
an explicit view override. A split inherits its source's effective policy and
override metadata by value; later changes are independent. Reset restores the
startup policy, including visibility, width and padding, for that view only.

## Runtime records and lifecycle

| Record | Semantic fields | Owner / lifetime |
| --- | --- | --- |
| Partial gutter settings | Optional visibility, mode, current-line form, side, minimum digits and padding. | Existing layered TOML candidate; absence remains explicit. |
| Effective startup defaults | Complete admitted gutter leaves and palette roles. | Existing immutable session snapshot. |
| View gutter policy | Checked view identity, effective values and overridden leaves. | Existing editing view; retained across buffer switch. |
| Number geometry | Side, digit width, padding/separator widths, field bounds and text bounds. | One panel projection; terminal cells, not document coordinates. |
| Number row | Real logical line, current-row flag, label and semantic role. | Visible projection only; no per-buffer-line retained array. |

Reuse existing buffer/view identities and source line metadata. Closing a view
releases its policy/projection; switching its buffer invalidates old labels and
recomputes width, without changing the view's chosen presentation policy.
The number records introduce no marker store or provider generation registry.
The fixed bookmark projection borrows the separate canonical bookmark owner;
its derived index and view visibility do not introduce another source/history owner.

## Fixed marker projection

The optional one-cell area is allocated from the remaining content after the
complete available number field. Its side follows `gutter.side`; the final text
area is shared by painting, reveal, wrapping, pointer hits and search. The marker
is omitted first under narrow geometry, preserving the three-column text minimum.
Both requested policies survive temporary omission. Reset clears number overrides
only; it does not change the independent marker visibility.

The buffer owns a derived logical-row set keyed by source stamp, canonical file
identity and collection generation. Revision-current resolved anchors populate
it using the rope's line index; peer views reuse it. A record mutation, edit,
load/resolution or path change rejects the old index. Closing the buffer releases
it. Each visible row performs a set lookup, not a collection scan. No index or
view policy is serialized.

Multiple names on one logical line coalesce to one glyph. Only the first visual
fragment receives it. Folded bodies, continuation rows and filler remain blank.
Lazy storage admission occurs through native session transitions on first enabled
marker demand for a file-backed view. Painting neither reads storage/targets nor
relocates or saves records. The original numbering validation metadata above
remains historical; the bookmark tracker owns the extension's separate evidence.

## Related contracts

Bookmark state belongs to the separate [bookmark specification](editor-bookmarks.md)
and its [model](editor-bookmarks-model.md). Completion and optional
server processes belong to [native autocompletion](editor-autocompletion.md)
and [LSP support](editor-language-server-support.md).
[Server installation](editor-language-server-installation.md) owns optional
server payloads and installation records. This gutter model introduces none
of those entities, stores or adapters.
