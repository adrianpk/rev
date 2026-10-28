# Editor Configuration Model

Status: Completed
Completed: 2026-10-27
Validated candidate: `95713efb7f79fbc72417615a921841f8d49730ed`
Gate validation: `FWDUCVRJCYBKFXEWEFLKPKA35E`
Approved: product owner, 2026-10-27; unified configuration and interaction delivery authorized.
Date: 2026-10-27
Specification: [Editor Configuration](editor-configuration.md)
Editor model: [rev Editor Model](rev-editor-model.md)
Implementation boundary: rev-configuration-ui, after completed rev-editor delivery
Plan: [Configuration and Interaction Delivery](../plan/rev-configuration-ui.md)

## Purpose

Describe persistent TOML shapes, source layering, typed effective settings,
scopes, and the reference catalog required by the paired proposal. These are
semantic records, not a mandate for public Rust APIs, a crate per table, or a
general extension/configuration framework.

The validated candidate and gate above retain the historical configuration/UI
result. The [input/navigation delivery](../plan/rev-input-navigation.md) adds the
implemented gutter/mouse leaves and packaged first-start publication described
here; its own integrated gate remains pending.

The option inventory is deliberately broader than the selected delivery.
Configuration targets existing editor capabilities and the agreed indentation
and keybinding policies. The separately captured panel, vertical command-list,
and file-completion work remains explicit ticket scope. Missing capabilities in
the inventory are potential future expansions, not implementation tasks for
this delivery. A commented example still documents a supported option; comments
do not turn an unimplemented option into a feature.

## Storage ownership

| Location | Owned content | Boundary |
| --- | --- | --- |
| User XDG config directory | `config.toml` and explicitly included TOML files. | Startup may publish the complete packaged template only when no entry exists; loading never rewrites existing settings. |
| User XDG data directory | Durable user resources when a feature provides them. | No addon or resource installation is introduced here. |
| User XDG state directory | Operational state when a feature provides it. | No session/history persistence schema is introduced here. |
| User XDG cache directory | Disposable derived data when a feature provides it. | Removing cache cannot remove settings or document text. |
| Program image | Complete typed defaults, fixed commands, supported palette roles and the complete bootstrap template. | Startup does not depend on a sample/default file on disk. |
| Distributed sample | Reference assignments and commented supported alternatives. | Documentation artifact; not an obligatory runtime input. |

The XDG roots are resolved from their respective environment variables before
reading settings. They are not recursively determined by the configuration
file whose location is being searched. Relative XDG values are ignored.

No automatic project discovery, database, executable settings, or on-disk
session migration belongs to this format.

Normal application startup may create only absent configuration parents and
publish a complete no-clobber `config.toml` before ordinary loading. Any existing
entry, including invalid/empty files, links and directories, is preserved.
Failure falls back to usable built-ins with a diagnostic. Pure path resolution,
explicit loading and snapshot admission remain read-only. No data/state/cache
root is created merely to start. CFG01, CFG14, CFG15, CFG23 and CFG24 own this
exception; it creates no new settings schema or persisted session state.

## Source records

| Record | Fields in semantic order | Contract |
| --- | --- | --- |
| Source | Source identity, resolved path, declaring parent, declaration order. | Identifies one explicit file; symlink aliases cannot hide include cycles. |
| Partial document | Optional schema version, ordered includes, present concern tables, present language tables, present keymap tables. | Absence survives parsing until resolution. |
| Assignment | Source, key path, typed raw value, available line/column span. | Supports precise errors and per-leaf provenance. |
| Include edge | Declaring source, path literal, resolved target, list position. | Relative paths use the declaring file's parent. |
| Candidate | Loaded source graph, partial layers, resolved typed settings, validated command maps. | Has no authority to mutate live text during preparation. |
| Effective settings | Global values, language descriptors/overrides, context maps, palette, source generation. | Published only after complete validation. |
| Diagnostic | Source/key/span, failed rule, concise message. | No unrelated environment dump or silent user-file repair. |

