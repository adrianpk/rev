# Slice 8: Guidance and gate preparation

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `docs/rev-projects-workspaces-08-guidance`
PR: pending

## Purpose

Deliver PW23 guidance and reconcile the approved PW01-PW22 opening and ownership
contracts. Close verified Slice 7 PR #304 and prepare the final documentation PR
for the unchanged controller gate. Integration, the combined gate and final
thirteen-path closure remain controller-owned.

## Delivered Behavior

- The User Guide now leads from standalone files through temporary directory
  work, Project promotion and named restoration to a two-root independent
  Workspace. Short CLI and native-screen examples use existing keys and commands.
- Focused how-tos cover membership, conversion versus duplication, rename,
  relocation and forgetting while preserving source files. An explanation
  separates live editing, named snapshots, root ownership and dashboard display.
- Reference material documents all context commands and the exact version-1 TOML
  records, typed identities, revision linkage, native path encoding, scalar
  positions, byte/count limits, atomic publication and preserved failures.
- Defaults, prompts, tree guidance and baseline requirements describe checked
  directory replacement and the five-action startup screen. Project/Workspace
  sessions restore explicitly; no inactive startup-restoration setting is exposed.
- Markdown remains the Spore source. Guide ordering, previous/next links, Spore
  navigation and the existing checker agree. One Unreleased entry describes the
  complete product outcome and links the new learning path.

## Implementation Notes

Closure/activation is `a7363e012ec0f2a883ad711c202e2dd96d8a7fbb`: PR #304 is delivered at
`277dbabb068c10bf3bdc011bebe5e2c845519afc`, with six passing checks in receipt
`KKQIRFTGHTBUIST5T7J3T2ZQTP`.
T8.1/T8.2 and the Unreleased portion of T8.3 are implemented in `eba31a7072fbe1e4ad78e3f89b034e5ec0c48391`.
The existing documentation-check navigation case was updated for the new chapter;
its rejection cases and all seven existing tests remain active.

The new chapter uses a new stable filename after file browsing; existing chapter
paths and the old management anchor remain valid. Generated HTML stays under
ignored `target/userguide`; the build performed no publication or installation.
No runtime source, dependency, banner, configuration schema or gate changed.

All previous canonical PRs were verified merged into dev. Their delivered reports
retain their original introductions and detailed validation evidence:

