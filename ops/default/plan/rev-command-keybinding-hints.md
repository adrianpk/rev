# Command Keybinding Hints Delivery

Status: In progress
Date: 2026-11-06
Delivery set: rev-command-keybinding-hints
Slice strategy: behavior-first
Approved: product owner; continue the established preparation commits, plan, slicing, push and one autonomous Looper delivery through dev
Planning baseline: `421ab4b62604`
Specification: [Command Keybinding Hints](../spec/editor-command-keybinding-hints.md)
Tracker: [Command Keybinding Hints Tracker](../tracker/rev-command-keybinding-hints.md)
Concern: [Pending Work Index](../README.md)
Ticket: [TKT-20261031095151](../ticket/reviewing/20261031095151-show-keybindings-beside-m-x-commands.md)
Active slice: Slice 2 - Acceptance and guidance

## Outcome and strategy

Show one effective shortcut beside bound command names in M-x and the shared
modal colon selector. Capture the originating input owner before command entry.
Use admitted runtime maps and the existing dispatch rules, not a second binding
catalog. Preserve name-only filtering, recent/frequent ranking, completion,
invocation, Visual selection and canonical usage identities.

Two behavior-first slices deliver the usable annotation, then joint acceptance
and guidance. No additional slice is delegated. Hints introduce no dependency,
configuration key, persistence change or model companion.

## Prerequisites and authority

Vertical completion, effective configurable maps, both editing profiles,
context-local dispatch, registered AI commands, command ranking and the
Markdown-derived Spore guide are delivered prerequisites. Rust/Cargo, Bash,
existing native surface/PTY helpers and Spore provide the validation tools.

The owner's continuation authorizes preparation commits on dev, the two recorded
slice branches/worktrees and canonical PRs, controller-owned Rebase + Fast-forward
integration into dev, the selected final gate and documentation closure. Looper
is the execution owner and merges slice PRs without waiting for owner approval.
Main promotion, installation, tags, releases and mirrors remain separate.

Native recovery may diagnose six technical incidents, at most two per family,
retry two verified environmental prerequisites and integrate at most two scoped
final-gate repair PRs. Preserve the same run, frozen commands, conversation,
consumed budgets and source authority. Dependency changes, new product decisions,
weaker assertions and extra slices are not delegated. Retain the installed
defaults of 200 worker turns and five stalled turns.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Effective hints | `feat/rev-command-keybinding-hints-01-effective` | `feat(commands): show effective keybinding hints in command selectors` | `ops/default/report/slices/rev-command-keybinding-hints/slice-1-effective.md` |
| Slice 2 | Acceptance and guidance | `test/rev-command-keybinding-hints-02-acceptance` | `test(commands): verify keybinding hints and document selector behavior` | `ops/default/report/slices/rev-command-keybinding-hints/slice-2-acceptance.md` |

## Slice 1: Effective hints

Dependency: approved committed preparation on clean dev.
Requirements: KBH01-KBH05, with UI02 and existing CMR01-CMR05 behavior preserved.

- T1.1: Expose the smallest read-only inspection needed from effective maps and
  the actual resolver. Capture the originating profile/state/context, including
  terminal ownership, before the minibuffer takes focus. Prepare a bounded
  command-identity snapshot using admitted sequences of at most eight chords.
- T1.2: Annotate shared selector rows with one normalized shortest-then-lexical
  hint. Respect local precedence, overrides and input consumption. Use existing
  palette roles, full-row selection and cell geometry; omit a hint that cannot
  fit completely after the full name and a two-cell minimum gap. Preserve
  filtering, ranking, Tab, Return, cancellation and usage identity.
- T1.3: Extend meaningful existing commands/modal/display and keymap coverage
  for origin capture, overrides, normalization, unbound commands, narrow layout
  and dispatch agreement. Use the changelog-normalize skill for one Unreleased
  user-visible outcome. Record the implementation report and ticket evidence.

## Slice 2: Acceptance and guidance

Dependency: Slice 1 integrated. Requirements: KBH01-KBH05 together.

- T2.1: Add `examples/command-keybinding-hints-acceptance.rs`, helpers under
  `examples/command_keybinding_hints_acceptance/` as needed, and
  `scripts/check-command-keybinding-hints.sh`. Reuse existing surface and PTY
  support. Exercise the detailed matrix natively and one bounded real
  input/render journey per profile, rather than multiplying every context
  through PTYs. Keep tests independent of actual user state and credentials.