The root metadata is `version = 1` and optional `include = [...]`.
Omitting `version` implies 1. TOML 1.1.0 is the accepted input format.
Unrecognized versions must not be interpreted as if their schema were known.

All user tables are partial. A missing integer is not zero; a missing boolean
is not false. In particular, deserializing a language table into a fully
defaulted global settings structure would lose the intended inheritance.

## Resolution model

The accepted precedence has two distinct dimensions: source order and scope.

1. Construct complete built-in global settings and built-in bindings.
2. Resolve explicit includes in declared order, including their own validated
   include order, under finite graph/input limits.
3. Overlay the root file's own partial settings over included values.
4. Validate the merged schema, role names, bounds, and effective keymaps.
5. Resolve global values against built-ins.
6. Identify a buffer's language through the accepted deterministic file rules.
7. Overlay that language's present, permitted buffer-scoped values over the
   resolved global values.

At the same scope, a later scalar wins. Supported tables merge by leaf key.
Arrays replace completely, including an allowed explicit empty array. The
include list itself orders source discovery; it is not a setting to concatenate
blindly across layers. Native duplicate-key errors within one TOML source
remain errors.

Language scope is more specific than global scope after source resolution.
Moving a global setting into a later file does not make it defeat an explicit
matching language setting. Global-only fields in a language table are errors.

Each effective value may retain its origin for diagnostics. An inspection
command is not mandated by retaining that information. No second tracker or
configuration database is needed.

## Catalog contract

Every setting selected for implementation needs the following comparable
catalog fields:

| Field | Meaning |
| --- | --- |
| Key | Stable TOML table/key path. |
| Owner | The component that applies its effect. |
| Type | Boolean, integer, float, enum string, color, array, or checked map. |
| Default | The exact complete built-in value. |
| Bounds and units | Accepted numeric range, enum members, and any sentinel semantics. |
| Scope | Global, buffer/language, or input context. |
| Application | Startup, new-buffer default, or supported runtime policy replacement. |
| Capability | The actually delivered behavior that makes this key meaningful. |
| Reference | Sample comment/assignment and fuller reference text where needed. |

The runtime schema, built-in values, and exhaustive sample must be checked
against the same catalog. This does not require implementing a generic schema
language. A small typed Rust owner plus deterministic consistency checks is
sufficient. New roles and options cannot appear only in one of these surfaces.

## Confirmed configuration surfaces

| Surface | Shape | Settled behavior |
| --- | --- | --- |
| Palette | Fixed semantic role to color assignments. | Complete built-ins; present leaves override; sample includes all supported role colors actively. |
| General editing | Separate indentation style, indentation width, tab display width. | Global values with partial per-language overrides; no whitespace rewriting on load. |
| Language settings | Named descriptor plus supported partial buffer overrides. | Editing preferences do not require a parser for that language. |
| Keybindings | Context to supported chord/sequence to existing command name. | Editor/minibuffer/tree are distinct; omitted bindings inherit. |
| Optional file composition | Ordered explicit include paths. | Application-defined layering with errors/cycle detection; limits fixed by CFG08. |
| Reference sample | Active defaults and commented supported optional alternatives. | Exhaustive documentation of the supported surface, not a roadmap of unimplemented features. |

The spaces/4 and Go/2 example establishes how overrides work. It does not
select two-space Go indentation as a built-in language convention for every
installation. An optional sample override can demonstrate it without changing
the common fallback.

## Accepted configuration catalog

This table and the palette table are the complete shipped surface selected for
this delivery. Each default is compiled in. All values apply at startup; buffer
policies are resolved from that snapshot when a buffer opens or its path changes.
Only the three `editor` leaves and `grammar` may occur under `language.<id>`.
No future-inventory entry extends this table implicitly.

