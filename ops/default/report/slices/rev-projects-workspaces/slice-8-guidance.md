# Slice 8: Guidance and gate preparation

Status: delivered
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `docs/rev-projects-workspaces-08-guidance`
PR: `#305`
Canonical PR: [#305](https://forge.adrianpk.com/adrianpk/rev/pulls/305)
Report introduction: [immutable report](https://forge.adrianpk.com/adrianpk/rev/src/commit/533813a14bf21856d9d04890b0b28c05bf05eff7/ops/default/report/slices/rev-projects-workspaces/slice-8-guidance.md)

Integrated: PR #305 merged into dev at `cecd8205c7f61f503e9c4c1da963cefe6686600f`; all six required commands passed on that clean HEAD in receipt `COXYQZ74QJZGY2OD4BJK355A66`.

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

T8.3 is complete after the controller's successful immutable-candidate gate and
the configured documentation closure below. All eight canonical PRs and reports
are delivered. The plan, tracker and approved core specification/model are
Completed. P19 remains outside this completed core.

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

Implementation-time required checks passed in this order. The final PR-reference
HEAD was pushed and its six-command clean-HEAD receipt is recorded above:

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

Gate preparation also passed on the integrated source:

- `cargo metadata --locked --offline --format-version 1` — passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked --offline --bin rev --example projects-workspaces-acceptance --example documentation-check --example userguide-publication` — passed.
- `bash -n scripts/check-projects-workspaces.sh` — passed.
- `bash -n scripts/build-user-guide.sh` — passed.

All frozen-gate targets were admitted with 118 cached packages. The selected
primary-checkout editor and three examples compiled in 44.65 seconds. Rust/Cargo
1.98, rust-analyzer 0.3.3089, Bash, Git, Spore, cc, Python 3, util-linux script and
core utilities are available. Cached Rust/Python grammar parser/scanner sources
are present for owned fixtures. The Pi external-executable override is absent.
An owned temporary HOME/XDG fixture ran the existing PTY wrapper with `/bin/true`,
verified `REV_TTY_RESTORED`, created no user catalog and was removed afterward.
This admits tools and isolation; it does not execute an editor journey or prove
the combined behavior. No full gate ran during preparation.

## Risks and Follow-ups

- The final controller gate passed and documentation closure is complete.
  Native resident-memory costs and local performance observations below remain
  evidence limits, rather than portable memory or latency guarantees.
- The build proves local Markdown projection and links, not publication, browser
  deployment, automatic last-session restoration or portable performance bounds.


## Delivery-set closure

Status: Completed on 2026-11-07. All eight canonical PRs (#298-#305) are merged
into dev, and all eight slice reports are delivered. The controller's successful
gate tested immutable candidate `6bbc6138ac9227beca51e6b86f759bbf39e36c94` with receipt
`ZS6AIYOX66WRNHE5ICPVA46GIP`. The receipt JSON and matching log were verified against
the exact frozen command. It ran from 2026-11-07T04:13:13.249586792Z to 2026-11-07T04:26:50.430550265Z,
817.18 seconds (13 minutes 37.18 seconds). Closure changes only the thirteen
configured documentation paths; tested runtime, dependencies, artwork, guides,
scripts and assertions are unchanged. No new slice, repair PR or gate rerun was
needed. The core specification/model, plan, tracker and index entry are closed.

The exact successful combined command was:

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 && cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --lib --test contexts --test configuration --test commands --test modal --test display --test files --test tree --test recovery --test pointer -- --test-threads=1 && cargo test --locked --test git_integration source -- --test-threads=1 && cargo test --locked --test integrated_terminal lifecycle -- --test-threads=1 && cargo test --locked --test shell_commands lifecycle -- --test-threads=1 && cargo test --locked --test pi --test lsp -- --test-threads=1 && bash scripts/check-projects-workspaces.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check
```

The gate passed 1,036 selected Rust tests with no failures or ignored cases.
Nested isolated child-test output is included in its parent suite, not counted
a second time. Filtering below is the frozen gate's explicit selection.

| Suite/selection | Passed | Filtered | Reported execution seconds |
| --- | --- | --- | --- |
| `lib` | 152 | 0 | 43.46 |
| `commands` | 172 | 0 | 215.31 |
| `configuration` | 160 | 0 | 96.75 |
| `contexts` | 62 | 0 | 28.19 |
| `display` | 45 | 0 | 12.14 |
| `files` | 75 | 0 | 11.12 |
| `modal` | 120 | 0 | 93.40 |
| `pointer` | 67 | 0 | 36.70 |
| `recovery` | 25 | 0 | 3.79 |
| `tree` | 71 | 0 | 12.04 |
| `git_integration source` | 9 | 233 | 21.51 |
| `integrated_terminal lifecycle` | 8 | 41 | 10.77 |
| `shell_commands lifecycle` | 9 | 26 | 5.65 |
| `lsp` | 46 | 0 | 30.66 |
| `pi` | 15 | 0 | 28.54 |

Reported compilation phases were 50.42 seconds for all-target Clippy, 57.85
seconds for the first combined test build, then 4.70/1.58/0.92/2.01 seconds for
Git source, terminal lifecycle, shell lifecycle and Pi/LSP test builds. The
acceptance helper build reported 0.05 seconds. Formatting, documentation checking
and Spore rendering have no separate phase timer in this log; total receipt time
includes them. The unchanged vendored Crossterm warning was retained.

The final gate's actual acceptance observations were:

| Boundary | Observation |
| --- | --- |
| Ordinary catalog | 50 artifacts / 1,000 file references; 325,058 encoded bytes; retained vector capacity 458,752 bytes; 771,339 microseconds; longest service pass 5,810 microseconds, resize 27 microseconds. |
| Encoded session boundary | 512 files; 2,094,740 charged session bytes against 2,097,152; 2,095,089 catalog bytes; oversized candidate rejected without truncation. |
| Aggregate/count boundary | 256 Projects, 128 Workspaces, 64 roots, 512 files; 8,354,858 encoded bytes against 8,388,608; vector capacity 11,927,552; candidate 8,354,610 bytes. |
| Storage resident memory | 69,704 KiB before storage fixture; 84,420 encoded; 131,772 prepared; 171,780 published; peak 176,484 KiB. |
| Cancellation | Three canceled reads retained 171,788 KiB each; staged cancellation retained source generation/stamp/state; post-cancel 240,260 KiB, peak 245,696 KiB. |
| Two 512-file rounds | Restore 4,134,380 / 4,168,985 microseconds; capture 4,283,815 / 4,328,206 microseconds; post-round residency 281,376 / 305,536 KiB; longest service pass 48,862 microseconds, resize 425 microseconds. The 513th capture was rejected with unchanged state/source. |
| Resource ownership | Peak 10 descriptors / 3 threads; final counts returned to baseline 4 descriptors / 1 thread. Native peak RSS 364,692 KiB; post-owner-drop RSS 312,184 KiB versus initial 9,344 KiB. |
| Non-modal PTYs | Three serial starts in 3,968 ms; output 545,586 + 6,686 + 7,784 bytes; observed editor peak RSS up to 30,844 KiB. |
| Modal/NO_COLOR PTYs | Three serial starts in 4,049 ms; output 285,714 + 4,593 + 4,837 bytes; observed editor peak RSS up to 32,180 KiB. |
| Idle | Native and Linux PTY probes observed zero CPU ticks over 350 ms; native context work had no pending timer. |

The native/PTY entrypoint passed startup actions and banner/resize behavior,
explicit file/directory restarts, temporary opening of registered paths,
promotion, Unicode anchors, independent sessions, empty Workspace handling,
duplicate/cancel/undo, membership and atomic conversion. Source/configuration
bytes and TTY restoration passed. The script reports all owned fixtures cleaned
on exit, and native descriptor/thread owners returned to baseline. No remaining
owned fixture required a stop; no unrelated process was stopped.

Documentation passed 618 commands, 289 configuration entries, 91 reachable pages
and 622 local links/anchors. Spore completed the local User Guide build. These
results establish local projection and behavior, not deployment/publication.
Measured RSS includes allocator-retained pages and the native process's combined
fixtures; boundary catalogs carry a material resident cost. These local Linux
observations are not portable latency, memory, font-rendering or physical
power-loss durability guarantees. The exact-count/encoded-byte rejection and
ownership assertions remain unchanged. P19 automatic last-session restoration,
unrelated proposals, main promotion, installation, releases, tags and mirrors
retain their separate boundaries.
