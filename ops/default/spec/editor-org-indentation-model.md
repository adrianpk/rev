# Org Visual Indentation Model

Status: Completed
Completed: 2026-11-05
Date: 2026-11-05
Approved: product owner, 2026-11-05; paired specification approval and explicit Looper startup request
Specification: [Org Visual Indentation](editor-org-indentation.md)
Existing owners: [Heading Presentation Model](editor-org-heading-presentation-model.md), [Soft Wrap Model](editor-soft-wrap-model.md), [Configuration Model](editor-configuration-model.md), [Org Mode Model](editor-org-mode-model.md)

## State ownership

| Record | Owner and contract |
| --- | --- |
| `org.indent` | Existing immutable startup snapshot; boolean, default `true`. Omitted values inherit; explicit values follow existing include/override provenance. Invalid types reject the whole candidate through existing handling. |
| Runtime override | One `Option<bool>` equivalent per live buffer; `None` inherits startup. Toggle and explicit enable/disable record a value. It is not persisted or added to edit history. |
| Effective policy | Existing buffer Org enablement and the startup/override value. A non-Org buffer does not project indentation. Disabling Org suspends this policy without deleting the override. |
| Structural line observation | Native current buffer/revision and enclosing heading level, with literal-block ownership respected. Preamble level is zero. No second Org parser or independently maintained outline. |
| Projected fragment | Checked source span, structural margin, concealed prefix, continuation affinity and source/cell mapping at one text width. Virtual cells have no independent source scalar. |
| Layout identity | Existing buffer/revision, width, tabs, folds and observations extended by effective indentation and view-local pretty/hiding policies. Level changes and presentation toggles retire affected derived observations. |

The new setting belongs directly to `[org]`, independently of `[org.headings]`:

```toml
[org]
indent = true
```

Startup admission remains startup-only. Commands do not edit configuration;
closing and reopening a buffer uses the admitted startup value. No cache file,
migration or additional configuration directory is introduced.

## Visual margins and mapping

Use terminal-cell columns relative to the pane's text area after the existing
gutter. For level N >= 1, ideal additional heading margin is N-1 and ideal body
margin is 2N. In pretty view with leading stars hidden, the existing N-cell
source prefix plus separator places title and body at 2N. Pretty presentation
off retains all source stars and still adds the heading margin. Source spaces
and tabs keep their existing source identity and tab-stop policy; virtual cells
do not change the source column used to expand a tab.

Heading continuations use the ideal title margin 2N. Body continuations use the
same structural margin as their first row. Existing source whitespace remains
literal content; this does not introduce new list hanging-indentation rules.
Tables and code retain their relative source columns under one outer margin.

With indentation disabled and pretty/hide-leading-stars enabled, remove the
first N-1 prefix cells from display rather than draw blanks. Every marker starts
at column zero and its title at column two. Source/visible-star presentation
keeps the literal prefix width. Hidden stars share the marker's cursor cell;
a marker hit targets the final star, while keyboard operations and copy retain
every source star. The first fragment still owns the complete prefix source
span, including when budgeted layout spans several passes. Wrapping, scrolling,
hits and overlays use this same translation.

Pretty/source changes retire view layout observations. Pending-to-ready Org
recognition also invalidates flat-prefix observations. Skip hidden stars under
the existing glyph budget without retaining one pending glyph per concealed
cell. This is the owner's post-delivery clarification; historical gate evidence
below still belongs to its original candidate.

For text width W, cap any virtual margin at max(0, W-2), using checked arithmetic
or saturation before multiplication/allocation. This leaves room for a normal
two-cell grapheme when W >= 2. At width 1, use zero virtual margin and existing
oversized-grapheme behavior. The established suspended-pane policy still owns
unusable geometry. Capping is a display adjustment, not a change to level N.
Do not emit endless virtual-only continuations at an over-deep heading.

Each real source scalar keeps its canonical position. Prefix source stars
retain separate mappings after the added margin. A hit inside virtual padding
maps to the first source position of that visual fragment; selection and copy
do not invent spaces. Source-backed selection paint begins at real text cells.
Rendered offsets are not accepted as byte or scalar offsets.

## Lifetime and bounded work

All views of the same buffer observe one effective indentation choice; their
widths, folds and pretty-heading overrides remain view-owned. A view rebound to
another buffer adopts that buffer's indentation policy. The three commands are
available in editable and read-only Org views, never a text edit. New default
bindings and a fourth reset command are unnecessary.

Reuse current bounded Org observation and view-layout caches. The existing
8192-fragment and 1 MiB cached-text limits per view remain upper bounds; account added
margin text/mapping in those bounds rather than retaining an uncharged cache.
Do not construct an N-level padding string before clamping it to W. No extra
idle deadline, worker, external process or dependency is required.

Invalidate or reobserve on source revision, structural coverage, fold, geometry
or effective-policy changes. Render literal rows coherently while structural
evidence is pending. Undo, reload and mode suspension preserve runtime policy;
buffer close retires it. Reports must record actual native/resource/PTY evidence
and distinguish it from the specification's required behavior.

## Completed delivery evidence

The paired specification and model are Completed through canonical PRs
#276-#279. Controller receipt `YE523W7IGTDKXEUNGCMA3NNJBY` passed the unchanged
full gate at `61e6e6a1b73d94577c7851aae6f200f6101d35c7`. The
[Slice 4 closure](../report/slices/rev-org-indentation/slice-4-acceptance.md#delivery-set-closure)
records source-only copy/save and edit history, buffer-owned runtime policy,
checked projection, native/PTY lifecycle and resource acceptance. Eight native
cache/work cases include eight runtime policy changes at widths 6 and 512,
with actual retention peaks of 8192 fragments and 1048576 bytes under the
unchanged limits. Deep and over-limit outlines retain finite source progress,
and the resource cases introduce no idle deadline.