| Key | Type and default | Bounds / values | Owner and scope |
| --- | --- | --- | --- |
| `version` | Integer, `1` (also when absent) | Only 1 | Loader, global schema |
| `include` | String array, `[]` | Explicit paths, CFG08 limits | Loader, source-local traversal |
| `editor.indent_style` | String, `"spaces"` | `spaces`, `tabs` | Buffer editing policy; language-overridable |
| `editor.indent_width` | Integer, `4` | 1..32 display columns | Buffer editing policy; language-overridable |
| `editor.tab_width` | Integer, `4` | 1..32 display cells | Buffer coordinates; language-overridable |
| `scroll.page_overlap` | Integer, `2` | 0..1000 rows; clamp to permit one-row progress | View scrolling, global |
| `gutter.visible` | Boolean, `true` | True/false | Global startup default; view-local override |
| `gutter.mode` | String, `"absolute"` | `absolute`, `relative` | Global startup default; view-local override |
| `gutter.current_line` | String, `"absolute"` | `absolute`, `zero`, `hidden` | Global startup default; view-local override |
| `gutter.side` | String, `"left"` | `left`, `right` | Global startup default; view-local override |
| `gutter.minimum_digits` | Integer, `4` | 1..20 digits; full source count still fits | Global startup geometry |
| `gutter.left_padding` | Integer, `0` | 0..4 terminal cells | Global startup geometry |
| `gutter.right_padding` | Integer, `1` | 0..4 terminal cells | Global startup geometry |
| `mouse.enabled` | Boolean, `true` | True/false | Session startup; mouse-mode override |
| `mouse.scroll_lines` | Integer, `3` | 1..100 logical rows; drag tick also clamps to text height | Global native pointer policy |
| `windows.split_ratio` | Float, `0.5` | Finite 0.1..0.9; clamp to geometry minima | Layout, global initial split |
| `minibuffer.candidate_limit` | Integer, `8` | 1..32 rows, clipped to available geometry | Prompt completion, global |
| `minibuffer.match` | String, `"prefix-then-substring"` | `prefix-then-substring`, `prefix`, `substring` | Command candidates only; deterministic registry order within rank |
| `minibuffer.history_limit` | Integer, `128` | 1..4096 entries | Session command history, global |
| `minibuffer.input_limit` | Integer, `4096` | 1..65536 Unicode scalars | Existing prompt editor, global; excess input rejected visibly |
| `history.group_limit` | Integer, `512` | 1..65536 complete groups | Buffer history, global |
| `history.payload_target_bytes` | Integer, `67108864` | 1..1073741824 logical payload bytes; soft target | Buffer history; preserve newest coherent group |
| `kill_ring.capacity` | Integer, `32` | 1..4096 entries | Linear kill ring, global |
| `kill_ring.coalesce` | Boolean, `true` | True/false | Consecutive directional kills; never combine rectangle store |
| `tree.visible` | Boolean, `false` | True/false | Tree initial visibility; editing focus retained |
| `tree.width` | Integer, `24` | 1..512 columns, geometry may constrain it | Tree panel preference, global |
| `tree.max_fraction` | Float, `0.3333333333333333` | Finite 0.1..0.5 | Tree maximum frame share; preserve baseline small-frame hiding |
| `tree.root` | String, `"auto"` | `auto`, `cwd`, `file-parent` | Tree initial root; auto=file parent when supplied, otherwise cwd; file-parent also falls back to cwd |
| `tree.show_hidden` | Boolean, `true` | True/false | Tree listing; preserves current default including dotfiles |
| `tree.directories_first` | Boolean, `true` | True/false | Tree name sorting; deterministic filename tie-break |
| `syntax.enabled` | Boolean, `true` | True/false | Bundled highlighting, global |
| `syntax.max_bytes` | Integer, `0` | 0..1073741824 UTF-8 bytes; 0=no configured cutoff | Syntax work only; editing remains available |
| `files.newline` | String, `"lf"` | `lf`, `crlf` | New-file default only |
| `files.bom` | Boolean, `false` | True/false | New-file default only |
| `files.associations` | Pattern-to-language string map, no user overrides | Exact basenames or `*.suffix`; rules below | Buffer language identification, not grammar installation |
| `language.<id>.indent_style` | Optional string, inherit editor | Same bounds as editor leaf | Matching buffer only |
| `language.<id>.indent_width` | Optional integer, inherit editor | Same bounds as editor leaf | Matching buffer only |
| `language.<id>.tab_width` | Optional integer, inherit editor | Same bounds as editor leaf | Matching buffer only |
| `language.<id>.grammar` | Optional string | `rust`, `plain`; Rust identity defaults to rust, others plain | Existing bundled Rust parser or plain text |
| `keys.editor`, `keys.minibuffer`, `keys.tree` | Sequence-to-command string maps, no user overrides | Valid existing commands in the named input context | Fixed Rust command/action identities; complete default maps documented |
| `palette.*`, `palette.syntax.*`, `palette.gutter.*` | RGB strings, exact table below | Six-digit `#RRGGBB`; fixed role names | Semantic renderer roles, global |

