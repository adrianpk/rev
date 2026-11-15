# Pair Preferences and Native State

Status: Partially approved
Editing status: Completed
Editing completed: 2026-11-15
Editing approval: product owner, 2026-11-15; rev-delimiter-editing
Navigation status: Completed
Navigation completed: 2026-11-15
Navigation approval: product owner, 2026-11-15; rev-delimiter-navigation
Presentation status: Draft
Date: 2026-11-08
Updated: 2026-11-15
Specification: [Delimiter Pairs](editor-delimiter-pairs.md)
Related specifications: [Delimiter Navigation](editor-delimiter-navigation.md), [Delimiter Presentation](editor-delimiter-presentation.md)
Inspection baseline: dd1b569dd9c6e480ae4dd82a1f298bcc8478072f
Review scope: one shared model; separate editing, navigation and visual policies

## Configuration proposal

Use the existing startup settings loader, origins, validation and language
merge owners. No new state file, settings directory or startup discovery is
introduced. Missing leaves inherit compiled defaults; malformed candidates use
the existing whole-candidate diagnostic/fallback policy. Runtime commands never
save settings. The editing implementation accepts the five approved leaves below;
future presentation leaves remain proposals.

The first delivery approves only enabled, wrap_selection, delete_empty,
skip_closing and quotes, all defaulting true. The remaining leaves below are
future presentation proposals; do not accept or install them in editing delivery.

Combined editing and future presentation schema:

```toml
[pairs]
enabled = true
wrap_selection = true
delete_empty = true
skip_closing = true
quotes = true
highlight = "depth"
pulse_ms = 250
```

| Leaf | Type and proposed meaning | Scope |
| --- | --- | --- |
| enabled | Boolean; automatic typed pair editing, approved true. | Initial buffer policy. |
| wrap_selection | Boolean; automatic ordinary linear-region wrapping, approved true. | Buffer editing policy. |
| delete_empty | Boolean; adjacent empty-pair Backspace, approved true. | Buffer editing policy. |
| skip_closing | Boolean; skip only an admitted existing closer, approved true. | Buffer editing policy. |
| quotes | Boolean; automatic quote editing consults the language profile, approved true. | Buffer editing policy; not a parser switch. |
| highlight | depth, match, pulse or off; depth is the settled default. | Initial view policy. |
| pulse_ms | Integer 50..1000, proposed 250 ms. | Finite view presentation lifetime. |

pairs.enabled=false does not disable explicit navigation or pair presentation.
pairs.quotes=false does not change source syntax or invalidate an existing
string's classification. Explicit pair-wrap/unwrap retain their own eligibility
and read-only checks; automatic editing need not be enabled to invoke them.

### Per-language inheritance and origins

Extend the existing singular language table, not a parallel lookup mechanism:

```toml
[language.rust.pairs]
enabled = false
quotes = false
```

The editing nested group admits only enabled, wrap_selection, delete_empty,
skip_closing and quotes. Each leaf is optional in the parsed document. Omitted
siblings inherit resolved global pairs leaves. The language identity remains
available when syntax color is disabled or a grammar is absent. Unknown leaves,
wrong types and malformed related values reject through existing validation.
Retain origin paths such as language/rust/pairs/enabled as distinct components.

At the inspected baseline, Language admitted indentation and grammar/asset
fields only. Editing delivery extends the native Document, merge, validation,
origins and effective resolution with this nested group. Global and language sources must pass their normal
independent and merged validation before publication. Runtime buffer override wins over the
resolved base enabled value until the buffer closes; language changes recompute
the base/profile without discarding that explicit override.

### Visual roles and palette inheritance

The proposed explicit visual override leaves retain their existing draft names:

```toml
[palette.pairs]
match_background = "#283457"
match_text = "#c0caf5"
mismatch = "#e0af68"
depth_colors = ["#7aa2f7", "#e0af68", "#bb9af7", "#7dcfff", "#9ece6a", "#ff9e64"]
```

