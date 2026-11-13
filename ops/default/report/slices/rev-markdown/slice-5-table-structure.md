# Slice 5: Table row and column operations

Status: reviewing
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-05-table-structure`
PR: pending

## Purpose

Deliver T5.1-T5.3: checked body-row and whole-column insertion/deletion, retained
source geometry and contextual structural routes in both interaction profiles.

## Delivered Behavior

- Insert a blank body row above the current body row, or first at a header.
  Contextual Meta-Return inserts below, or first at a header. Insertion refuses
  the delimiter. Delete only body rows; retain the header and delimiter even
  after removing the final body row.
- Insert an unspecified-alignment column before the current column; delete the
  current column while retaining at least one. Preserve surviving alignments
  and original source, including short body rows, escapes, code and Unicode.
- Use one canonical transaction and undo unit per structural action. Surviving
  cells retain other-pane points and marks; original EOF newline identity, BOM
  and CRLF remain intact. Refusals and canceled/stale work publish no partial edit.
- Expose four exact names through M-x/modal colon and M-S-Down/Up/Right/Left
  through contextual editing maps. Preserve explicit parent/local mappings,
  modal Normal/Visual/Replace states and completion ownership.

## Implementation Notes

Implementation and report introduction: `2636b40a7873391abbec3ba8d0e00de38b91ee06`.

The private table structure owner changes admitted source-range shapes before
using the existing bounded layout, retained-source and canonical publication
owners. New rows copy container continuation prefixes without retaining duplicate
source anchors. Short rows gain logical empty cells before column edits. Removing
the final body row transfers its ending identity to the last retained row.

Existing source stamp, mode generation, originating view/point, read-only,
cancellation and two-second cooperative worker checks continue to govern every
publication. No parser dependency, SDK/wire or persistence contract changes.

## Contracts Added or Changed

- markdown-table-insert-row, markdown-table-delete-row,
  markdown-table-insert-column and markdown-table-delete-column join the exact
  native catalog. markdown-meta-return dispatches to table insertion before lists.
- keys.markdown has twelve factory entries. Structural defaults activate only
  in admitted table context for non-modal editing and modal Insert.
- The complete sample, command descriptions, binding/reference catalogs and
  existing Markdown Unreleased entry describe the new actions and refusal rules.

## Files of Interest

- src/markdown/tables/structure.rs and action.rs: source shape operations and
  checked cursor destinations with unchanged finite admission bounds.
- src/session/markdown/tables.rs: dispatch through the existing action lifecycle.
- src/command/markdown.rs and src/input/keymap/: exact names, descriptions,
  factory routes and inherited map priority.
- tests/markdown/table_structure.rs and native table/input checks: placement,
  source/geometry/history, authority, cancellation and input regressions.
- docs/reference/markdown-tables.md: structural placement and refusal contract.

## Validation

Full workspace lint passes all 1,877 first-party Rust files, both packages and
74 Cargo roots, with zero blocking or deferred diagnostics. Documentation verifies
707 commands, 322 settings, 107 reachable pages and 759 local links/anchors.
All 18 focused Markdown table integration cases pass in 54.59 s; five table unit
checks, including Org work limits, pass in 2.22 s. The combined command takes
128.58 s including compilation. Earlier input/table unit checks pass five cases
in 2.64 s, including contextual structural hints and completion precedence.

All five required development commands passed in canonical order on clean
published HEAD `2636b40a7873391abbec3ba8d0e00de38b91ee06`, from
2026-11-13T21:11:10Z to 2026-11-13T21:33:12Z. All 586 tests across six native
targets passed without failures or ignored cases. The modal command retains its
exact contexts:: selection (103 unrelated cases filtered). Ordered full lint,
documentation and whitespace checks pass.

| Target | Passed | Seconds |
| --- | --- | --- |
| commands | 205 | 671.10 |
| configuration | 169 | 315.71 |
| markdown | 49 | 127.33 |
| org_tables | 73 | 72.20 |
| pointer | 69 | 75.87 |
| modal contexts | 21 | 33.03 |

Native targets took 1,295.24 s in total. The five-target command took 1,270.63 s
including preparation; the modal command took 35.15 s. Ordered lint took 15.62 s,
documentation 0.59 s and whitespace 0.01 s. The immutable-index commit hook also
passed complete workspace coverage. Actual cost changes no command, timeout,
assertion or acceptance condition.

Final worker acceptance requires one controller receipt on the exact clean
published HEAD after PR-reference commits, with no later HEAD changes. Integration
and the full six-slice delivery gate remain controller-owned.

Required ordered commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test pointer --test org_tables -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

Focused commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib --test markdown table -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib markdown:: -- --test-threads=1`

The initial structural run passed six of seven tests; its shared-pane fixture
incorrectly assumed splitting changes focus. The corrected fixture checks the
actual other pane and passes. Initial full lint passed Clippy but found two
cyclomatic-11 functions. Separating layout admission from operation selection and
mapping structural dispatch results resolved both under the unchanged maximum 10.

## Risks and Follow-ups

Full PTY acceptance and User Guide delivery remain Slice 6. The controller owns
integration and the full six-slice delivery gate. Existing finite source/output,
row/column/cell/width, parser and worker bounds remain unchanged.