New-file preferences never rewrite an existing file's BOM, separators or final
newline. They apply once at buffer creation. Runtime key actions and explicit
user edits retain normal command/history ownership. Candidate matching modes
do not change filesystem matching or search case policy.

### File associations and precedence

Built-ins recognize `*.rs` as rust; `*.go` as go; `*.rb`, `Gemfile`, `Rakefile` as
ruby; `*.c`, `*.h` as c; `*.cc`, `*.cpp`, `*.cxx`, `*.hpp` as cpp; `*.py` as python;
`*.js`, `*.mjs`, `*.cjs` as javascript; `*.ts`, `*.tsx` as typescript; `*.json` as
json; `*.toml`, `Cargo.toml` as toml; `*.md` as markdown; `*.sh` as shell; and
`Makefile` as make. Every unmatched basename is plain. These identities select
editing preferences; they do not promise syntax grammars beyond Rust.

A user association requires no matching language table: it may select an ID
whose editing leaves inherit the global settings. IDs use lowercase ASCII
letters, digits and hyphens, starting with a letter. Values are not paths.
Patterns are case-sensitive literal basenames or one leading `*.` followed by
a nonempty literal suffix; no other wildcard, slash or path separator is valid.
Compare the final basename, not its canonical target basename, so a visited
alias retains its visible filename policy.

Any matching user rule takes priority over all built-ins. Within that class,
an exact basename wins over suffix rules; otherwise the longest suffix wins.
For an identical pattern, ordinary include/root leaf precedence selects one
mapping. Equally long different suffixes cannot match the same basename; no
filesystem order or map iteration breaks ties. Thus shared extensions are
ordinary customization, not ambiguous schema errors. For example:

```toml
[files.associations]
"*.h" = "cpp"
"*.test.ts" = "test-typescript"
"special.h" = "c"
```

Here `special.h` selects c; other `.h` names select cpp. A Go override with only
`indent_width = 2` inherits all other global leaves. No built-in Go override
forces that illustrative value.

### Loader bounds and parser selection

The root has depth zero. Maximum include depth is 16 edges; at most 64 source
occurrences are admitted, including repeats and root. Each source is limited
to 256 KiB and total reads to 2 MiB. Include paths must be nonempty local paths;
relative includes resolve from their declaring source. Canonical identities and
filesystem identity detect cycles on the active traversal stack; a shared leaf
in two acyclic branches is permitted and applies in each declared position.

Version omission means 1 in every source. Unknown explicit versions reject the
whole candidate. Use pinned `toml = "=1.1.8"` (release metadata spec-1.1.0;
MSRV 1.85) and `serde = "=1.0.229"` with derive (MSRV 1.56), compatible with
this repository's Rust 1.98. Read/parse features only; no serializer or config
rewriting is required. Resolve and commit Cargo.lock in Slice 1. These are the
only new direct dependencies authorized by the TOML implementation scope.

