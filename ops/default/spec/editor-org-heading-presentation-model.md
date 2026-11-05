# Org Heading Presentation Model

Status: Completed
Completed: 2026-11-03
Date: 2026-10-28
Reviewed: 2026-11-03
Version: 0.2
Approved: product owner, 2026-11-03; paired heading-presentation specification approval
Specification: [Org Heading Presentation](editor-org-heading-presentation.md)
Existing owners: [Org Mode Model](editor-org-mode-model.md), [Editor Model](rev-editor-model.md), [Configuration Model](editor-configuration-model.md)

## Ownership

Canonical Org source owns heading level and content. Native recognition owns
the current structural observation. A view projection owns decoration and
source/cell mappings. The terminal owns font selection and glyph shaping.
There is no alternate document text, heading hierarchy or font database.

## Approved preference records

| Key | Meaning | Default and contract |
| --- | --- | --- |
| `org.headings.pretty` | Whether an Org view initially decorates recognized heading prefixes. | Boolean, `true`; runtime changes do not write preferences. |
| `org.headings.bullets` | Ordered level-1-first glyph sequence. | `["◉", "○", "✸", "✿", "◆", "▶"]`; 1..=32 entries, at most 32 UTF-8 bytes per entry and 1024 bytes in total. |
| `org.headings.hide_leading_stars` | Whether the first N-1 source stars are drawn as blank cells in pretty view. | Boolean, `true`; preserve their cells and canonical source positions. |
| `org.headings.cycle_bullets` | How a deeper heading resolves after the sequence ends. | Boolean, `true`; false retains the final glyph. |

Delivered startup preferences, admitted by the native binary and packaged sample:

```toml
[org.headings]
pretty = true
bullets = ["◉", "○", "✸", "✿", "◆", "▶"]
hide_leading_stars = true
cycle_bullets = true
```

Every marker must be one printable non-whitespace grapheme occupying exactly
one cell under the existing native segmentation/width policy. Reject empty or
whitespace-only entries, controls and zero/two-cell markers. Reuse the existing
bookmark marker admission approach rather than introducing another Unicode
width library. Duplicate entries are valid. An empty list is invalid even when
`pretty` is false; configuration validity does not depend on runtime enablement.

Use existing startup configuration composition, provenance and whole-candidate
failure behavior. Omitted leaves inherit; a supplied marker array replaces the
entire inherited array rather than appending to it. Built-ins work without a
configuration file. The complete four-field catalog, exhaustive sample and
packaged template are delivered together. No font family, theme package or
separate configuration root is introduced.

For structural level `level >= 1` and list length `count >= 1`, use index
`(level - 1) % count` when cycling and `min(level - 1, count - 1)` otherwise.
This resolves level 7 to ◉ by default and ▶ when cycling is disabled. Level is
native structural evidence, not inferred from a wrapped or decorated screen row.

## Commands and view lifecycle

| Command | Active Org view outcome |
| --- | --- |
| `org-pretty-headings-mode` | Toggle the effective pretty flag and record a view-local override. |
| `org-reset-heading-presentation` | Clear that override and restore the immutable admitted startup flag. |

Both commands use the existing shared M-x/modal-colon catalog and may be bound
through existing context keymaps; no default keybinding is introduced. Outside
an active native Org view, report unavailability without changing mode, text or
other views. Read-only Org buffers admit these presentation controls.

Within heading settings, only the pretty flag has a runtime override. The marker list, hide-leading-stars
and cycle choices remain immutable startup settings. A split copies the
effective policy and override metadata by value; later changes are independent.
Like gutter presentation, a view retains its choice through buffer switches,
applies it only while Org is active and releases it when the view closes.
Source edits, undo and external reload retire derived mappings without resetting
the override. Reset never writes preferences or changes the interaction profile.

## Derived view records

| Record | Evidence and lifetime |
| --- | --- |
| Recognized heading | Existing Org document/revision, level and checked prefix/title ranges. Literal-block ownership is already resolved by the Org owner. |
| Effective heading presentation | Immutable heading settings and optional view-local pretty override, independent of the buffer-owned indentation policy. No text revision is created by either choice. |
| Prefix projection | Original source span, concealed-star cells, selected marker, admitted cell width and title boundary after the current structural margin. Current source-to-display mapping remains authoritative; the virtual margin has no source scalar. |
| Presentation snapshot | Buffer/revision, view/folding/geometry/settings, effective buffer indentation and current structural coverage, with bounded visible runs. Changed policy or stale mapping is rejected before pointer or edit actions. |

Preserve the original prefix width with N-1 leading cells and a final one-cell
marker. With hiding enabled, paint the leading cells blank; otherwise paint
their original stars. Disabled pretty mode paints all source stars. The existing
separator and original prefix width remain unchanged. Pretty/source comparisons
at the same effective indentation policy retain title and wrap positions. The
[Org Indentation Model](editor-org-indentation-model.md) adds its capped outer
heading/body/continuation margins through the shared source/display coordinate
owner. That buffer policy is independent of this view's pretty override.

Each leading cell maps to its original star, and the final marker cell maps to
the final star. Point entering the prefix or editing it does not reveal or
collapse additional cells. Selection highlights source-backed cells; copy,
save, undo and AI acceptance continue to use canonical text. A native source-view
toggle makes all stars readable without changing source point or geometry.

Only current recognized heading spans can be projected. Literal block text,
list items and unverified/incomplete observations retain source rendering.
Coloring disabled does not disable structural admission for enabled heading
presentation. Use the existing Org observation worker and its finite bounds;
cache viewport projection through the existing owner and retain no second
whole-document decoration index. The settings payload is bounded by the limits
above; current source/projection invalidation prevents stale pointer admission.

## Reference glyph identities

| Level | Glyph | Unicode identity |
| --- | --- | --- |
| 1 | ◉ | U+25C9 FISHEYE |
| 2 | ○ | U+25CB WHITE CIRCLE |
| 3 | ✸ | U+2738 HEAVY EIGHT POINTED RECTILINEAR BLACK STAR |
| 4 | ✿ | U+273F BLACK FLORETTE |
| 5 | ◆ | U+25C6 BLACK DIAMOND |
| 6 | ▶ | U+25B6 BLACK RIGHT-POINTING TRIANGLE |

These identities match the observed configuration. They are not a guarantee
of identical rasterization across fonts or terminals. No screenshot, private
configuration file, heading text or font file is stored by this model.

## Delivery evidence

The paired specification and model are Completed through canonical PRs #238-#240.
Controller receipt `T2QVMEBIU5HJTNBIOEXKDCUFSU` passed the unchanged full gate
at `18f625997de4c0ac2e1ae50d6fea667fe14f2c2d`. The
[Slice 3 closure](../report/slices/rev-org-headings/slice-3-acceptance.md#delivery-set-closure)
records source/configuration/view, read-only result, geometry, reload and real
terminal acceptance for the original heading-only layout. Glyph admission and
source-star identity retain that contract. The composed outer margins and policy
identity above belong to the later [Org Indentation delivery](../tracker/rev-org-indentation.md);
its acceptance does not relabel this historical receipt.
