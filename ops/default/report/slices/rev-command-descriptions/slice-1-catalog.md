# Slice 1: Complete descriptions

Status: delivered
Delivery set: rev-command-descriptions
Plan: [Command Descriptions Delivery](../../../plan/rev-command-descriptions.md)
Tracker: [Command Descriptions Tracker](../../../tracker/rev-command-descriptions.md)
Branch: `feat/rev-command-descriptions-01-catalog`
PR: `#307`
Integrated: `bc32ff49587305185702c499010678c742795d19`

## Purpose

Deliver CDS01-CDS05 by completing the existing command description metadata
without changing command identities, handlers, bindings or selector layout.

## Delivered Behavior

- All 618 built-ins return the exact approved single-line summary, including
  unbound actions and the native Git, Org, terminal, modal and context catalogs.
- M-x and modal colon display those summaries through the existing shared
  three-column selector. User-defined AI commands retain absent metadata and
  blank description cells; saved instructions never become descriptions.
- Name-only matching and Tab completion remain intact. Existing clipping,
  effective shortcut precedence, full-row selection and palette roles remain
  owned by the existing selector.

## Implementation Notes

T1.1: `9ea81860949a33f592cb03c8cafa72764d38d0e5` populates the existing
command and family metadata owners.
T1.2: `812e42fc4319b4a624b9111a264e2906f29504f0` extends the existing catalog
guard and native selector checks.
T1.3: `489f3641fa50c17e5b965d1cba92a1452c2b5b2a` updates the command
reference, Markdown User Guide, Unreleased notes and this report. Its commit is
the report's immutable introduction. Canonical
[PR #307](https://forge.adrianpk.com/adrianpk/rev/pulls/307) merged into `dev`
at `bc32ff49587305185702c499010678c742795d19` on 2026-11-07.

The public optional accessor delegates static summaries to the existing family
owners. Parameterized requests use their canonical action summary. No parallel
dispatch registry, dependency, configuration, persisted state or drawing I/O is
introduced. The Org summary array follows its existing name-array ordering.

## Contracts Added or Changed

- Every live built-in must match exactly one nonempty, single-line definition in
  the approved specification. The extended catalog-reference test rejects
  missing, duplicate, unknown or incorrect summaries and catalog drift.
- One parameterized native selector test covers both entry profiles, bound and
  unbound built-ins, a context-local action, undescribed AI metadata, name-only
  matching/completion and source immutability.
- Existing hint regressions now inspect populated description cells while
  retaining exact/narrow/zero widths, Unicode shortcut fit, selection colors,
  trailing cells and invocation/ranking checks.

## Files of Interest

- `src/command.rs`, `src/command/*.rs`: canonical command/family metadata.
- `src/buffer/mode.rs`, `src/buffer/whitespace.rs`,
  `src/buffer/edit/history.rs`: summaries beside the existing action names.
- `tests/commands/palette.rs`, `tests/commands/editor_keymaps.rs`,
  `tests/display/ranking.rs`: catalog, profile and presentation guards.
- `docs/reference/commands.md`,
  `docs/tutorials/user-guide/04-search-and-commands.md`, `CHANGELOG.md`:
  complete-catalog behavior and exact examples. Markdown remains the Spore source.

## Validation

All nine ordered implementation commands passed locally:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib display::completion::tests -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands catalog_reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands description -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display hint -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

The selected tests passed six tests: two completion unit tests, the full
618-command catalog guard, one eight-case profile/metadata test and two hint
geometry/color tests. Documentation validation passed 618 commands in each
combined inventory, 289 configuration entries, 91 reachable pages and 623 local
links/anchors. The Spore build projected and rendered all 91 Markdown pages
under ignored build output. No preview server or external publication was used.

The controller validator passed all nine ordered commands on the exact clean
PR-reference HEAD `bc32ff49587305185702c499010678c742795d19`, which is also the
verified integrated commit. This evidence covers the focused slice checks.
The controller owns the integrated gate on the next clean dev candidate after
these delivery entries are committed; that gate and delivery-set closure remain
pending.

During authoring, the catalog guard rejected wording altered during metadata
generation; the exact approved summaries were restored. The hint-color check
also rejected its former blank-column expectation; it now verifies the existing
secondary-text role through the final terminal cell. Focused reruns passed.

## Risks and Follow-ups

No unresolved implementation risk is recorded. Slice 1 is integrated and its
delivery entries are closed. The ticket and approved delivery artifacts remain
open until the controller-owned integrated gate passes and final closure is
authorized by the current phase.
