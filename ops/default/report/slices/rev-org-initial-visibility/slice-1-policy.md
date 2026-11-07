# Slice 1: Initial visibility and drawers

Status: delivered
Delivery set: rev-org-initial-visibility
Plan: [Org Initial Visibility Delivery](../../../plan/rev-org-initial-visibility.md)
Tracker: [Org Initial Visibility Tracker](../../../tracker/rev-org-initial-visibility.md)
Branch: `feat/rev-org-initial-visibility-01-policy`
PR: `#310`
Integrated: `414fb6ac0f0321211b9b07ebf2b1c34c384e42e7`

## Purpose

Deliver OIV01-OIV05 as one vertical increment: configurable initial Org
visibility, structural document precedence, whole-view drawer controls and
explicit startup reset.

## Delivered Behavior

- Admit overview, content, showall and showeverything through
  org.startup_folded. Omission preserves the fully visible compatibility default.
- Resolve the last recognized nonliteral STARTUP token in source order,
  case-insensitively, without altering source or discarding earlier valid
  choices when unknown words occur.
- Initialize a new view once after current complete native observation.
  Explicit pending choices win; splits inherit presentation and subsequent
  choices remain window-local across buffer switches. Edits/reload retire stale
  offsets without reapplying startup.
- Hide, show or toggle valid property/custom drawer bodies independently of
  outline folds. Restore startup explicitly with the fourth named command.
  All four have complete descriptions and no factory shortcut in either profile.
- Preserve current cycling, singular drawer controls, full reveal, source
  navigation and deliberate AI REQUEST initialization. Read-only source is eligible.
- Update the configuration sample/catalog, live command-description inventory,
  Org references, existing Org User Guide chapter and Unreleased notes.
  Spore builds from canonical Markdown through the existing projection.

## Implementation Notes

Implementation commit: `83bfa87d005aac504af7c9a814e7b1403e7067cb`.

T1.1 and T1.2 share that coherent runtime/acceptance commit; T1.3 product
documentation is included there. This report and the reviewing ticket transition
complete T1.3. Canonical PR [#310](https://forge.adrianpk.com/adrianpk/rev/pulls/310) targets dev.
Report-introduction commit: `b8b5df26589b91b533af023ef1ae75eef312e9c1`.
The PR-reference commit changes delivery metadata only. Canonical PR #310 was
merged through Rebase + Fast-forward into dev at `414fb6ac0f0321211b9b07ebf2b1c34c384e42e7`.

The existing bounded observation worker resolves metadata. The visibility owner
retains heading levels and drawer bits; checked source ranges install atomically.
The existing transient live-buffer position owner retains presentation separately
for each live window, bounded by the live buffers/windows. No persistent state,
dependency, background worker or source scan during drawing is added.

Startup content includes archived descendants; existing runtime archive cycling
keeps its previous behavior. Malformed/incomplete observations do not admit
global startup/drawer ranges. Unicode, CRLF and unterminated EOF retain native
source identity and point anchors.

## Contracts Added or Changed

- org.startup_folded is a native finite enum leaf with include precedence,
  provenance, strict invalid-value rejection and the showeverything default.
- Document metadata is owned by the structural result, not by rendering.
- Explicit choices suppress pending automatic startup; reset re-resolves the
  current policy. Splits clone current presentation; reload never resurrects
  old ranges.
- The catalog contains 622 unique built-ins, including 237 OrgCommand variants
  plus org-mode. The previous 618 descriptions and historical evidence remain
  unchanged; four exact summaries extend the live inventory.
- All visibility changes preserve source bytes, dirty state, mark and editing
  history. Named selector invocations produce exactly one usage record.

## Files of Interest

- `src/config.rs`, `src/config/merge.rs`, `src/config/project.rs`: native admission.
- `src/org/visibility/startup.rs`, `src/org/visibility/state.rs`: structural policy and drawer bits.
- `src/view/visibility.rs`, `src/session/org.rs`, `src/session/contexts/positions.rs`: view lifetime.
- `src/command/org.rs`: canonical names and exact descriptions.
- `tests/org_editing/visibility*.rs`, `tests/configuration/org_visibility.rs`,
  `tests/commands/org_visibility.rs`, `src/session/ai/results.rs`: native acceptance.
- `docs/reference/org-mode.md`, `docs/tutorials/user-guide/06-org-notes.md`:
  behavior and guided use.

## Validation

Implementation preflight passed the following commands in canonical order.
The separate Looper receipt for the final clean PR HEAD, including PR-reference
commits, is authoritative; the controller owns integrated acceptance.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_editing visibility -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_editing fold -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands visibility -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands catalog_reference -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands description -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display hint -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration org_visibility -- --test-threads=1`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::ai::results::tests -- --test-threads=1`
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration reference -- --test-threads=1`
- Passed: `git diff --check`

Every selected filter executed tests: visibility 19, fold 9, command visibility 4,
catalog reference 1, descriptions 1, display hints 2 and configuration visibility 2.
Focused AI-result checks passed 6 cases and configuration-reference checks passed 11.
Documentation checked 91 reachable pages and 628 local links/anchors; Spore
strict check, link validation, rendering and anchor finishing passed.

The twelve-command slice validation receipt covers clean PR HEAD
`414fb6ac0f0321211b9b07ebf2b1c34c384e42e7`. The unchanged controller-owned
integrated gate subsequently passed at `dc31493589c9a35eff79edebe1765a76a3fa4651`.

## Risks and Follow-ups

None recorded. No external provider or owned service fixture was required.
No installation or publication was performed.


## Delivery-set closure

Status: Completed
Requirements: OIV01-OIV05 and the approved model
Canonical PR: [#310](https://forge.adrianpk.com/adrianpk/rev/pulls/310), merged into dev
Integrated implementation: `414fb6ac0f0321211b9b07ebf2b1c34c384e42e7`
Tested candidate: `dc31493589c9a35eff79edebe1765a76a3fa4651`
Gate log: `dc31493589c9a35eff79edebe1765a76a3fa4651.WGY3SYJSVJNN2WPJHND5EIZSCL.log`

The controller passed this exact unchanged command sequence on the immutable
candidate before documentation closure:

- Passed: `export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 && cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --test org_editing visibility -- --test-threads=1 && cargo test --locked --test org_editing fold -- --test-threads=1 && cargo test --locked --test commands visibility -- --test-threads=1 && cargo test --locked --test commands catalog_reference -- --test-threads=1 && cargo test --locked --test commands description -- --test-threads=1 && cargo test --locked --test display hint -- --test-threads=1 && cargo test --locked --test configuration org_visibility -- --test-threads=1 && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check`

All selected filters executed tests: 19 visibility, 9 fold, 4 command visibility,
1 catalog reference, 1 command-description, 2 display-hint and 2 configuration
visibility tests. Catalog verification passed 622 commands and 291 configuration
entries. Documentation checked 91 pages and 628 local links/anchors; the Spore
User Guide projection and build passed.

The plan, tracker, specification and model are Completed. The source ticket is
[solved as fixed](../../../ticket/solved/20261103111614-configure-org-drawer-visibility.md)
with its original lifecycle timestamps, branch, PR and real commit evidence.
The earlier AI presentation ticket links to that solved follow-up.

Only the injected closure documents differ after the tested candidate. Runtime,
tests, required assertions, command definitions, guide inputs, canonical slice
identity and the ordered gate remain unchanged. No gate was rerun during closure;
no owned test fixture required shutdown.
