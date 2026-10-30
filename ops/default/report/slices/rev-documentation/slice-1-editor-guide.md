# Slice 1: Everyday Editor Guide

Status: delivered
Delivery set: rev-documentation
Plan: `ops/default/plan/rev-documentation.md`
Tracker: `ops/default/tracker/rev-documentation.md`
Branch: `docs/rev-documentation-01-editor`
PR: `#168`
Integrated commit: `01f03f00729f4465d3d853ee68fd9c30aa26c7cf`

## Purpose

Let readers learn everyday editing without taking an Org course. Keep Org
available through its own learning entry and preserve the existing examples.

## Delivered Behavior

- The User Guide has six chapters: first file, editing, buffers/panes,
  search/commands, tree and preferences.
- Org has a separate three-chapter path for notes, tasks/agenda and tables.
  Its index links focused procedures and the existing complete control catalog.
- README and all quadrant indexes distinguish those paths. Editor recovery
  stays with ordinary file tasks; Org procedures have their own heading.
- All 53 documentation pages remain reachable from the documentation index.
- A native Rust checker validates local targets, heading anchors, page
  reachability and both learning paths. A Bash driver defines the selected
  documentation gate in the approved command order.

## Implementation Notes

Existing chapter filenames and URLs remain. The visible sixth editor chapter
uses the original preferences filename; renaming it would break a published
link without helping the reader. Next/Previous links stay within each path.

Every original guide code block and the complete configuration sample are
unchanged. This slice changes navigation and introductory prose, so it reuses
the baseline's native workflow evidence rather than repeating unrelated PTY,
performance or whole-editor checks. T1.1 is commit
`4f8c8dc13d169fbbcb13f2477b5f828b8067ddfb`.

T1.2 retains entry-point commit `0c207ece72388bac167fed080351a65890b34a4a`.
Native checks are commit `53641811317a5545ee306f147f8bf5d6a9429aaf`.
The checker uses the Rust standard library without new dependencies. It scans
README, CHANGELOG and every Markdown page under docs; fenced examples and
inline code do not become links. Cycles are allowed, orphan pages reject,
and each guide must retain its own exact index and chapter sequence.

The branch adopts the approved controller-owned execution contract and reuses
PR #168. The immutable report link remains its original introduction commit,
`0c207ece72388bac167fed080351a65890b34a4a`.

The integration repair includes approved dev baseline
`34d8dba32dffcc78b0189fea33dd734f3458111f` in the slice history. The only
conflicts were plan/tracker metadata. Resolution retains the approved
controller-owned execution and Rust/Bash validation contract together with
Slice 1's reviewing state. Original task commits and the report introduction
are preserved; the repaired candidate received the successful controller
receipt recorded below.

## Contracts Added or Changed

The core learning sequence no longer crosses Org before reaching preferences.
The Org guide states its shared editing prerequisites and returns to the
User Guide index. This does not change the CLI, keybindings, configuration,
packaged sample or application behavior.

The Bash driver propagates failures and runs the native documentation checker,
command reference tests and configuration reference tests in that order.
Split-catalog completeness adaptation was completed in Slice 2 under T2.2.

## Files of Interest

- `docs/tutorials/user-guide/index.md`: the core chapter order.
- `docs/tutorials/user-guide/org/index.md`: independent Org learning entry.
- `docs/tutorials/user-guide/05-file-tree.md` and `09-preferences.md`:
  the direct tree-to-preferences navigation.
- `README.md` and the four quadrant indexes: separate reader entry points.
- `examples/documentation-check.rs`: native link, anchor and navigation checks.
- `scripts/check-documentation.sh`: the approved Rust/Bash gate driver.

## Validation

Controller receipt `E5KZ2QKMPB33LEFR44KLTMDIUW` passed all six ordered
handoff commands below for the exact integrated commit
`01f03f00729f4465d3d853ee68fd9c30aa26c7cf`: 53 reachable pages, 255 local
links/anchors and four regression tests. Canonical PR #168 is verified merged.

- `cargo test --example documentation-check` passed during implementation:
  four tests cover Markdown literals, missing targets/anchors, orphan pages
  and rejection of navigation crossing between the two learning paths.
- `cargo run --quiet --example source-check -- --root examples/documentation-check.rs`
  passed during implementation: safe Rust, at most five parameters, nesting
  at most three and cyclomatic complexity at most ten.
- The retained chapter diff changes navigation only; original guide code
  blocks and the complete configuration sample are unchanged.

The controller receipt recorded these successful handoff commands in order
against the exact integrated candidate:

- `cargo run --quiet --example documentation-check`
- `cargo test --example documentation-check`
- `cargo run --quiet --example source-check -- --root examples/documentation-check.rs`
- `rustfmt --check --edition 2024 examples/documentation-check.rs`
- `bash -n scripts/check-documentation.sh`
- `git diff --check`

The controller passed the combined delivery gate
`scripts/check-documentation.sh && git diff --check` at immutable candidate
`770d783a3147a24cc613460554b6261d22c92749`, receipt
`QARF7RQ3WL333CPQYZPN45TXC4`. The integrated result covers 58 reachable
pages, 330 local links/anchors, complete command/configuration catalogs and
21 native reference tests. Final closure changes only the configured
operational documents after that tested candidate.

External HTTP links are outside the local checker. No runtime instructions
changed.

## Risks and Follow-ups

The reference extraction and completeness-guard updates are delivered through
PR #169. The v0.1.0 documentation and changelog are delivered through PR #170,
and the controller's final integrated-candidate gate passed. No follow-up
remains in this delivery set.