References: [toml parser](https://docs.rs/toml/1.1.8/toml/),
[Serde](https://docs.rs/serde/1.0.229/serde/). Their licenses are MIT OR Apache-2.0;
first-party rev remains GPL-3.0-only. No Emacs source is introduced.

## Future option inventory

These controls are not accepted keys. They must be rejected as unknown if used
now, and must not appear in the operational sample, even as commented examples:
auto indentation; current-line highlighting; visible tabs/spaces,
trailing spaces or separators; cursor shapes/blinking; scroll margins or reveal
steps; separator toggles; alternate window focus order; wrapping/all-match
search; time-based typing grouping; natural tree sorting; additional grammars;
project-local configuration; reload/watch; addons; session persistence.

Explicit [tabify/untabify](editor-whitespace-conversion.md) commands operate on
the admitted marked region. The Tab insertion policy and configuration loader
never perform those conversions implicitly. Gutter and mouse leaves remain
global/session or view preferences; `language.<id>` retains only its existing
three editing leaves and grammar selector.

## Palette reference and defaults

The requested Tokyo Dark Night reference is interpreted as Tokyo Night's
dark/night variant. The product owner accepted comparing its VS Code and
Neovim implementations and selecting practical defaults from those sources.

The reproducible references are:

- [VS Code Tokyo Night](https://github.com/tokyo-night/tokyo-night-vscode-theme/blob/7c0f11eaef322f293621ca7befe462214b7ea468/themes/tokyo-night-color-theme.json),
  revision `7c0f11eaef322f293621ca7befe462214b7ea468`.
- [Neovim resolved night palette](https://github.com/folke/tokyonight.nvim/blob/cdc07ac78467a233fd62c493de29a17e0cf2b2b6/extras/lua/tokyonight_night.lua),
  revision `cdc07ac78467a233fd62c493de29a17e0cf2b2b6`.

Both use `#1a1b26` for the editor background and `#16161e` for secondary
surfaces. Where they differ, use the resolved Neovim night colors. In
particular, body text uses `#c0caf5` and comments use `#565f89`, rather than
the darker VS Code body/comment values. Pane separators use Neovim's gutter
color for visible boundaries; focused borders use its blue accent. These role
assignments are rev's defaults, not a claim that the ports map every UI role
identically.

| TOML role | RGB default | Use |
| --- | --- | --- |
| `palette.background` | `#1a1b26` | Document background. |
| `palette.foreground` | `#c0caf5` | Ordinary document text. |
| `palette.cursor` | `#c0caf5` | Cursor color where supported by the terminal renderer. |
| `palette.selection` | `#283457` | Linear and rectangular selection background. |
| `palette.selection_text` | `#c0caf5` | Selected document text. |
| `palette.search` | `#3d59a1` | Current search match background. |
| `palette.search_text` | `#c0caf5` | Current search match text. |
| `palette.status` | `#16161e` | Active window status background. |
| `palette.status_text` | `#a9b1d6` | Status text. |
| `palette.status_inactive` | `#1a1b26` | Inactive window status background. |
| `palette.border` | `#3b4261` | Pane separators. |
| `palette.border_active` | `#7aa2f7` | Focused border accent. |
| `palette.tree` | `#16161e` | File-tree background. |
| `palette.tree_selected` | `#283457` | Selected tree row background. |
| `palette.tree_selected_text` | `#c0caf5` | Selected tree row text. |
| `palette.message` | `#c0caf5` | Prompt and ordinary message text. |
| `palette.warning` | `#e0af68` | Warning text. |
| `palette.error` | `#db4b4b` | Error text. |
| `palette.gutter.background` | `#1a1b26` | Number-field background. |
| `palette.gutter.foreground` | `#565f89` | Ordinary logical labels. |
| `palette.gutter.current_line` | `#7aa2f7` | Current-line label with non-color emphasis. |
| `palette.gutter.separator` | `#3b4261` | Number/text separator. |
| `palette.syntax.keyword` | `#9d7cd8` | Keyword captures. |
| `palette.syntax.string` | `#9ece6a` | Strings and the existing escape group. |
| `palette.syntax.comment` | `#565f89` | Comments. |
| `palette.syntax.type` | `#2ac3de` | Types and constructors. |
| `palette.syntax.function` | `#7aa2f7` | Functions. |
| `palette.syntax.number` | `#ff9e64` | The existing constant/number group. |
| `palette.syntax.punctuation` | `#89ddff` | The existing punctuation/operator group. |

The seven syntax roles match the current style owner. Splitting operator,
constant, or escape captures into additional configurable groups is not
required by this palette decision. The ordinary tree text inherits the
foreground role. Selection colors must remain readable over syntax styles.

The schema uses a `palette` table plus `palette.syntax` and `palette.gutter` subtables. A missing
configured role inherits its built-in color; an unknown role is an error.
Colors use `"#RRGGBB"`; terminal-default/transparent sentinels are not selected.
Gutter roles accompany the accepted numbering capability. Current-line text
highlighting and other future-inventory controls are not enabled by these roles.

## Keymap representation

Each input context owns its built-in map and optional user overrides.
Configured sequence strings are normalized through the same chord vocabulary
used by runtime input. Values resolve to existing command identities before
activation. They are never executable strings.

The complete effective map must detect an ambiguous command at a prefix that
also has descendants. Replacing the same inherited chord is valid layering;
two conflicting entries in a single TOML table remain invalid. An unresolved
command name cannot be advertised as a functioning binding.

Commenting out a custom mapping restores the underlying mapping. It does not
remove it. Values must name existing commands; this delivery has no explicit
unbinding, disable, or removal action. Invalid non-command values are rejected.
Cancellation must remain reachable in transient contexts.

## Application to existing editor state

| Existing owner | Permitted preference effect | Preserved invariant |
| --- | --- | --- |
| Buffer | Effective editing policy and recognized language preference. | Rope bytes and saved-state identity do not change by loading settings. |
| View | Reprojection, margins, cursor and presentation preferences. | Point remains a valid text position; views retain independent state. |
| Mark/region | Recompute derived display geometry as needed. | Text endpoints remain qualified to the current buffer. |
| Rectangle | Apply the source policy during extraction and the destination policy to yank columns. | Emacs-style copy/kill tab semantics; no grapheme split or accidental source mutation. |
| Syntax | Enable/disable or select a supported bundled mode; invalidate stale styling. | Canonical text/history survive parser changes and errors. |
| History | Apply an admitted whole-unit retention policy. | Immediate coherent reversibility and truthful saved state. |
| Ring | Admit capacity/coalescing changes under a defined policy. | Linear and rectangle stores remain separate. |
| Tree/layout | Recompute optional presentation and initial/root preferences. | No implicit file deletion/rename or loss of visiting buffers. |
| File association | Re-evaluate language after a successful path change; use format defaults only for new files. | Existing UTF-8/BOM/newline fidelity and publication truthfulness. |

Configuration sources are read at startup only. Editing them does not change
the running session. Manual reload and automatic change watching remain
potential future expansions. Newly opened buffers use the startup settings.
A successful rename or reassociation re-evaluates each affected buffer's
language overrides from that same startup snapshot, including visited children
of a renamed directory. Failed or canceled path changes preserve the policy.
Views of the same buffer cannot use incompatible tab/display policies; existing
bytes, history and saved-state identity are not rewritten by re-evaluation.

Tab insertion follows the next applicable display-column stop: tab indentation
stores one U+0009 character and displays it through the next `tab_width` stop;
space indentation inserts spaces through the next `indent_width` stop.
Existing tabs, spaces and pasted text are not converted by configuration.
Explicit tabs-to-spaces and spaces-to-tabs commands are a separate future task.

### Rectangle transfer reference

Tab/column transfer follows GNU Emacs 31.1 behavior observed in isolated batch
buffers. Copy expands selected tabs into the retained strings without changing
the source. Kill preserves complete interior tabs; partial tab edges are
materialized according to the source columns. Yank inserts the retained strings
at one left display column per destination row. Any retained tab then uses the
destination's stops; the right edge is not forced to the original source width.

For source rows `a\tb` and `c\td` at tab width 4, copy retains `a   b` and
`c   d`. Kill retains `a\tb` and `c\td`. In a destination with tab width 8,
both forms align their row starts, but retained tabs can change internal width.
Rev's existing grapheme, short-line and transaction invariants still apply.

## Illustrative TOML excerpt

This excerpt demonstrates shape and inheritance and includes every selected
palette default. It is not the exhaustive distributed sample for the remaining
configuration catalog, whose accepted catalog is defined above.

```toml
version = 1

# Optional composition; paths are relative to this file.
# include = ["palette.toml", "languages/go.toml"]

[editor]
indent_style = "spaces"
indent_width = 4
tab_width = 4

# Optional language override; missing values inherit from editor settings.
# [language.go]
# indent_width = 2
# tab_width = 2

[palette]
background = "#1a1b26"
foreground = "#c0caf5"
cursor = "#c0caf5"
selection = "#283457"
selection_text = "#c0caf5"
search = "#3d59a1"
search_text = "#c0caf5"
status = "#16161e"
status_text = "#a9b1d6"
status_inactive = "#1a1b26"
border = "#3b4261"
border_active = "#7aa2f7"
tree = "#16161e"
tree_selected = "#283457"
tree_selected_text = "#c0caf5"
message = "#c0caf5"
warning = "#e0af68"
error = "#db4b4b"

[palette.syntax]
keyword = "#9d7cd8"
string = "#9ece6a"
comment = "#565f89"
type = "#2ac3de"
function = "#7aa2f7"
number = "#ff9e64"
punctuation = "#89ddff"

[keys.editor]
"C-x C-s" = "save-buffer"
# "M-j" = "next-line"
```

The final sample must include every accepted option and all palette role
assignments, with comments/reference metadata sufficient to explain defaults,
allowed values, units, scopes, and optional usage. Its defaults must agree with
compiled defaults for that same release.

## Runtime presentation and pointer state

The [gutter model](editor-gutter-model.md) defines checked per-view policy,
explicit overrides, inherited split state and disposable visible number rows.
The immutable startup snapshot is the reset source. Runtime commands change
only the active view and never persist settings. Mouse enable/disable is one
session override; captured source view/anchor, latest edge position and at most
one 100-ms deadline remain ephemeral native state. Disabling, cancellation,
resize or source/view loss retires capture. Released/inside/disabled idle has
no pointer deadline or periodic work. Neither control adds a storage owner.

## Validation evidence

- Absent versus explicit false/zero/empty partial values.
- Same-scope leaf overrides, table merging, array replacement, and root/include order.
- Language scope overriding global scope without resetting unmentioned siblings.
- Include aliases/cycles, missing explicit includes, and finite input/work limits.
- Unknown options/roles, invalid bounds/types/colors, and source-aware diagnostics.
- Effective binding replacement, aliases, prefix conflicts, and context isolation.
- Exhaustive sample/catalog/default equality and complete default command map.
- Missing/invalid config startup preserving a usable state; file edits taking
  effect only after a new startup.
- Palette/tab/keybinding behavior through the real terminal entry points.
- Unchanged text bytes/history/dirty state when preferences only change presentation.
- CRLF/BOM/final-newline fidelity and truthful save outcomes after configuration changes.

The original configuration/UI acceptance obligations passed through the controller-owned full gate
on the validated candidate above. The final Slice 30 report records the native,
terminal, performance and catalog evidence. The original 48-slice delivery
remains closed with its own historical evidence.
