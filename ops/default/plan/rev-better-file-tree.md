# Better File Tree Delivery

Status: Approved
Date: 2026-11-06
Delivery set: rev-better-file-tree
Slice strategy: behavior-first
Approved: product owner; specification/model, preparation commits, three-slice plan, push and one autonomous Looper delivery through dev
Planning baseline: `9076cab4c5bc`
Specification: [Better File Tree](../spec/editor-better-file-tree.md)
Model: [Better File Tree Model](../spec/editor-better-file-tree-model.md)
Tracker: [Better File Tree Tracker](../tracker/rev-better-file-tree.md)
Concern: [Pending Work Index](../README.md)
Active slice: Slice 3, Acceptance and guidance

## Outcome and strategy

Deliver the approved richer file tree in one retained Looper run with three
slices. `tree.decorated` defaults to true; explicit false preserves the current
minimal labels. The existing `gutter.nerd_fonts` selects fixed ordinary or Nerd
Fonts symbols for decorated entries. Its factory default stays false. Root,
directory, source, document, configuration, generic-file and symlink roles use
the approved entry palette. Existing tree visibility intent remains unchanged.

The behavior-first strategy gives Slice 1 a usable decorated ordinary-symbol
presentation, Slice 2 completes the font choice and shared terminal geometry,
and Slice 3 validates the combined interaction and completes guidance. Every
slice must compile and pass its checks. Do not add a fourth tuning slice.

Keep tree observations, root selection, sorting, lazy expansion, explicit
refresh, canonical entry identity, click policy and both interaction profiles.
Persistent selection and active-buffer following retain their separate ticket;
the screenshot's multiple roots do not introduce a workspace abstraction.

## Prerequisites and authority

Native configuration composition, file/directory navigation, pointer input,
panel geometry, both interaction profiles, the fixed Nerd Fonts preference and
Markdown-derived Spore guidance are delivered prerequisites. Use Rust/Cargo,
Bash, existing PTY helpers and Spore; no new package or runtime is planned.

The startup request authorizes preparation on dev, the three named slice
worktrees and canonical PRs, controller-owned Rebase + Fast-forward merges into
dev, the selected final gate and documentation closure. Looper owns progress,
merges and the final gate. Do not wait for owner merges of slice PRs or start a
second controller. Main promotion, release, installation and mirror publication
are separate operations.

Native recovery may diagnose six technical incidents, at most two per family,
retry two verified environmental prerequisites and integrate at most two
final-gate repair PRs within this specification. Keep the same run, consumed
budgets, worker conversation, frozen checks and source authority. Dependency
changes, new product decisions, weaker assertions and additional slices are
not delegated. Retain the controller defaults of 200 worker turns and five
stalled turns. Resource recovery does not authorize deleting unrelated files.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Decorated presentation | `feat/rev-better-file-tree-01-decoration` | `feat(tree): add decorated and minimal presentations` | `ops/default/report/slices/rev-better-file-tree/slice-1-decoration.md` |
| Slice 2 | Font icons and geometry | `feat/rev-better-file-tree-02-icons` | `feat(tree): add fixed Nerd Fonts icons and cell-safe labels` | `ops/default/report/slices/rev-better-file-tree/slice-2-icons.md` |
| Slice 3 | Acceptance and guidance | `test/rev-better-file-tree-03-acceptance` | `test(tree): verify decorated presentation and document modes` | `ops/default/report/slices/rev-better-file-tree/slice-3-acceptance.md` |

## Slice 1: Decorated presentation

Dependency: approved committed preparation on clean dev. Requirements: BFT01,
BFT03 and the ordinary-symbol portion of BFT02/BFT04.

- T1.1: Add the startup boolean and seven entry foreground roles through typed
  configuration, merge, provenance, validation, defaults and the complete sample.
  Keep the tree background scalar and existing selection colors compatible.
- T1.2: Derive the fixed entry kind from observed type/name, with root, symlink
  and directory precedence and a generic fallback. Render ordinary symbols,
  disclosure and semantic colors by default. Explicit false keeps current
  minimal labels, independent of the existing font flag. No new filesystem
  reads, language analysis, Git process or entry state is introduced.
- T1.3: Extend existing tree/configuration/display coverage for omitted, false,
  true, invalid and composed settings; compiled kinds, palette overrides and
  minimal compatibility. Update the sample, defaults and reference for the
  delivered ordinary-symbol behavior. Add one reader-facing Unreleased outcome
  using the changelog skill; do not advertise the pending icon path as delivered.

## Slice 2: Font icons and geometry

Dependency: Slice 1 integrated. Requirements: BFT01-BFT04 and model.

- T2.1: Select the approved Nerd Fonts catalog through `gutter.nerd_fonts` only.
  Apply one checked prefix geometry for both catalogs and preserve the existing
  two-cell nesting step, root disclosure, slashes and incomplete diagnostics.
  No font probing, arbitrary icon configuration or new dependency is added.
