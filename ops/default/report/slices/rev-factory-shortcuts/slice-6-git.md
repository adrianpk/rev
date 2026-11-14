# Slice 6: Git Defaults

Status: reviewing
Delivery set: rev-factory-shortcuts
Plan: [Factory Shortcut Alignment Delivery](../../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../../tracker/rev-factory-shortcuts.md)
Branch: `feat/rev-factory-shortcuts-06-git`
PR: `#52` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/52)); open for review
Report introduction: [immutable Slice 6 report](https://forge.adrianpk.com/rev/rev/src/commit/465c097161a86c5919c49806d1b3fc1ab7e5f287/ops/default/report/slices/rev-factory-shortcuts/slice-6-git.md)

## Purpose

Deliver accepted P14-P19 and the precise History quick-access amendment
through existing native Git owners.

## Delivered Behavior

History A reviews cherry-pick, c a prepares amend against checked HEAD,
c f prepares fixup for the exact selected revision and + pages ancestry.
Old History P/F/f factory leaves are removed. Refs f/P add direct fetch and
reviewed push aliases, retaining &/M f and p/M p. Names and explicit old user
assignments remain valid. History c is prefix-only; Status c still opens
literal composition directly.

## Implementation Notes

The nine-leaf delta matches the accepted manifest exactly. The exhaustive
sample equals compiled defaults: 29 History entries and 51 Refs entries.
History title and finite-page hints follow the new routes. Native handlers,
projection identities, confirmation, source admission and budgets are unchanged.
The frozen catalog, semantic matrix, startup template and manifest are untouched;
manifest SHA-256 remains
`e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf`.

Six regressions extend the existing shared factory-shortcut filter. Both
profiles use the same isolated installed-Git fixture, whose unchanged setup
is separated from engine-only helpers for reuse. Histories have three local
commits and two-entry pages; observation keeps the existing five-second deadline.
Cases check ordered/full revision identities, range/sequence marks, checked
HEAD versus selected fixup target, native review and cancellation, retired
leaves, names/old overrides, retained paging selection, prefix/role transitions
and source/index/disk/HEAD preservation. Dirty source state is retained where
native admission permits it; cherry-pick review uses clean visiting source.

Fetch performs one minimal local bare-remote operation per profile. Its aliases
reject a selected branch in place of a configured remote. All push aliases
retain exact upstream review; Return and changed suffixes cannot accept it.
Push, composition and sequence reviews are cancelled before mutation. Existing
Git tests retain their identities, assertions and limits, with only selected
shortcut input/comment spellings adapted. All-target lint compiles those roots.

## Contracts Added or Changed

P14-P19 and the History amendment implement FSA02, FSA03, FSA05 and FSA07-FSA09
within current native ownership. Selected legacy binding rows and directly
related compatibility notes are reconciled. Original Git completion, candidate
and gate metadata retain their historical scope. Fetch remains direct selected-
remote transport; no menu, generic pull or functional-parity adapter is added.

## Files of Interest

- [Factory defaults](../../../../../src/input/keymap/defaults.rs) and
  [configuration sample](../../../../../docs/reference/config.toml).
- [History regressions](../../../../../tests/commands/factory_shortcuts/git.rs),
  [Refs and prefix guards](../../../../../tests/commands/factory_shortcuts/git/refs.rs)
  and [shared real-Git fixture](../../../../../tests/git_integration/repository_fixture.rs).
- [Git reference](../../../../../docs/reference/git.md),
  [bindings](../../../../../docs/reference/bindings.md) and
  [History guidance](../../../../../docs/tutorials/user-guide/19-git-history-and-remotes.md).
- [Selected legacy rows](../../../spec/editor-git-integration.md) and
  [accepted mapping contract](../../rev-factory-shortcuts/mapping-contract.md).

## Validation

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: PASS; workspace formatting, strict Clippy and native source policy across 1,913 files, two packages and 75 Cargo roots.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands factory_shortcuts -- --test-threads=1`: PASS; 34 tests, including six new Git regressions.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration keymaps -- --test-threads=1`: PASS; 11 inheritance, admission, normalized-key and override regressions.
- `git diff --check`: PASS.

No additional existing Git behavior gate is selected: the accepted implementation
changes input defaults and presentation hints only. The historical Git
acceptance inventory retains its frozen baseline; Slice 7 owns the new
accepted-manifest gate. Final evidence belongs to the clean HEAD after
canonical PR references.

Focused checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: PASS; 110 reachable pages, 780 local links/anchors, 707 canonical names in each combined inventory, 322 configuration entries, 10 command-reference and 12 configuration-reference regressions; all 28 sample maps equal compiled defaults.
- `cargo fmt --all --check`: PASS.

The earlier focused command
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands factory_shortcuts::git -- --test-threads=1`
passed six cases in 35.82 seconds before the final ordered-target assertion.
The final shared run above verifies the strengthened assertion. Accepted
nine-edit, artifact-link, incidental-identifier and unchanged-fixture-body
checks also pass.

Initial focused runs exposed incorrect test assumptions about read-only key
rejection, retained History refresh, the eight-chord parser bound and native
clean-source admission before cherry-pick review. Corrected setup and observations
preserve those native behaviors. The focused cases passed. The first
all-target lint found a fixture-module name collision under an existing
acceptance example; renaming that extracted module preserves fixture behavior.
The final ordered-target assertion checks each numbered full revision identity.

The ordinary target compiled cold dependencies in 3m 14s. The successful local
lint took 60.09 seconds; shared shortcut and configuration runs took 140.85
and 20.88 seconds including compilation. Documentation validation took 27.53
seconds. The staged regression commit hook took 263.47 seconds.
These are observed costs; each checkout retains its exclusive build/cache
ownership, one Cargo job, disabled incremental output and no dev/test debuginfo.
The injected tests retain their single-thread settings.

The four exact commands are recorded through controller validation after the
canonical PR references are committed. This report records local evidence;
the integrated full gate and physical terminal acceptance remain outstanding.

## Risks and Follow-ups

Physical terminal acceptance remains with Slice 7; the controller owns the
integrated full gate. Existing Git operation admission and consent remain
authoritative.
