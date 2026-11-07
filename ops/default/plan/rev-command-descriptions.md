# Command Descriptions Delivery

Status: Approved
Date: 2026-11-07
Delivery set: rev-command-descriptions
Slice strategy: behavior-first
Approved: product owner; implement the complete catalog in one slice through the established Looper workflow
Planning baseline: `c8ccf85472618bf61d8860d998d71cd39ebb1bf2`
Specification: [Command Descriptions](../spec/editor-command-descriptions.md)
Tracker: [Command Descriptions Tracker](../tracker/rev-command-descriptions.md)
Ticket: [TKT-20261106202027](../ticket/ready/20261106202027-complete-command-catalog-descriptions.md)
Active slice: Slice 1 - Complete descriptions

## Outcome and strategy

Fill the existing optional description accessor for every built-in command,
using the specification's 618 exact definitions. The existing selector already
supports name, effective shortcut and description. One vertical slice therefore
delivers the metadata, coverage guard, representative presentation checks and
documentation together. Do not split this uniform catalog pass into families.

## Prerequisites and authority

The three-column selector, effective keybinding discovery, ranking, both input
profiles, native catalogs and Markdown-derived Spore guide are delivered.
No dependency, configuration or persistent-model change is needed.

The owner's implementation instruction authorizes the preparation commit/push
on dev, the single recorded branch/worktree and canonical PR, controller-owned
Rebase + Fast-forward integration into dev, the selected gate and closure.
Looper owns execution and slice integration. Main promotion, installation,
release tags, mirrors and unrelated pending work remain separate.

Retain installed worker budgets and recovery behavior. Do not weaken checks,
add slices or introduce new product decisions to complete this metadata pass.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Complete descriptions | `feat/rev-command-descriptions-01-catalog` | `feat(commands): describe every built-in command` | `ops/default/report/slices/rev-command-descriptions/slice-1-catalog.md` |

## Slice 1: Complete descriptions

Dependency: approved committed preparation on clean dev.
Requirements: CDS01-CDS05; preserve KBH06 and existing command identity.

- T1.1: Implement the 618 specified summaries with the existing command/family
  metadata owners. Preserve canonical names, aliases, handlers and optional
  user-defined command metadata. Do not create a parallel dispatch registry.
- T1.2: Extend the existing catalog-reference guard to check exact name/text
  coverage against the specification. Add one parameterized description test
  covering M-x and modal colon, bound/unbound commands and an undescribed user
  command. Retain existing Unicode clipping, hint geometry, color, selection,
  name-only completion and ranking checks. Avoid per-command UI tests or new
  PTY machinery for static strings.
- T1.3: Update the command reference, relevant User Guide example and Unreleased
  notes, then build their Spore projection. Produce the canonical slice report,
  preserve historical delivery evidence and update ticket/index references as
  the ticket moves. Commit with conventional messages without task suffixes;
  record task/commit correspondence in the tracker or report.

## Ordered slice validation

For Cargo checks below, prepend exactly:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
The Bash guide build uses the same environment. The local configuration freezes
the full commands in this order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --lib --bin rev -- -D warnings`
3. `cargo test --locked --lib display::completion::tests -- --test-threads=1`
4. `cargo test --locked --test commands catalog_reference -- --test-threads=1`
5. `cargo test --locked --test commands description -- --test-threads=1`
6. `cargo test --locked --test display hint -- --test-threads=1`
7. `cargo run --locked --quiet --example documentation-check`
8. `bash scripts/build-user-guide.sh`
9. `git diff --check`

The description selection must run at least one meaningful parameterized test;
zero matching tests do not establish acceptance. Reuse `palette::catalog_reference`
for the complete live catalog and specification check. Tests must expose an
omitted or incorrect summary instead of only mirroring a string lookup.

## Integrated gate

Run the same ordered checks on the exact integrated clean dev candidate.
Expected cost is a few minutes with existing artifacts: static metadata, focused
native presentation checks and guide projection. This change does not justify
repeating Git workflows, Org execution, LSP acquisition or the full repository
suite. Build no optimized executable for this delivery.

## Closure

After the controller gate passes, mark the slice report delivered, the ticket
solved with resolution fixed, and this plan/tracker/specification Completed.
Update all ticket links and the pending index. The local configuration admits
only those exact documentation paths after the gate; keep runtime, tests and
generated guide inputs unchanged during closure.
