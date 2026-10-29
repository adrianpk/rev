# Slice 40: edit Org table rows columns and references

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-40-table-structure`
PR: [#141](https://forge.adrianpk.com/adrianpk/rev/pulls/141)

## Purpose

Deliver structural table editing under the
[approved plan](../../../plan/rev-org-mode.md#slice-40-edit-org-table-rows-columns-and-references)
and [tracker](../../../tracker/rev-org-mode.md#slice-40-edit-org-table-rows-columns-and-references).

## Delivered Behavior

T40.1 delivers row/column insertion, removal and movement, stable alphabetical
and numeric sorting, and checked flat-list/one-column-table conversion. Meta
arrows dispatch to native table operations in table context. Sorting honors the
active region's endpoint rows and mark column, while retaining separator groups.
T40.2 rewrites absolute field/column/row references across structural edits,
including assignment targets and supported rectangular ranges. Unsupported or
ambiguous transformations reject the complete operation before publication.
All table and formula changes use one ordinary undo unit.

## Implementation Notes

Task commits: T40.1 `f1fc1cf7ecf203aca3fc60579a91293ba7981a51`,
T40.2 `fe1580557d2eef0aaf126385a3d95659e3191768`,
T40.3 `10a4c277d4e2bdc8313fbb9134838fa263b91987`.

Structural plans use the existing captured canonical rope, native scanner,
layout and single cancellable job. The buffer transaction owner verifies retained
source stamps, equal candidate bytes, disjoint source/output ranges and containment
inside the actual changed source/candidate pair. Shared points and marks follow
those retained cell values. Other positions use ordinary edit affinity.
History replays the same map in both directions; sparse lookup uses binary search.
No fragment receives an invented buffer identity or separate persistence owner.

The existing table bounds remain: 8 MiB source/output, 10000 rows, 256 columns,
100000 rendered data cells and 4096 displayed columns per fragment. Canonical
retained maps admit at most 200000 runs. Sorting is bounded to selected rows or
one separator group and admits a/A or n/N; non-finite/nonnumeric numeric keys
reject. Uppercase selects reverse order without reversing equal-key groups.
Conversion admits contiguous flat single-line lists and one-column tables
without separators; nested/continued items, literal table delimiters and
ambiguous table shapes reject before publication.

Formula rewriting tracks original logical axis identities independently of
retained-byte provenance. It validates every interior range coordinate rather
than treating endpoint provenance as proof of unchanged semantics. Only changed
coordinate digits receive actual stamped replacements; all formula lines join
the same canonical transaction. Identical cell bytes still retain real movement
identity and shared positions. The rewrite scanner admits compatible arithmetic,
finite literals and the documented function list, bounded to 100000 tokens and
64 parenthesis levels; it does not evaluate or recalculate formulas.

T40.3 extends native save/history/shared-view checks and the existing RGB and
NO_COLOR table journeys. The journeys retain their original two-session count,
8-second session deadline, 1 MiB terminal-output ceiling, 32-entry/64 KiB fixture
bounds, idle assertions and cleanup checks. They await completed table navigation
before sending the next prefix; an intermediate checking message is not accepted
as completion. Supported formula insertion/hlines now assert successful source
changes and complete undo; an explicit unsupported formula still rejects.

The stable numeric-sort check exposed signed-zero ordering from `total_cmp`.
Finite numeric comparison now treats signed zero, zero and empty fields as equal
keys, preserving their source order in both directions. Read-only structural
checks select an actual movable column before testing mutation denial; table-edge
rejection is distinct from buffer authority.

## Contracts Added or Changed

- ORG12: real structural table operations and retained cell/source coordinates.
- ORG14: checked absolute-reference rewrites preserve logical targets; unsupported syntax, removed targets and noncontiguous/reordered ranges reject atomically.

## Files of Interest

- `src/org/table/edit/`: structural plans, sorting and conversion.
- `src/buffer/edit/retained.rs`: canonical retained-source admission.
- `src/session/tables.rs`: current-source publication and owned sort prompts.
- `tests/org_tables/structure.rs`: actual editing/history/shared-view workflows.
- `src/org/table/references/`: bounded reference scanning and explicit logical-axis/range validation.
- `tests/org_tables/references.rs`: actual structural reference transactions and atomic rejection.
- `tests/org_tables/lifecycle.rs`: structural file fidelity, identical shared rows, sort/current-source guards and bounded rejection.
- `scripts/configuration_ui/org_tables.py`: owned physical-key, canvas, save and cleanup checks.
- `docs/reference/org-tables.md`: supported commands and boundaries.

## Validation

- `cargo test --test org_tables -- --nocapture`: 23 workflows pass, zero failed or ignored, including six structural workflows for row/column edits, separator boundaries, group/region sorting, conversion and shared positions/marks with undo/redo.
- Changed-source quality passes with unchanged parameter, nesting and cyclomatic limits.
- The first row-up fixture expected a row to jump over two lines; its exact expectation now matches a single adjacent-row move, including movement across a separator. The implementation's row move did not change for this fixture correction.

- `cargo clippy --lib --test org_tables --test commands -- -D warnings`: passed.
- `cargo test --lib buffer::edit::retained -- --nocapture`: canonical retained-source admission passed, including mismatched bytes, stale/foreign identities, overlapping/out-of-range claims, claims outside actual changes and read-only rejection.
- `cargo test --test commands command_sequences -- --nocapture`: passed.
- `cargo test --test commands catalog_reference -- --nocapture`: passed after removing one duplicate documentation entry for the existing row-insert command; the new binding remains documented.
- `python3 scripts/check-config-reference.py`: 155 schema/catalog entries, 54 RGB roles and 249 bindings agree; the 19431-byte packaged template is exact.

- `cargo test --test org_tables references -- --nocapture`: seven reference workflows passed, covering multiple CRLF formula lines, insert/delete/move, rectangular shifts, complete range-target proof, unsupported forms, identical cells and undo/redo.

- `cargo test --test org_tables metadata -- --nocapture`: five existing metadata, unknown-formula preservation and table-ownership workflows passed.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed for the reference implementation; changed-source quality also passed.

- `cargo test --test org_tables lifecycle -- --nocapture`: five workflows passed for structural BOM/CRLF/Unicode/tab saves and undo/redo, identical shared rows/marks, sorted formula transactions, cancellation/stale source, range-region rejection and 64-level/100000-token limits.
- `cargo test --test org_tables equal_numeric_keys -- --nocapture`: passed after reproducing and correcting signed-zero reverse-sort instability.
- `cargo test --test org_tables readonly_tables -- --nocapture`: passed for native navigation plus row/column mutation denial.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tables`: both existing owned RGB/NO_COLOR sessions passed with physical structural keys, exact formula digits, sort prompts, atomic rejection, save bytes, idle state and cleanup; zero skipped.

- Focused controller batch `CJ47HAEC7XJ5QCUSEWUSBSGOLS` at `aae331e63d614a24e807ed74016eef9c03940aaf` exposed three catalog fixtures missing the newly delivered `org-table-kill-row` result. Their complete lexical expected list and substring counts now include that command; prefix matching expectations remain unchanged.
- `cargo test --test commands -- palette::filter_order selection::query_changes vertical::configured_matching`: all three corrected fixtures passed; changed-source quality passed. Final controller evidence is regenerated for the corrected candidate.

- Final required receipt `U3Q43PKH3AXV25XJSCHQRWP5E2` at `e0f3401ad564847c2286ceafaba23c9114b39147` passed the five ordered repository checks and 256 native tests.
- Final focused receipt `7YB63JTRCI5JXC6V7RGZMGMORG` at the same head passed 341 native regression tests, configuration parity and both owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions.
- Canonical PR #141 merged into dev at `e0f3401ad564847c2286ceafaba23c9114b39147`, verified against Forgejo and origin/dev.

## Risks and Follow-ups

The pre-existing outline timer assertion diagnostic remains in the Slice 39
report; no timer behavior or assertion is changed here.
