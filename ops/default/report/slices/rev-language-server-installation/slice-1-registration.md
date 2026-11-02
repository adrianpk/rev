# Slice 1: Existing server registration

Status: reviewing
Delivery set: rev-language-server-installation
Plan: [Installation Delivery](../../../plan/rev-language-server-installation.md)
Tracker: [Installation Tracker](../../../tracker/rev-language-server-installation.md)
Branch: `feat/rev-language-server-installation-01-registration`
PR: pending

## Purpose

Register installed stdio servers and select one semantic source for any admitted
language identity through the existing configuration and command owners.

## Delivered Behavior

- `lsp-setup` guides existing-profile selection, new executable registration and
  native-only selection through M-x and modal colon prompts. Known associations
  are reused; custom identities can add basename/suffix patterns without a grammar.
- `lsp.preferred` selects a compatible server or `false`; absent choices preserve
  lexical selection. Per-server literal environment overrides merge by key and
  inherit the process environment without shell expansion.
- Setup saves and applies admitted LSP/association settings. Explicit
  `lsp-reload-settings` preserves runtime enablement, exclusions, startup profile,
  unrelated settings, unsaved text, undo, point and active mark.
- Changed sources retire old activations and stale completion results. An unchanged
  connection in another root remains usable. Semantic and native candidates stay
  separate; a grammar-free native-only source reports unavailable native syntax.

## Implementation Notes

The bounded source loader retains selected-root provenance, file/link identities
and exact source bytes. Settings publication preserves root symlinks, comments,
ordering, includes and unchanged profile representations. It validates the entire
candidate, acquires the shared OS-owned management lock and rechecks the complete
captured graph immediately before atomic publication. Invalid startup fallback
cannot authorize overwriting an invalid source. A post-publication directory-sync
failure returns the published snapshot with a durability warning.

The approved `toml_edit` parse/display dependency is pinned at 0.25.17. Existing
buffer policy, shared command/prompt, completion and connection owners apply the
restricted settings delta. No configuration watcher or second language registry
is introduced.

## Contracts Added or Changed

- INS01/INS03: generic registration and associations remain independent of bundled
  grammars and the later acquisition catalog.
- INS04: explicit compatible choice or native-only selection; preserved lexical
  fallback and activation-generation rejection of late old-server responses.
- INS05: complete-candidate admission, source-conflict rejection, root symlink
  preservation and restricted live application.
- Environment limits: 64 entries, 128-byte names, 8 KiB per value, 64 KiB total.
  Invalid environment diagnostics omit values.

## Files of Interest

- `src/config/edit.rs`, `load.rs`, `lsp.rs`, `lsp_validation.rs`, `resolve.rs`:
  admission, provenance and restricted atomic settings publication.
- `src/session/lsp/setup.rs`, `src/session/lsp.rs`, `src/lsp/settings.rs`:
  guided registration, scoped application and source retirement.
- `tests/lsp_installation/`: real filesystem boundaries, native command flows,
  controlled stdio routing and late-response coverage.
- `docs/reference/config.toml`, `defaults.md`, command/binding catalogs and
  `docs/how-to/use-language-server.md`: delivered fields and setup instructions.

## Validation

Focused development checks passed:

- `cargo check --locked`
- `cargo test --locked --test configuration lsp_ -- --test-threads=1`
- `cargo test --locked --test configuration template:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation workflows:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation routing:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation settings::rejected_settings -- --exact`
- `cargo test --locked --test lsp_installation workflows::scoped_reload -- --exact`
- `cargo test --locked --test lsp_installation settings::profile_comments -- --exact`
- `cargo run --locked --quiet --example documentation-check`
- `cargo test --locked --test commands palette::catalog_reference -- --exact`
- `git diff --check`

Both startup profiles exercise the actual command entry and cancellation at each
registration stage. Scoped reload preserves an unsaved edit, undo, point and an
active mark. Controlled stdio peers verify arbitrary-language didOpen/completion,
literal environment delivery, explicit source switching, native fallback, an
unchanged other-root connection and a response sent after source retirement.
They do not establish an upstream server's semantic quality.

The initial controller batch passed format and Clippy, then failed configuration
fallback comparisons because their expected snapshots omitted the newly retained
selected root. The expectations now compare complete default snapshots carrying
the exact selected root, preserving every existing equality assertion. Focused
`cargo test --locked --test configuration candidate:: -- --test-threads=1` and
`cargo test --locked --test configuration org_agenda:: -- --test-threads=1` passed.

The controller records the configured ordered format, Clippy, configuration/LSP/
installation regression and whitespace checks against a clean immutable HEAD.
The PR-reference follow-up head requires its own receipt before integration.
The combined delivery-set gate belongs to the controller after all slices.

## Risks and Follow-ups

Managed acquisition and real Rust/Go package evidence remain with their approved
later slices. No managed payload or acquisition helper runs in this slice.
