# Slice 2: Catalog and guided setup

Status: reviewing
Delivery set: rev-language-server-installation
Plan: [Installation Delivery](../../../plan/rev-language-server-installation.md)
Tracker: [Installation Tracker](../../../tracker/rev-language-server-installation.md)
Branch: `feat/rev-language-server-installation-02-catalog`
PR: pending

## Purpose

Make configured semantic sources, installed recipe presets and optional acquisition
prerequisites discoverable without restricting custom language identities.

## Delivered Behavior

- `lsp-servers` opens a native read-only buffer snapshot with selected sources,
  configured profiles, local executable availability and finite recipe metadata.
  It separates native-only selection, native fallback, external availability,
  managed recipe prerequisites and unverified managed payload/version facts.
- `lsp-setup` suggests known associations, configured identities and recipe
  languages while accepting other valid identities. Installed `recipe:<profile>`
  presets reuse observed executables and offer structured arguments/markers.
- `lsp-select-server` persists a compatible configured profile or native-only
  choice through the same M-x/colon prompt owner. It asks for patterns only when
  the selected identity has no known association.
- Presets retain selected initialization options, settings, literal environment
  values and root markers. Known builtin/user associations stay intact. A
  configured preset-named profile keeps its authoritative command and arguments.

## Implementation Notes

The read-only `lsp::catalog` contains eight approved recipes and ten launch
profiles, including separate HTML/CSS/JSON profiles and no managed Org recipe.
Recipe acquisition metadata targets Linux x86-64. Generic external registration
and language selection remain independent of that catalog and target.

Availability observes local file metadata and at most 128 PATH entries. Relative
launch/search paths report that a client root is required. Observation never runs
an executable, discovers remote versions or parses installation records. Managed
payload readiness/version remains explicitly unverified. The source selector
continues to work with absent or corrupt installation records.

Snapshots use the existing canonical buffer, unique-name, read-only and view
owners. They display at most 128 language rows and 32 admitted configured profiles,
with a UTF-8-safe 64 KiB visible result ceiling. Each explicit invocation produces
a normal result snapshot that can be closed or left through buffer commands.
Environment values are omitted. Existing setup persistence and scoped live apply
retain ownership of source conflicts, runtime mode and configuration admission.

## Contracts Added or Changed

- INS01: native discovery, installed presets and actionable unavailable-executable
  or missing-prerequisite results.
- INS03: recipe/grammar-independent identities and source selection; the catalog
  does not become a language registry or a configuration/installation store.
- INS04: explicit compatible-source/native-only selection retains the existing
  semantic/native policy and authoritative configured command.
- Public commands: `lsp-servers`, `lsp-select-server`; `lsp-setup` adds installed
  recipe shortcuts while retaining the custom executable path.

## Files of Interest

- `src/lsp/catalog.rs`, `src/lsp/catalog/inventory.rs`: finite metadata and bounded
  local observations.
- `src/session/lsp/setup.rs`, `src/session/lsp.rs`: shared selectors and native
  read-only inventory snapshots.
- `src/config/associations.rs`, `src/buffer/lifecycle.rs`: existing association
  suggestions and canonical result buffers.
- `tests/lsp_installation/catalog.rs`, `selectors.rs`: filesystem observations,
  both-profile interactions, registry independence and result bounds.
- Existing command/binding catalogs, language-server reference/how-to and
  Unreleased notes describe the delivered commands and observation limits.

## Validation

Focused development checks passed:

- `cargo check --locked`
- `cargo test --locked --test lsp_installation catalog:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation selectors:: -- --test-threads=1`
- `cargo test --locked --test commands palette::catalog_reference -- --exact`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The native selector journey runs once per startup profile with owned XDG/PATH
roots. Executable fixture files record any invocation; none runs. Tests cover
language/profile/pattern and preset argument/marker cancellation, unknown and
incompatible profiles, a missing preset, inherited options and associations,
preset-named custom command preservation, arbitrary uncatalogued identities,
corrupt-registry preservation, read-only snapshots, omitted environment values
and UTF-8-safe visible output limits. Controlled metadata/interaction fixtures do
not establish an upstream server's protocol or semantic quality.

The controller owns the exact clean-HEAD receipt for the configured ordered
format, all-target Clippy, command/installation regressions and whitespace checks.
The PR-reference follow-up head requires its own receipt before integration.
The combined delivery-set gate remains with the controller after all slices.

## Risks and Follow-ups

Managed acquisition, verified installation facts and real package probes remain
with their approved later slices. Executable presence and prerequisite metadata
are observations, not verification of a managed payload or running LSP client.