These are examples of explicit overrides, not a theme independent of the active
palette. If absent, resolve match background/text from palette.selection and
palette.foreground, mismatch from palette.warning, and the depth cycle from
syntax.function, syntax.number, syntax.keyword, syntax.type, syntax.string and
syntax.punctuation after ordinary palette merging. Explicit user pair colors
win. Preserve absent leaves until that resolution; do not prematurely freeze
factory RGB values over user palette overrides.

Colors use existing opaque #RRGGBB parsing; depth_colors has 1..8 entries.
Depth repeats modulo the effective cycle without adding glyphs or indentation.
Only admitted structural delimiters receive depth foreground. In depth style,
the active-pair mark retains that foreground and adds background/underline.
match_text applies to marked structural delimiters in match/pulse; a marked
quote retains its string foreground. The active-pair mark otherwise uses the
proposed underline/background policy. Selection/search and cursor/
diagnostic ownership remain authoritative as defined in the presentation spec.
NO_COLOR uses supported attributes without manufacturing punctuation or colors.

## Qualified source and operation model

| Value | Qualified fields and lifetime |
| --- | --- |
| Pair profile | Recognized language, syntax/lexical policy version and admitted delimiter/quote/context rules. |
| Buffer editing policy | Resolved global/language base plus explicit buffer-local automatic-editing override. |
| Typed pair result | Buffer id/revision, opening and closing source spans/roles, delimiter kind, depth when proved, classification provenance and confirmed/unmatched/mismatched/uncertain/limited outcome. |
| Group or expression motion result | Buffer id/revision, motion unit, mode policy, initial point, signed count, qualified target/boundary or refusal; not proof of a typed mate. |
| Match request | Captured source/view/point identity, operation and entry policy, one shared finite scan/work budget and cancellation generation. |
| View presentation | Style, last non-off style, qualified point/viewport and visible source spans with independent overlay authority. |
| Pulse | One generation, qualified spans and monotonic expiry; at most one pending mark per view. |

Coordinates are native source units. Terminal-cell spans are derived by the
existing projection, not stored as editable source positions. Quotes can have
identical visible endpoints while retaining opening/closing roles. No raw tree
nodes or pointers survive a syntax revision. A mode grouping result can coexist
with a typed mismatch; never use that grouping result to claim a valid typed
pair for edits or colors. Unknown nesting depth cannot be replaced by guessed
zero merely to fill a viewport.

Classification provenance distinguishes current native syntax from a declared
bounded lexical fallback. A bare range from an old revision has no publication
authority. A new revision can reuse a retained result only when verified source
mapping proves both its transformed spans and lexical/nesting conditions; the
qualification is then advanced explicitly. Otherwise retire the unproved result
and expose the appropriate uncertain/limited state.

## Edit and literal-input ownership

The pair owner prepares one source transaction with source/read-only/region
preconditions, admitted policy, complete edits and point/mark mapping. Ordinary
edit/history admission creates its undo unit. Both ends of insertion/wrapping/
unwrap are one operation. Overtyping a confirmed closer is navigation without
source history. Counted input consumes the existing validated count once and
cannot reset its request budget for each generated mate.

Approved automatic wrapping deactivates the region and places point after the
outer opener for opener input, or after the outer closer for closer input.
Undo restores the original source and selection orientation through native
history. Rectangular/block input retains its existing owner. Refusal publishes
neither an incomplete source change nor a fabricated point/mark update.

Distinguish physical typed input, paste, completion, AI/reload/programmatic edits
and accepted macro literal steps. Generated mates belong to the original source
effect, never a new synthetic key event. Replay of raw typed steps can apply the
effective typing policy once; replay of accepted literal edits remains literal.
Preserve the macro library's existing identity, counters and history contracts.
Pairing owns no kill ring, prompt text, terminal stream or provider output.

## Independent runtime and presentation lifetime

