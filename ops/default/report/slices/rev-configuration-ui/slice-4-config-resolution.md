# Slice 4: Resolve layered settings

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-04-config-resolution`
PR: [#53](https://forge.adrianpk.com/adrianpk/rev/pulls/53)

## Purpose

Resolve validated configuration layers against compiled defaults and prepare
one coherent snapshot, preserving source precedence and language inheritance.

## Delivered Behavior

- Merges partial concern and language tables by present leaf, preserving
  absence and replacing explicit false, zero and empty arrays correctly.
- Applies the loader's ordered includes before parent/root leaves. Language
  editing overrides apply after the final global settings, retaining omitted
  global siblings independently of bundled grammar availability.
- Projects every accepted scalar and RGB role into typed settings while
  retaining the source of each explicitly assigned leaf and mapping entry.
- Returns an immutable snapshot after source and merged-candidate validation.
  Failed preparation yields complete built-ins and one actionable diagnostic,
  with no rejected settings, language overrides or origins retained.
- Keeps prepared snapshots fixed when files change. Explicit preparation uses
  the selected user root and literal includes, without project discovery.

## Implementation Notes

- T4.1: `88a4185b3e0ce8062c6de09531106f08bf1d389f` adds typed partial
  merging, complete projection, language policy resolution and source origins.
- T4.2: `3a6b77457c047978871d7a01934fb39c5e114792` adds atomic preparation
  outcomes and tests for rollback, revalidation and fixed snapshots.
- T4.3: `5b7a40e088cf87acc0feaa00e4a66e6e98d5d7da` adds complete catalog
  projection and explicit-source isolation coverage
  and this report. All 14 new focused configuration tests pass.
- Fixed concern records and small Option helpers keep merge and projection
  explicit. `Settings::default()` remains the single compiled-default owner.
  Numeric conversion and RGB decoding occur only after admission succeeds.
- Include arrays remain source-local traversal metadata. Partial overlays
  replace those arrays, but settings resolution never rescans or concatenates
  include paths after the loader has established source order.
- Snapshots own settings and source paths, with private fields and shared
  accessors. Origins map decoded key components to source occurrences; literal
  periods in association patterns remain part of one key component.
- Revalidation errors retain source and key. Original parser diagnostics retain
  available byte spans and scalar positions; later validation does not invent
  locations for altered in-memory source records.

## Contracts Added or Changed

- `Document::overlay` merges one partial layer and returns assigned leaf paths
  for provenance. Empty tables preserve inherited mapping entries.
- `config::resolve(&Sources)` validates complete occurrence coverage, every
  source and the merged document before returning a `Snapshot`.
- `Snapshot::settings` provides shared typed settings access. `editor` resolves
  a selected language over final globals; `grammar` selects the admitted
  bundled mode; `origin` identifies the last explicit assigning source.
- `config::prepare(&Path)` returns one `Startup` outcome with a valid snapshot
  or complete compiled defaults plus one diagnostic. Preparation performs no
  session installation, source repair or command execution.

## Files of Interest

- `src/config/merge.rs`: present-leaf/table merging and assigned paths.
- `src/config/project.rs`: complete typed projection and language editor policy.
- `src/config/resolve.rs`: snapshot admission, source order and leaf origins.
- `src/config/startup.rs`: complete-candidate preparation and fallback.
- `tests/configuration/merging.rs`, `resolution.rs`, `candidate.rs`: precedence,
  source provenance, candidate rejection and snapshot stability.
- `tests/configuration/resolution_contract.rs`: all scalar and palette roles,
  false/zero/map inheritance and isolated project-CWD source selection.

## Validation

Development checks:

- `cargo test --test configuration merging::`: passed; three tests.
- `cargo test --test configuration resolution::`: passed; two tests.
- `cargo clippy --lib --test configuration -- -D warnings`: passed.
- `cargo test --test configuration candidate::`: passed; five tests.
- `cargo test --test configuration resolution_contract::`: passed; four tests,
  including one isolated child process with its own CWD and environment.

The controller recorded all five ordered checks passing on clean head
`100d9a7d939b96a16e8309abba7af2c9705dc6e3`, validation
`42OENLRHMME4XSVSRQF6H2QUUO`, including 49 configuration tests. Canonical PR #53
integrated that exact head into dev by Rebase + Fast-forward.
Required commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Filename-to-language selection remains assigned to Slice 5. Context-specific
keymap normalization and conflict admission remain assigned to Slice 17.
Runtime startup activation remains assigned to Slice 20.
