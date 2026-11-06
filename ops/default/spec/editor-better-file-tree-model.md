# Better File Tree Model

Status: Completed
Completed: 2026-11-06
Validated candidate: `e0351ac918a760b7940e660494f2309d67bd30a4`
Gate validation: `2WXLCTXPDKGL7ZMY7KI52IQSFP`
Date: 2026-11-06
Approved: product owner, 2026-11-06; specification/model and one autonomous Looper delivery through dev
Specification: [Better File Tree](editor-better-file-tree.md)
Plan: [Better File Tree Delivery](../plan/rev-better-file-tree.md)
Tracker: [Better File Tree Tracker](../tracker/rev-better-file-tree.md)
Baseline models: [Configuration](editor-configuration-model.md), [File and Directory Navigation](editor-file-navigation-model.md), [Better Gutter](editor-better-gutter-model.md)

## Configuration contract

| Key | Type | Compiled default | Effect |
| --- | --- | --- | --- |
| `tree.decorated` | Boolean, new | `true` | Select decorated or existing minimal tree labels. |
| `gutter.nerd_fonts` | Existing boolean | `false`, unchanged | Select the compiled Nerd Fonts catalog for decorated tree entries, as well as its existing gutter use. |
| `palette.tree_entries.root` | Six-digit RGB string, new | `#7aa2f7` | Root icon and label. |
| `palette.tree_entries.directory` | Six-digit RGB string, new | `#e0af68` | Directory icon and label. |
| `palette.tree_entries.source` | Six-digit RGB string, new | `#7aa2f7` | Source-file icon and label. |
| `palette.tree_entries.document` | Six-digit RGB string, new | `#e0af68` | Document icon and label. |
| `palette.tree_entries.configuration` | Six-digit RGB string, new | `#bb9af7` | Configuration-file icon and label. |
| `palette.tree_entries.file` | Six-digit RGB string, new | `#c0caf5` | Unclassified-file icon and label. |
| `palette.tree_entries.symlink` | Six-digit RGB string, new | `#7dcfff` | Symlink icon and label. |

Each new leaf participates in the existing include/root precedence and
source-origin records. Explicit values override compiled defaults. A wrong
boolean type or malformed palette value rejects the complete candidate through
the existing configuration loader; ordinary all-default fallback remains owned
by that loader. Changes become effective at the next startup.

`palette.tree` remains an RGB scalar for the pane background. The new sibling
table does not turn it into a table. Existing `palette.tree_selected` and
`palette.tree_selected_text` retain selection ownership and precedence over
entry foreground roles. Minimal mode ignores the new entry colors and kind
icons, but does not change the font preference's gutter effect.

## Effective presentation

| Decorated | Nerd Fonts | Effective tree presentation |
| --- | --- | --- |
| `false` | `false` | Existing minimal labels. |
| `false` | `true` | Existing minimal labels. |
| `true` | `false` | Ordinary-symbol decorated labels. |
| `true` | `true` | Nerd Fonts decorated labels. |

Symbols come from fixed compiled catalogs defined by the specification.
Configuration stores no code point, icon name, arbitrary glyph, font path or
font-detection result. No new font flag or per-directory override is added.

## Derived row state

Entry kind, disclosure state, icon and palette role are derived presentation
values. They are not stored as filesystem metadata or durable entry records.
Observed path, depth, directory/symlink identity, expansion and incomplete state
retain their current owner. The row retains canonical identity separately from
its clipped display label.

Rendering uses the already loaded visible entry sequence. File-kind matching
uses a bounded compiled basename/suffix catalog, without content reads, parser
state, LSP requests, Git status or separate background indexing. Unknown names
map to the generic file role. Existing selection and viewport state are not
replaced by icon state.

## Compatibility

Existing configuration without the new leaves remains valid and receives the
decorated default. `tree.decorated = false` restores minimal presentation.
Existing explicit palette values remain intact; the feature does not rewrite
user configuration. Newly generated full samples include the new effective
settings through the existing generation contract.

The change introduces no bookmark, buffer, root, entry-identity or session-store
migration. The related active-buffer synchronization ticket remains open under
its own behavior decisions.

## Completed delivery evidence

The approved three-slice delivery is complete through canonical PRs #287-#289.
The retained controller passed the unchanged full gate at
`e0351ac918a760b7940e660494f2309d67bd30a4`, receipt `2WXLCTXPDKGL7ZMY7KI52IQSFP`.
The [tracker](../tracker/rev-better-file-tree.md#completed-gate-evidence) and
[Slice 3 closure](../report/slices/rev-better-file-tree/slice-3-acceptance.md#delivery-set-closure)
map BFT01-BFT04 and this model to implementation, configuration/presentation
acceptance, both input profiles, native/PTY geometry and local Markdown/Spore
guidance. Historical baseline candidates and receipts retain their identities.
Only permitted closure Markdown files change after the tested candidate;
required behavior and assertions remain intact. Physical-font drawing and the
separate selection-synchronization follow-up retain their stated boundaries.
