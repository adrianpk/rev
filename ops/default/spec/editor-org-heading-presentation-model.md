# Org Heading Presentation Model

Status: draft
Date: 2026-10-28
Reviewed: 2026-10-31
Version: 0.1
Specification: [Org Heading Presentation](editor-org-heading-presentation.md)
Existing owners: [Org Mode Model](editor-org-mode-model.md), [Editor Model](rev-editor-model.md), [Configuration Model](editor-configuration-model.md)

## Ownership

Canonical Org source owns heading level and content. Native recognition owns
the current structural observation. A view projection owns decoration and
source/cell mappings. The terminal owns font selection and glyph shaping.
There is no alternate document text, heading hierarchy or font database.

## Proposed preference records

| Field | Meaning | Contract |
| --- | --- | --- |
| Enabled | Whether an Org view initially decorates recognized heading prefixes. | Final default remains for review; runtime changes do not write preferences. |
| Bullets | Ordered level-1-first glyph sequence. | Reference sequence is ◉, ○, ✸, ✿, ◆, ▶; validate a finite nonempty list of visible single-cell graphemes. |
| Leading stars | Source versus concealed/indented presentation. | Source stars remain canonical; exact indentation/reveal policy must be admitted. |
| Depth overflow | How a deeper heading resolves after the sequence ends. | Cycling versus retaining the last glyph is unresolved. |

Illustrative future spelling; these keys are not currently admitted:

```toml
[org.headings]
# Proposed presentation preferences; no font family is selected here.
pretty = true
bullets = ["◉", "○", "✸", "✿", "◆", "▶"]
hide_leading_stars = true
```

Use existing startup configuration composition, provenance and whole-candidate
failure behavior. Do not create a theme package, font installer or separate
configuration root. Runtime view overrides must remain separate from the
immutable admitted startup snapshot.

## Derived view records

| Record | Evidence and lifetime |
| --- | --- |
| Recognized heading | Existing Org document/revision, level and checked prefix/title ranges. Literal-block ownership is already resolved by the Org owner. |
| Effective heading presentation | Startup settings, optional view-local override and settings generation. No text revision is created by changing it. |
| Prefix projection | Original source span, concealed-star cells, selected marker, admitted cell width and title boundary. Current source-to-display mapping remains authoritative. |
| Presentation snapshot | Buffer/revision, view/folding/geometry/settings observations and bounded visible runs. Stale mapping is rejected before pointer or edit actions. |

Preserving the original prefix width with blank leading-star cells and a final
single-cell marker is the proposed first geometry. Any reveal-on-edit policy
must update the same projection owner, not change source or invent a second
cursor. Source copy/save/undo continue to use native document operations.

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
