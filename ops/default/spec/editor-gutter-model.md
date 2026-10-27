# Editor Gutter Model

Status: accepted
Approved: product owner, 2026-10-27; specification and model accepted.
Date: 2026-10-27
Specification: [Editor Gutter](editor-gutter.md)
Configuration owner: [Editor Configuration Model](editor-configuration-model.md)
View owner: [rev Editor Model](rev-editor-model.md)
Implementation boundary: a later delivery; active configuration/UI scope unchanged

## Control model

All gutter behavior has usable factory defaults. User preferences persist in
the existing TOML file; native M-x commands change active-view session state.
No configurable command language, provider registry or second settings store is
introduced. This is an accepted later schema extension, not a current sample or
an addition to the active configuration/UI delivery.

| Concern | TOML startup default | Native M-x control | Runtime scope |
| --- | --- | --- | --- |
| Visibility | `gutter.visible = true` | Toggle | Active editing view |
| Calculation mode | `gutter.mode = "absolute"` | Set absolute/relative, cycle | Active editing view |
| Current-line form | `gutter.current_line = "absolute"` | Cycle absolute/zero/hidden | Active editing view |
| Side | `gutter.side = "left"` | Toggle left/right | Active editing view |
| Minimum number digits | `gutter.minimum_digits = 4` | Startup setting | Global default used by views |
| Left padding | `gutter.left_padding = 0` | Startup setting | Global default used by views |
| Right padding | `gutter.right_padding = 1` | Startup setting | Global default used by views |
| Palette | Four `palette.gutter` roles below | Startup setting | Renderer palette snapshot |
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

Colors reuse the pinned Tokyo Night night palette selected by the existing
configuration model. Omitted leaves inherit through built-ins, includes and
root under the existing partial-value rules. These fields are global/view
preferences; the existing language editing tables do not acquire new gutter
keys implicitly. Invalid preferences reject the whole startup candidate.

Number labels always fit the complete admitted buffer line index. The fixed
implementation limit of 20 digits is not a configurable truncation maximum.
The default separator is a checked one-cell `│` with printable `|` fallback.
Arbitrary separator glyphs and lane declarations are not part of this scope.

### Illustrative future TOML

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
```

Document these keys and supported alternatives in the exhaustive sample when
actually implemented. Do not add them to the currently running delivery.
Startup loading remains unchanged: editing TOML affects a later startup, while
M-x operations change view-local overrides immediately.

## Compiled command identities

Every listed command must have a real Rust handler and be discoverable through
M-x. Aliases use existing context-keymap validation; no executable config body
is evaluated. Do not claim Emacs or Odra command-name compatibility beyond the
explicit names retained here.

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
No marker store, provider generation registry or future bookmark/LSP adapter
is introduced by these records.

## Related future specifications

Bookmark state belongs to the separate [bookmark draft](editor-bookmarks.md)
and its [candidate model](editor-bookmarks-model.md). Completion and optional
server processes belong to [native autocompletion](editor-autocompletion.md)
and [LSP support](editor-language-server-support.md).
[Server installation](editor-language-server-installation.md) owns optional
server payloads and installation records. This gutter model introduces none
of those entities, stores or adapters.
