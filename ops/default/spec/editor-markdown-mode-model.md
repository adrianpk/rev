# Markdown Policy and Actions

Status: Approved
Date: 2026-11-13
Specification: [Markdown Editing and Presentation](editor-markdown-mode.md)
Plan: [Markdown Delivery](../plan/rev-markdown.md)
Approved: product owner, 2026-11-13; Markdown scope and parser adoption subject to verified compatible licensing

## Owners and analysis

The canonical buffer owns bytes, revision, marks, transactions and history.
A Markdown owner interprets source; the shared projection owns display/cursor
mappings. Store observations qualified by buffer and revision, with source byte
ranges, heading level, list depth, literal ownership and table/cell boundaries.
Keep parser types private and retain original source instead of decoded event
text for edits. A pipe splitter or a Tree-sitter highlight capture alone cannot
authorize a table mutation. Incomplete or stale evidence authorizes no edit.

Selected parser: `pulldown-cmark = { version = "=0.13.6", default-features = false }`.
Enable only `Options::ENABLE_TABLES`; do not enable HTML output, task lists,
footnotes or unrelated extensions. The accepted dependency uses MIT terms.
Use a bounded worker snapshot; one bounded contiguous source copy per admitted
revision is acceptable for this parser API. Check cancellation around parser
construction and between event batches, discard late results and avoid reparsing
on every draw. No native-library API or permanent parser thread is introduced.

The inspected minimal graph adds only pulldown-cmark 0.13.6 (MIT) and unicase
2.10.0 (MIT OR Apache-2.0; use MIT). Reuse the locked bitflags 2.13.2 and memchr
2.8.3 already present. Preserve existing dependency versions and pin the two new
packages in Cargo.lock. Retain the exact upstream MIT notices for the added
packages in a shipped third-party notice document; keep Rev's GPL license intact.

The owner prefers fewer dependencies and may replace this parser with an owned
implementation later. Keep external parser types inside a private adapter;
original source and editor-owned records remain authoritative. This establishes
a replaceable boundary, not a plugin registry or a second implementation now.

Keep heading/list/table recognition independent of syntax color enablement.
Apply the existing configured syntax byte cutoff, with an initial native cap
of 8 MiB and 200,000 retained nodes/captures and 256 structural levels. Existing
table limits in the specification also apply. A lower active bound wins.

## Preference contract

Use existing TOML include/override/provenance and whole-candidate rejection rules.
Omitted fields inherit their factory values; existing preferences are not
rewritten. The complete sample and the command/configuration catalogs stay exact.

| Preference | Default | Effect and delivery |
| --- | --- | --- |
| `markdown.enabled` | `true` | Admit native Markdown mode and automatic `.md`/`.markdown` recognition; Slice 1. |
| `markdown.pretty` | `true` | Initial per-view decoration policy for ATX headings and unordered lists; Slice 2, extended to lists in Slice 3. |
| `keys.markdown` | Compiled map | Markdown editing overlay, inheriting effective editor/profile maps with explicit user overrides preserved; Slice 3. |

`palette.markdown` has RGB leaves `heading`, `heading_2` through `heading_6`,
`emphasis`, `link`, `code`, `quote`, `list` and `table`. Heading defaults are the
six exact colors in the specification. Other defaults are respectively
`#9d7cd8`, `#7aa2f7`, `#9ece6a`, `#565f89`, `#89ddff` and `#89ddff`.
Emphasis includes strong emphasis. Inline semantic colors precede enclosing
heading/table colors. Selection and search retain their existing precedence.
Semantic palette leaves arrive in Slice 1; heading leaves arrive in Slice 2.
There is no reference to user-customized Org colors in Markdown policy.

The pretty override belongs to a view: splits inherit its effective value,
then change independently; reset restores the startup value without TOML writes.
Text mode suspends Markdown presentation. Read-only views admit presentation
controls. Glyph sequences are the fixed approved profile, with no new marker
array preferences. The terminal continues to own font coverage.

## Named actions and default routes

Both profiles expose these exact names in M-x/modal colon and descriptions.
Editing chords apply to non-modal editing and modal Insert; preserve modal
Normal/Visual state ownership and existing completion-popup precedence.
Explicit user mappings retain priority over factory context defaults.

| Commands | Factory editing route |
| --- | --- |
| `markdown-mode` | Named only; explicit mode selection does not rename the file. |
| `markdown-pretty-mode`, `markdown-reset-presentation` | Named only; active-view toggle/reset. |
| `markdown-continue-item` | Named; M-RET dispatches here in a list. |
| `markdown-indent-item`, `markdown-outdent-item` | M-Right / M-Left in lists. |
| `markdown-table-create` | C-c followed by the vertical-bar key on a blank line. |
| `markdown-table-align` | C-c C-c in a table. |
| `markdown-table-next-field`, `markdown-table-previous-field` | TAB / S-TAB in a table. |
| `markdown-table-next-row` | Named; contextual Return dispatches here in a table. |
| `markdown-table-insert-row`, `markdown-table-delete-row` | M-S-Down / M-S-Up for body rows; M-RET also inserts a body row in a table. |
| `markdown-table-insert-column`, `markdown-table-delete-column` | M-S-Right / M-S-Left in a table. |
| `markdown-return`, `markdown-meta-return` | RET / M-RET context dispatch; outside a supported table/list action retain ordinary newline behavior. |

Create-table asks for explicit columns x body rows and retains a header plus
delimiter even for zero body rows. Dimensions obey the specified bounds.
Insert-column inserts before the current column; delete-column removes it.
Insert-row inserts above the current body row, or the first body row at a header;
contextual M-RET inserts below. Delete-row refuses header/delimiter rows.
All controls refuse invalid targets or read-only edits without partial mutation.

## Transactions and validation

An action plan carries its source revision, disjoint replacements, optional
cursor destination and retained-source ranges. Only the existing buffer owner
can validate and publish it. One source action creates one undo unit; already
aligned navigation and presentation create none. Preserve newlines, cell source,
marks and shared view points using existing checked publication semantics.

Keep Markdown parsing, list actions and table actions separate from Org syntax,
formula rules and command enums. Extract a shared pure helper only for a concrete
common contract and cover the affected Org path in focused regressions.
No new workspace/session persistence record or SDK/wire protocol is introduced.
