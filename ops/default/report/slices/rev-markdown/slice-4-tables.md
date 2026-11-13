# Slice 4: Table creation and field editing

Status: reviewing
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-04-tables`
PR: `#37`

## Purpose

Deliver T4.1-T4.3: parser-admitted GFM table source shapes, explicit creation,
alignment and field/row navigation in both profiles. Explicit row/column
insertion and deletion remain Slice 5.

## Delivered Behavior

- Recognize only tables admitted by the private GFM parser. Retain original cell
  spans, container prefixes and delimiter alignment. Accept optional outer pipes,
  short body rows, escaped pipes and inline markup. Refuse extra body cells that
  GFM rendering would discard and mismatched header/delimiter context.
- Create on a blank line from explicit columns x body rows, retaining a header
  and delimiter even with zero body rows. Cancellation or invalid dimensions
  leave source unchanged. New tables keep neighboring paragraphs separate, and
  their prospective parser observation must admit the exact requested shape.
- Align source padding and navigate fields while skipping delimiters. Next-field
  at the final field and next-row at the last row append a body row; previous-field
  at the first header field stays there. Already aligned movement has no history.
- Preserve cell bytes, Unicode, BOM, CRLF and final-newline identity. One action
  publishes one canonical undo unit. Retained source points in other panes follow
  their cells and exact undo restores their original positions.
- Expose all five table names in M-x and modal colon. Factory C-c then vertical
  bar creates; C-c C-c aligns, TAB/S-TAB traverse and Return advances rows in a
  current table. Factory table routes are restricted to non-modal editing and
  modal Insert with admitted context; explicit parent/local remaps and completion
  retain their priority.

## Implementation Notes

Implementation and report introduction: `af6d28fdbf6afca809206e0298648a8c8d33b369`.
Canonical PR: https://forge.adrianpk.com/rev/rev/pulls/37
Development evidence commit: `3c422f273ec11051ecbbc51301784c0fef01c2ae`.

The parser adapter records table and row source ranges alongside heading/list
observations. Separator scanning operates only inside that admitted context and
retains original bytes rather than decoded inline event values. Parser types stay
private, with no dependency, SDK/wire or persistence changes.

A separate temporary table-action worker uses a bounded rope snapshot, source
stamp, mode generation, originating view and point. Cancellation, stale source,
read-only authority or an intervening prompt prevent publication. The existing
canonical buffer/window owner validates edits, retained source and cursor targets.
Table source rules and command enums remain separate from Org.

The finite profile admits at most 8 MiB source/output, 10,000 total rows,
256 columns, 100,000 cells and 4,096 displayed columns per cell. Existing source
byte/event/depth and two-second cooperative worker bounds still apply. A lower
configured syntax byte limit wins.

## Contracts Added or Changed

- markdown-table-create, markdown-table-align, markdown-table-next-field,
  markdown-table-previous-field and markdown-table-next-row join the exact native
  command catalog. markdown-return dispatches to table navigation before lists.
- keys.markdown has eight factory entries. Context selection also feeds hints,
  macro key capture and plugin priority without a new configuration or wire role.
- The complete sample, command descriptions, binding inventory, table reference
  and existing Markdown Unreleased entry describe the delivered behavior.

## Files of Interest

- src/markdown/parse.rs and src/markdown/tables/: GFM admission, source shapes,
  bounded layouts, creation/navigation plans and original-source retention.
- src/session/markdown/tables.rs and src/session/markdown.rs: prompt/action
  lifecycle, cancellation and canonical publication.
- src/input/markdown.rs, src/input/keymap/ and input hint/macro/plugin owners:
  context-dependent factory defaults and inherited remap precedence.
- tests/markdown/tables.rs and native table/input tests: source, transaction,
  profile, cancellation, bounds and shared-point regressions.
- docs/reference/markdown-tables.md and the existing command/configuration
  inventories: concrete native table behavior and complete catalog coverage.

## Validation

All four required development commands passed in canonical order on clean
published implementation HEAD `af6d28fdbf6afca809206e0298648a8c8d33b369`, from
2026-11-13T20:00:17Z to 2026-11-13T20:19:18Z. All 489 tests across four native
targets passed without failures or ignored cases. Full workspace lint passes all
1,875 first-party Rust files, both packages and 74 Cargo roots, with zero blocking
or deferred diagnostics. Documentation verifies 703 command names, 322 settings,
107 reachable pages and 759 local links/anchors. Whitespace passes.

Final worker acceptance requires one controller validation receipt on the exact
clean pushed HEAD after PR-reference commits, without modifying that tested HEAD.
The controller owns integration and the full six-slice delivery gate.

Required ordered commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test org_tables -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

| Target | Passed | Seconds |
| --- | --- | --- |
| commands | 205 | 675.22 |
| configuration | 169 | 282.27 |
| markdown | 42 | 94.85 |
| org_tables | 73 | 66.19 |

The native targets took 1,118.53 s in total; their exact combined command took
1,125.47 s including preparation. Ordered lint took 15.13 s and documentation
0.60 s. This measured cost changes no command, timeout or acceptance condition.
The installed immutable-index hook also passes complete workspace coverage.

All eleven focused table integration cases passed in 21.37 s; four native
unit checks passed in 1.86 s. Their exact development commands were:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown tables:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib markdown:: -- --test-threads=1`

The initial focused run passed three of six cases; three expected-output failures
came from one missing padding space in the Unicode fixture. The corrected fixture
accounts for the terminal width of its wide and combining source. The subsequent
seven-case run passed without failed or ignored tests in 17.33 s. Initial native
source checking identified two complexity violations, split into their table
navigation and blank-line admission owners without exceptions or weaker limits. The complete lint pass also caught
command-name/description match complexity and a nested profile fixture; their
format-owned dispatch and fixture routing were split under the unchanged policy.
Creation now isolates adjacent paragraphs and verifies the prospective GFM table
shape, protecting unclosed literal blocks at EOF.

## Risks and Follow-ups

Explicit row/column structural actions remain Slice 5. Full PTY acceptance and
User Guide delivery remain Slice 6. The existing foreign crossterm unused_parens
warning is outside first-party source policy.