- T2.2: Update the command reference and existing search-and-commands User Guide
  chapter with one annotated row and the primary-hint/absent-hint rules. Update
  binding/prompt references only where changed behavior requires it. Project
  Markdown into Spore and reconcile the interaction baseline UI02 before the
  gate, preserving prior delivery evidence.
- T2.3: Record real reports, immutable introductions and task/commit/PR evidence.
  Prepare a clean integrated candidate for the controller-owned frozen gate.
  Complete the ticket and delivery metadata only after the selected gate passes.

## Acceptance boundaries

Use native parameterized cases for factory/configured bindings, inherited and
shadowed leaves/prefixes, Org/Git/tree owners, terminal input, modal states,
normalized Meta aliases, multiple sequences, unbound and registered AI names.
Compare each advertised sequence with actual dispatch through the same owner.
Include identity/ranking/filtering independence, Tab/Return, scrolling,
selection, Unicode widths, narrow/zero geometry, resize and NO_COLOR.

Build outside journey deadlines. Each real editor journey has a 30-second outer
deadline and a 1 MiB observed-output limit. Reuse existing stricter helper limits;
do not relax them. Wait for observable output instead of fixed sleeps, restore
terminal attributes and reap only owned children. Do not require an external
AI provider, language server, live repository mutation or private user files.

## Slice validation

For every Cargo command except formatting, prepend exactly:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
The local configuration records this prefix literally.

| Slice | Required commands in order, with the Cargo prefix above |
| --- | --- |
| Slice 1 | `cargo fmt --all -- --check`; `cargo clippy --locked --lib --bin rev -- -D warnings`; `cargo test --locked --test commands --test modal --test display -- --test-threads=1`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |
| Slice 2 | `cargo fmt --all -- --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked --test command_usage -- --test-threads=1`; `bash -n scripts/check-command-keybinding-hints.sh`; `cargo run --locked --quiet --example documentation-check`; `git diff --check`. |

Run changed native cases during implementation. The combined acceptance script
belongs to the final gate. Avoid repeated historical gate chains or an all-target
test sweep. Do not disable tests, weaken assertions or change unrelated behavior.

## Final gate

Run once on the exact clean integrated dev candidate, after both canonical PRs
are merged and reports delivered. Only the controller owns this combined gate.
Verify target admission, tool availability, Bash syntax, guide inputs and output
access during preparation. The acceptance entrypoint is a Slice 2 obligation.

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test commands --test modal --test display --test command_usage -- --test-threads=1
cargo test --locked --test configuration keymaps -- --test-threads=1
bash scripts/check-command-keybinding-hints.sh
cargo run --locked --quiet --example documentation-check
bash scripts/build-user-guide.sh
git diff --check
```

This gate protects changed key resolution, shared selection, frame rendering,
usage identity, configuration admission and Markdown/Spore guidance. The preceding
ranking delivery measured about 309 seconds for commands/modal/display tests and
3.5 seconds for usage tests, plus roughly 102 seconds for Clippy and test
compilation. Selecting configuration keymaps avoids repeating unrelated
configuration tests. Expect approximately 7-10 minutes with reusable artifacts;
this is an estimate, not a deadline or a passing result. Record actual stage
timings in the final report. Cold compilation may take longer.

## Ticket lifecycle and closure

Move the source ticket from ready to in_progress when Slice 1 starts and to
reviewing when its canonical review is prepared. Keep it reviewing through
Slice 2 and the final gate. Keep the spec, index, plan and tracker links aligned
with every move. Populate actual lifecycle timestamps, branch, PR and commits;
do not invent implementation evidence during preparation.

After the controller passes the exact candidate, move reviewing to solved with
resolution fixed, link both delivered reports and record gate evidence. Pin both
old and new ticket paths before execution, because a rename changes two paths.
Only these eight exact Markdown paths may change after the successful gate:

- `ops/default/plan/rev-command-keybinding-hints.md`
- `ops/default/tracker/rev-command-keybinding-hints.md`
- `ops/default/spec/editor-command-keybinding-hints.md`
- `ops/default/README.md`
- `ops/default/report/slices/rev-command-keybinding-hints/slice-1-effective.md`
- `ops/default/report/slices/rev-command-keybinding-hints/slice-2-acceptance.md`
- `ops/default/ticket/reviewing/20261031095151-show-keybindings-beside-m-x-commands.md`
- `ops/default/ticket/solved/20261031095151-show-keybindings-beside-m-x-commands.md`

Complete the plan, tracker, specification and index together. Retain the tested
candidate, receipt, exact commands, immutable report introductions, actual
measurements and local evidence limits. Runtime, tests, guides, changelog and
acceptance assertions must remain unchanged after the gate.
