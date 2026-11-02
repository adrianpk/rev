# Slice 6: Update cancellation and removal

Status: reviewing
Delivery set: rev-language-server-installation
Plan: `ops/default/plan/rev-language-server-installation.md`
Tracker: `ops/default/tracker/rev-language-server-installation.md`
Branch: `feat/rev-language-server-installation-06-lifecycle`
PR: pending

## Purpose

Manage verified replacements, cancellation and removal through native commands,
preserving settings, ordinary editing and ownership of running processes/data.

## Delivered Behavior

- `lsp-update-server` shows a managed installation's selected owned profiles for
  confirmation. It verifies one replacement before atomically retargeting them,
  preserving user options, associations and the prior payload. Shared web profiles
  update together; custom commands/arguments and unselected profiles stay intact.
- A failed acquisition preserves the current ready server. A source conflict
  leaves the replacement ready/unselected and preserves changed user bytes.
  An unselected installation's update does not select itself implicitly.
- `lsp-remove-server` confirms an admitted installation, refuses configured
  references or a running rev client, and removes only its owned stopped payload.
  Incomplete records with absent data can be removed. Active, unmarked and legacy
  caches are preserved; marked idle acquisition caches can be cleaned.
- `lsp-cancel-installation` cancels active work while editing continues. Its slot
  and mutation locks remain owned through helper stop/reap and cleanup. Focused
  prompts cancel through C-g/modal Escape; confirmed exit requests cancellation
  before returning Exit. Interrupted candidates remain truthful and unselected.

## Implementation Notes

Update captures exact old installation facts and selected profiles whose original
command/arguments still match a managed launch. It preserves the captured user
options and rechecks the entire settings graph before the shared writer publishes
one replacement. Ready facts remain independent from selected configuration.

Running clients hold shared directory-inode leases through server reap; removal
requires an exclusive lease and rechecks the directory identity. Leases include
resolved executable aliases, server-specific PATH and literal payload paths in
arguments/environment. Other state/configuration roots cannot bypass an active
client's lease. No lock file or idle observer is introduced for activation.

Removal validates references in the selected configuration/include graph, rejects
aliased directories/escaping links, marks facts incomplete, deletes the bounded
tree, synchronizes the parent and forgets the record. Partial deletion and registry
publication failures report their actual state. Existing settings remain unchanged.
Acquisition caches carry a native ownership marker and held directory lease;
explicit cleanup requires that marker, matching identity and an idle lease.

Native command naming/dispatch and prompt cancellation remain in their existing
owners. Small domain helpers keep changed functions within source bounds. The
existing native worker, cancellation token, process-group supervisor and settings
writer remain authoritative; no runtime dependency or second selection store
was added. Confirmed-exit cancellation does not affect an aborted quit prompt.

## Contracts Added or Changed

Three commands are added to the common registry and both native entry profiles.
Inventories agree at 417 commands and 219 settings. Installation registry schema 1,
user configuration fields, analysis enablement and semantic-or-native selection
remain unchanged. Old versions remain until explicitly unreferenced and stopped.

Existing limits remain: 32 records/2 MiB registry, 256 MiB native asset, 1 GiB
payload, 2 GiB cache, 65,536 tree entries/64 levels, 1 MiB helper output, 64 KiB
diagnostic tail, 900-second operation and two-second initial stop grace followed
by bounded forced stop/reap. Cleanup runs outside the input/render loop and checks
cancellation. Unknown cache entries are preserved, including legacy job names
without the native ownership marker.

## Files of Interest

- `src/lsp/install/{update,remove,lease}.rs`: captured replacement, references,
  destructive admission and cross-process payload ownership.
- `src/lsp/install/{storage,cache,cache_cleanup}.rs`: atomic registry removal,
  durable incomplete state and bounded idle-cache cleanup.
- `src/session/lsp/lifecycle.rs`, shared command/prompt dispatch and
  `src/session/bookmarks/quit.rs`: native confirmation/cancellation and exit.
