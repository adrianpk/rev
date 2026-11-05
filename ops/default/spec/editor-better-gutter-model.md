# Better Gutter Model

Status: Approved
Date: 2026-11-06
Approved: product owner; specification and one autonomous Looper delivery through dev
Specification: [Better Gutter](editor-better-gutter.md)
Plan: [Better Gutter Delivery](../plan/rev-better-gutter.md)
Tracker: [Better Gutter Tracker](../tracker/rev-better-gutter.md)
Baseline models: [Editor Gutter](editor-gutter-model.md), [Native Bookmarks](editor-bookmarks-model.md), [Configuration](editor-configuration-model.md)

## Configuration contract

| Key | Type | Default | Owner and effect |
| --- | --- | --- | --- |
| `gutter.nerd_fonts` | Boolean | `false` | Immutable startup settings; selects one of two compiled bookmark symbols. |
| `bookmarks.markers` | Boolean, unchanged | `true` | Startup default for existing active-view marker visibility. |
| `palette.gutter.bookmark` | Existing six-digit RGB value | `#7aa2f7` | Immutable renderer palette; explicit user overrides retain precedence. |

`false` maps to the solid square `■` (U+25A0); `true` maps to the flat bookmark
`` (U+F02E, Nerd Fonts `nf-fa-bookmark`). Each occupies one terminal cell under
the editor's width policy. This mapping is a fixed Rust-owned choice, not a
new command language, font registry or configurable character string.

The boolean merges as one leaf using the existing include/root precedence and
source-origin rules. A non-boolean value rejects the complete candidate through
the existing loader contract. File changes take effect at the next startup.
The flag does not install, embed or change terminal fonts. Detection is absent.

## Legacy configuration input

`bookmarks.marker` is no longer an effective setting. The parser continues to
accept its legacy string shape so old generated configurations do not fail.
The value is discarded before effective settings publication; it has no glyph
selection, palette or layout effect. This compatibility input is not advertised
as a configurable value, is not emitted by new samples, and does not create a
second glyph-precedence rule. Normal validation of unrelated leaves remains.

Existing bookmark palette values remain effective explicit overrides. The
upgrade does not rewrite user TOML or silently replace a selected color.

## View and source ownership

The active-view marker-visibility override and split inheritance keep their
existing owner. The font-support boolean remains a startup preference and adds
no runtime override, timer or persistence operation. Numbering changes and
`bookmark-markers-mode` retain their existing scopes.

The checked gutter geometry allocates one outside marker cell when admitted.
The number field, separator and source text retain independent areas. There is
no allocated folding area in this delivery. Rendering consumes the existing
revision-qualified bookmark-row index and the fixed selected symbol.

No bookmark name, identity, anchor, location evidence, saved record, format
version or data-file path changes. The existing bookmark storage model remains
authoritative and needs no data migration.

## Future marker contracts

Enabled, disabled and pending-verification breakpoints are respectively `●`
(U+25CF) in red, `○` (U+25CB) in gray and `◌` (U+25CC) in amber. These standard
Unicode symbols do not depend on the Nerd Fonts boolean. This document adds
no breakpoint record, debugger process or breakpoint configuration leaf.

The future folding column is present only when its eventual enabling setting
requests it. Disabled means zero column width and zero folding-specific
padding. No dormant folding leaf or empty column is added by this delivery.