| Slice | Canonical PR | Immutable report introduction | Integration commit |
| --- | --- | --- | --- |
| 1 | [#298](https://forge.adrianpk.com/adrianpk/rev/pulls/298) | [report](https://forge.adrianpk.com/adrianpk/rev/src/commit/fd3d8f51132f3a74b9ce8824084f128df5d1b664/ops/default/report/slices/rev-projects-workspaces/slice-1-state.md) | `975564c53cf7c6f420c7d7b3fbb8d5f46950e710` |
| 2 | [#299](https://forge.adrianpk.com/adrianpk/rev/pulls/299) | [report](https://forge.adrianpk.com/adrianpk/rev/src/commit/ab0f958003ad81ac8dc59d8f79c45d7e8e3416f8/ops/default/report/slices/rev-projects-workspaces/slice-2-directories.md) | `4a1b2ddfef6e15a97f0c50d86c46df582612b884` |
| 3 | [#300](https://forge.adrianpk.com/adrianpk/rev/pulls/300) | [report](https://forge.adrianpk.com/adrianpk/rev/src/commit/fc97a4f8a17e0776bddc24d6523da67611d8150c/ops/default/report/slices/rev-projects-workspaces/slice-3-projects.md) | `ff3b361ece896a42155c8817f0e51dc0e6a78fa8` |
| 4 | [#301](https://forge.adrianpk.com/adrianpk/rev/pulls/301) | [report](https://forge.adrianpk.com/adrianpk/rev/src/commit/2b9b7916024932d59cf6463d2bbb85d3f29090f8/ops/default/report/slices/rev-projects-workspaces/slice-4-workspaces.md) | `84a934fa2813486fa43944b30e36e01f1f6b35cc` |
| 5 | [#302](https://forge.adrianpk.com/adrianpk/rev/pulls/302) | [report](https://forge.adrianpk.com/adrianpk/rev/src/commit/5813d3f9dfaba049598fac448f48defef434c1b9/ops/default/report/slices/rev-projects-workspaces/slice-5-management.md) | `2d81d9c84ae59e85188d683a9366cd5afc9b2511` |
| 6 | [#303](https://forge.adrianpk.com/adrianpk/rev/pulls/303) | [report](https://forge.adrianpk.com/adrianpk/rev/src/commit/3e05e14b6920a6286c1935e854bdbfb6d7e9cfc3/ops/default/report/slices/rev-projects-workspaces/slice-6-startup.md) | `03751136d7b40e2ac823b1966db32384cbcefb87` |
| 7 | [#304](https://forge.adrianpk.com/adrianpk/rev/pulls/304) | [report](https://forge.adrianpk.com/adrianpk/rev/src/commit/f65b3a341e0d2c214b97a25810146e9955ecca6c/ops/default/report/slices/rev-projects-workspaces/slice-7-acceptance.md) | `277dbabb068c10bf3bdc011bebe5e2c845519afc` |

T8.3's report/changelog and gate preparation are delivered here. Its post-gate
closure portion remains pending with the tracker final-gate checklist. The
controller must merge Slice 8, validate the exact integrated candidate, then
complete only the configured documentation paths. This report does not claim
that the delivery set or its core specification is complete.

## Contracts Added or Changed

- NAV01/NAV02 distinguish a file visit from temporary context replacement;
  NAV03 scopes initial expansion to single-root browsing while Workspace peers
  begin collapsed. UI03 and consolidated R12 use native source/clock/job departure.
- Consolidated R01/R02/R28 and CFG19 reflect dashboard-first executable startup,
  one optional OS path argument and context-owned roots, without changing historical
  completed statuses, candidates, receipts or unrelated pending work.
- Persistent sessions use one-based logical lines and Unicode-scalar columns,
  root-relative or loose absolute references, source-anchored viewports and one
  selected snapshot per owner. Whole-application restoration and pane/tree layout
  restoration remain deferred.
- All fixed count/encoded-byte limits apply together. A Workspace remains a set
  of independent editing roots, with existing per-source service authority.

## Files of Interest

- `docs/tutorials/user-guide/20-projects-and-workspaces.md` and its guide/index navigation.
- `docs/how-to/compose-workspace.md` and `docs/how-to/manage-saved-contexts.md`.
- `docs/reference/context-state.md`, `contexts.md`, `defaults.md`, `file-tree.md` and `prompts.md`.
- `docs/explanation/editing-contexts.md` and `editing-model.md`.
- `examples/documentation-check.rs` and `publication/userguide/index.sptree`.
- `ops/default/spec/editor-file-navigation.md`, `editor-interaction.md`, `editor-configuration.md` and `rev-editor.md`.
- `README.md`, `CHANGELOG.md` and the plan/tracker/report links above.

## Validation

Implementation-time required checks passed in this order. Final clean-HEAD
controller evidence must follow the PR-reference commit and push:

- `cargo fmt --all -- --check` — passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings` — passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — passed.
- `bash -n scripts/check-projects-workspaces.sh` — passed.
- `bash -n scripts/build-user-guide.sh` — passed.
- `git diff --check` — passed.

The documentation checker passed 91 reachable pages and 622 local links/anchors;
combined inventories match 618 commands and 289 configuration entries.
All-target Clippy passed; the unchanged vendored crossterm warning remains
outside first-party scope.

Additional focused checks passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --example documentation-check -- --test-threads=1` — seven tests passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh` — projected/rendered 91 pages with strict link checks, local output only.

The initial Spore projection rejected a new documentation-index link that escaped
the documentation tree to CHANGELOG.md. Removing that link retained the release
boundary statement and the final local build passed. New guide/supporting HTML
pages exist; generated output is ignored. Historical specification metadata and
the supplied banner hash remain unchanged.

This documentation slice did not rerun the native/PTY acceptance matrix or the
combined delivery gate. Slice 7 retains its actual capacity, latency, memory,
resource, terminal and fixture observations; they are not relabeled as new
integrated-candidate evidence.

## Risks and Follow-ups

- Controller-owned merge, the frozen combined gate and final documentation-only
  closure remain pending. Gate receipts, actual phase/resource observations and
  all eight delivered report states must be recorded before set closure.
- The build proves local Markdown projection and links, not publication, browser
  deployment, automatic last-session restoration or portable performance bounds.