- `tests/lsp_installation/{lifecycle,lifecycle_native}.rs`: real settings,
  filesystem, process/client and both-profile workflows with controlled tools.
- Language-server how-to/reference, command catalog, bindings and Unreleased:
  user operations, retained references and cleanup constraints.

## Validation

Focused development checks passed:

- `cargo check --locked`
- `cargo test --locked --test lsp_installation lifecycle:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation lifecycle::running_removal -- --exact --nocapture`
- `cargo test --locked --test lsp_installation lifecycle_native:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation acquisition:: -- --test-threads=1`
- `cargo test --locked --test lsp_installation storage:: -- --test-threads=1`
- `cargo test --locked --lib session::quit::tests:: -- --test-threads=1`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install/update.rs --source src/lsp/install/remove.rs --source src/lsp/install/lease.rs --source src/session/lsp/lifecycle.rs --source src/lsp/install/storage.rs`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install/cache.rs --source src/lsp/install/cache_cleanup.rs --source src/lsp/install/remove.rs`
- `cargo run --locked --quiet --example source-check -- --source src/lsp/install/lease.rs --source src/session/bookmarks/quit.rs`
- `cargo run --locked --quiet --example source-check -- --source src/session/lsp.rs --source src/command/lsp.rs --source src/lsp/connection.rs --source src/session/file.rs --source src/session/prompt.rs`
- `cargo run --locked --quiet --example documentation-check`
- `cargo test --locked --test commands palette::catalog_reference -- --exact`

The four lifecycle cases cover a controlled web-package update from fixture
version 4.10.0 to 4.10.1, shared selected profiles, unchanged unselected profiles,
failed replacement, an include edit during acquisition, reference aliases and
arguments/environment, incomplete/escaping/aliased data, idle/active/unknown caches
and a real client's cross-process removal lease. The native lifecycle case runs
both startup profiles and verifies prompt cancellation, editing/point/settings
preservation, a blocked asset transfer, retained worker ownership, incomplete
candidate removal, successful update/removal and confirmed-exit group cleanup.
The helper leader is reaped and its recorded descendants no longer run. Fixtures
use owned roots and are removed after their processes stop. Native runtime was
17.83 seconds in the focused run; cold compilation is separate from that timing.

Source checks initially exposed oversized new functions, which were split along
their operation boundaries. An initial broader source invocation also included
unchanged `SettingsEdit::publish_inner` (complexity 14); its publisher logic was
not changed. The added pre-effect settings check delegates to the existing source
comparison. Current lifecycle/dispatch/storage checks pass the unchanged safe-Rust,
five-parameter, three-nesting and ten-complexity limits.

Controlled acquisition/protocol peers establish lifecycle mechanics; they do not
establish upstream semantic quality. This slice performs no new live upstream
acquisition. Existing explicit network checks remain opt-in and excluded from
ordinary regressions.

The first configured controller batch at
`bc9b5a74dba284c3c4a09b24c5b434410cc843c0`, receipt
`37MM3XAENA35U5Q3TXVI53LIBO`, passed `cargo fmt --all -- --check` and failed
`cargo clippy --locked --all-targets -- -D warnings` on one collapsible nested
conditional in keyboard-quit dispatch. The equivalent let-chain correction keeps
its behavior and source bounds. The next batch at
`3d2529b12d77a9bf936fdfdc2cbccdaacfc98d6e`, receipt
`6M4EBZLLTOG46YCWVNIXPDWTOO`, passed format and exposed a Clippy fixture
initializer warning after library lint succeeded. The fixture now initializes
its enabled/server fields together; assertions are unchanged. Fresh configured
validation and the canonical PR reference are pending. The integrated delivery-set gate remains with the controller.

## Risks and Follow-ups

A profile can keep an older payload referenced even when native-only is selected;
removal requires changing that profile as well as stopping its owning client.
Unmarked legacy cache data is retained for explicit inspection. Durability
uncertainty is reported rather than treated as rollback. Combined native/PTY
acceptance and final guidance remain in the approved final slice.