- T2.2: Use the same row identity and terminal-cell boundaries for painting,
  clipping and pointer input. Preserve full-row activation/selection policy,
  current focus-sensitive selection, color precedence and NO_COLOR behavior.
  Avoid changing navigation or the active-file synchronization ticket.
- T2.3: Verify all four boolean combinations, both symbol catalogs, glyph
  widths, source-independent classification, Unicode names, narrow/zero space,
  scrolling, resize, cached hits and gutter font behavior. Prefer parameterized
  cases in existing suites. Update the reference and Unreleased outcome with
  the completed font choice; do not duplicate the product outcome.

## Slice 3: Acceptance and guidance

Dependency: Slice 2 integrated. Requirements: BFT01-BFT04 together.

- T3.1: Add a bounded native Rust acceptance example at
  `examples/better-file-tree-acceptance.rs`, its helpers under
  `examples/better_file_tree_acceptance/` and Bash entrypoint
  `scripts/check-better-file-tree.sh`. Reuse existing frame/surface and PTY
  utilities. Cover the complete presentation/configuration matrix natively;
  run one short real input/render/resize/shutdown journey per interaction
  profile. Do not multiply PTYs across the entire matrix.
- T3.2: Complete Diataxis reference/defaults/migration guidance and the existing
  file-tree User Guide chapter, using short examples and a useful text preview.
  Keep the Spore projection derived from Markdown. Reconcile directly affected
  configuration/navigation/font-use descriptions before the gate, preserving
  their historical completion evidence and unrelated requirements.
- T3.3: Complete actual slice reports and task/commit/PR traceability, prepare
  the clean integrated candidate for the controller and close only after its
  frozen gate passes. Record timings and evidence limits. Do not create report
  placeholders during planning or run the full gate inside a worker turn.

Nerd Fonts acceptance proves selected/emitted code points and cell geometry,
not the appearance of an installed physical font. Any new Rev playground uses
isolated `gutter.nerd_fonts = true`; factory defaults remain unchanged.

## Slice validation

For all Cargo commands except formatting, use the existing low-debug prefix:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
Record this prefix literally in the local controller configuration.

| Slice | Required commands in order, with the Cargo prefix above |
| --- | --- |
| Slice 1 | `cargo fmt --all -- --check`; `cargo clippy --locked --lib --bin rev -- -D warnings`; `cargo test --locked --test tree --test configuration --test display -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 2 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test tree --test display --test pointer --test panels --test gutter -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 3 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test tree --test configuration -- --test-threads=1`; `bash -n scripts/check-better-file-tree.sh`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |

Adjust old label assertions only where the approved default intentionally
changes them. Use explicit minimal settings for tests about that presentation;
retain separate default-decorated assertions. Do not disable or weaken tests
to accommodate new output.

## Final gate

Execute the following once on the clean integrated candidate, in order. The
environment applies to all later Cargo and script invocations. The new script
is a Slice 3 obligation, not an existing planning check.

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test tree --test configuration --test display --test pointer --test panels --test gutter -- --test-threads=1
bash scripts/check-better-file-tree.sh
cargo run --locked --quiet --example documentation-check
bash scripts/build-user-guide.sh
git diff --check
```

The acceptance script builds the editor/driver once, uses owned HOME/XDG
fixtures, finite output and process deadlines, and verifies terminal restoration
and owned cleanup. Production tree bounds remain unchanged: 4096 observed
entries and 512 KiB raw filename bytes per expanded directory; painting and
ordinary movement consume cached observations only. Classification adds a
bounded compiled catalog and small derived row data, not an index or service.

The six suites protect tree interaction, configuration, renderer semantics,
pointer coordinates, panels and the shared font setting's gutter path. Two PTY
journeys complement the native matrix. Do not chain historical terminal, Git,
LSP, Pi, Org or completion acceptance scripts into this gate.

The previous Better Gutter final gate took 175.58s with cached artifacts and
one-job low-debug builds; that is historical context, not a timing claim for
this set. Expect minutes with build reuse and additional cold compile cost per
worktree. Record actual durations and investigate unexpected cost without
silently raising timeouts or repeating unrelated exhaustive suites.

## Closure

Use Conventional Commits without task-ID suffixes and the established Summary,
Changes, Validation PR body. Each slice report is the last Changes bullet,
linked through its immutable introduction commit. Validation lists successful
commands in the configured order, with whitespace last.

After the final gate, closure may update only this plan, tracker, pending index,
three delivered reports and this feature's specification/model. These eight
exact Markdown paths are pinned in the local configuration. Source, guides,
samples, changelog and baseline reconciliation must precede the tested candidate.

Completion requires all three PRs integrated, all reports delivered, BFT01-BFT04
and the model implemented, the exact candidate green under the unchanged gate,
and completed operational artifacts. No release, installation, guide deployment
or public mirror push is implied by delivery through dev.