Editing policy is buffer-local and shared by its views. Visual overrides, active
pair and pulse are view-local. Toggling either does not save configuration or
modify source/history. Navigation remains available regardless of either toggle.
Reload/close/source replacement retires stale matching and overlays. Resizing
reprojects qualified spans and preserves an existing pulse's expiry; it cannot
restart the duration. A view/source/point change cancels or replaces its mark.
After expiry/off/cancellation no pair-owned deadline remains. Depth/match do not
introduce polling or permanent background work.

## Bounded retention by delivery scope

- Fallback source inspection: at most 256 KiB per request; depth at most 256.
- Presentation: at most 8,192 eligible delimiter spans per view.
- Retained qualified pairing metadata: at most 2 MiB per buffer.
- Cooperative slice: at most 256 tokens; reuse the four-millisecond native input scheduling target.
- Pulse: one finite expiry per view, within the existing admitted view count.

Shared/cache allocations count once under an explicit owner. Byte limits also
bound oversized lexical tokens; counts do not multiply admission into unbounded
work. Exhaustion returns limited/uncertain rather than a guessed mate, guessed
depth or partial edit. Editing byte/depth/retention/cooperative bounds are approved
and the editing plan records the measured current primitive baseline and resource
sketch. Verify actual new-service and integrated workloads in Slices 2 and 7.
Presentation span/pulse bounds and held-navigation acceptance remain proposals
for their separate deliveries. No new permanent worker/service is introduced.

## Editing delivery boundary

[rev-delimiter-editing](../plan/rev-delimiter-editing.md) owns only the approved
five editing settings, qualified matching/context and native editing/provenance
state. For that editing delivery, navigation and visual records supplied design
context rather than implementation authority. Editing closure set Editing status
to Completed and retained this model's Partially approved status.

Implementation evidence is collected in the
[seven-slice tracker](../tracker/rev-delimiter-editing.md) and
[acceptance report](../report/slices/rev-delimiter-editing/slice-7-acceptance.md).
Editing is Completed after the controller's unchanged combined gate passed at
`eb65b227dee8b42bf8d0c8a6e0ef12f5675c756b`, receipt
`QD5ALW2WJR64VCCTK4CJAOZPDK`. The
[closure record](../report/slices/rev-delimiter-editing/slice-7-acceptance.md#delivery-set-closure)
preserves that tested identity separately from the subsequent Markdown-only dev
closure commit. At editing closure this model stayed Partially approved, with navigation and
presentation Draft. The subsequent navigation authority is recorded below;
presentation retains its separate Draft boundary.

## Navigation delivery boundary

[rev-delimiter-navigation](../plan/rev-delimiter-navigation.md) approves the shared
source-qualified motion request/result, seven command identities and six native
base-editor bindings in the [navigation specification](editor-delimiter-navigation.md).
No new startup leaves, persistence format, pair palette or presentation fields
are introduced. Automatic editing preferences are not navigation admission gates;
readonly source remains navigable through native focus and coordinate owners.

Typed Confirmed pair facts remain distinct from group/expression targets and
ordinary boundaries/refusals. The existing per-buffer retained metadata and
aggregate inspected-byte/depth/cooperative bounds apply across counted motion;
source/view/point/mark/focus changes retire pending publication. A mode group
cannot become typed pair evidence through cache reuse. Use the native mark and
source-motion/jump owners; do not add an independent history or per-view cache.

The five navigation slices are integrated through PRs #62-#66.
[Slice 5 acceptance](../report/slices/rev-delimiter-navigation/slice-5-acceptance.md)
delivers nonempty native batches, independent complete factory expectations and
two owned Linux PTY journeys. The controller's unchanged combined gate passed at
`27957128f7626ee16b5dbdc50af52e9cee27f3c8`, receipt
`YEU4CDCDB3RP65D5PG67RRYTYF`; its
[closure record](../report/slices/rev-delimiter-navigation/slice-5-acceptance.md#delivery-set-closure)
distinguishes that tested candidate from the later Markdown-only closure.

Navigation is Completed. Editing remains
Completed, Presentation remains Draft and this model remains Partially approved.
EIP04 closes only its expression portion; paragraph and broad parity stay pending.
