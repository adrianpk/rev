# Slice 7: Contextual keybindings

Status: delivered
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-07-keymaps`
PR: `#7`
Integrated: 92646729268ccbc15ea7bf432c45b60d5870eb4a through canonical PR #7

## Purpose

Deliver T7.1-T7.3: resolve passive and generation-owned contextual bindings through
native maps, with buffer scope, explicit user precedence and deterministic conflict
inspection in both startup profiles.

## Delivered Behavior

- Passive manifest defaults are admitted without starting programs. A matching
  key calls the qualified command through ordinary native dispatch and lazy
  activation. Runtime keymaps.bind/unbind manages global and explicit text-buffer
  contributions owned by the caller's process generation.
- Buffer scope precedes global scope. Within each scope, explicit user choices
  precede plugin defaults, which precede factory defaults. Native user maps and
  current user macros remain authoritative within their global scope.
- Persistent buffer selectors intersect native filetype and package provenance.
  An unrelated source does not inherit a result buffer's local binding. Global
  user customization cannot silently suppress a more local plugin default.
- Equal-priority conflicting defaults suppress every competing assignment.
  Installation, activation and RPC arrival order cannot select a winner. A
  conflicted local layer reveals the next applicable global/native policy.
- Native sequence normalization and admission protect prefix reachability,
  cancellation, command discovery, Escape/Meta recovery, macro abort, modal
  transitions and count grammar. Native prompts, popups, transient resources,
  terminal/tree focus and capture/source-edit owners retain their priority.
- Dispatch and command hints share the same effective native maps. Shadowed or
  conflicting suggestions do not become effective palette hints. plugin-list
  shows normalized applicable contributions, contexts, scopes, provenance and
  effective/shadowed/conflicting resolution for the current text buffer.
- Bind responses inspect explicit hidden targets without moving focus and return
  effective, underlying and competing sources. Unknown, duplicate or foreign
  unbind identities refuse the complete batch.
- Closing a target removes its local registrations. Retired generations lose
  their runtime contributions and configured defaults; new activation can restore
  configured defaults. Explicit user choices persist. Removal recomputes current
  policy instead of restoring an old saved mapping.

## Implementation Notes

Implementation: `ef082253d8093bf1af5041349b943be1c99ae378` implements T7.1-T7.3 and introduces
this report. Canonical [PR #7](https://forge.adrianpk.com/rev/rev/pulls/7) targets dev.

The existing native Map owns normalized sequence identity, prefix admission and
resolved command entries. Its contribution resolver processes local/user priority
before lower scopes and excludes conflicting peers before selecting fallback.
There is no plugin dispatch engine or separate key notation. Input's active-map
selection and existing hint probe both consume the resulting native maps.

Session retains only bounded generation-qualified registration metadata and
matches persistent selectors against native text owners. Resolved maps are cached
by source identity/filetype, contribution changes and lifecycle observations.
Ordinary key lookup retains the existing native map path; active-idle packages
introduce no polling wakeup. Current macro catalog changes and ordinary Org roles
recompute from their native owners. Native transient input does not consume the
plugin map.

Static admission validates merged native user maps and overlapping explicit
plugin-user selectors before publishing a configuration snapshot. Dynamic
admission validates package ownership, declared command identity and live text
handles before installation. The registration serial remains monotonic across
unbinds and restarts. Existing bidirectional request permits, generation retirement
and finite native service scheduling still apply.

## Contracts Added or Changed

- The connected host advertises keymaps and implements keymaps.bind/unbind.
  SDK wire types and version remain unchanged.
- Manifest and user binding declarations now participate in native resolution.
  Overlapping explicit user choices in one applicable scope reject startup;
  competing defaults remain inspectable without selecting a winner.
- Runtime bindings require the caller's declared command and a current connection
  generation. Buffer scope takes a native opaque text handle, never a path/name.
- Passive plus dynamic contributions remain bounded to 128 per package and
  512 per editor. A user configuration admits at most 512 selector overrides.
- Unbind accepts at most 128 registrations and validates the complete batch before
  removal. Hidden-target bind inspection uses the explicitly requested source.
- Inspection lists are bounded to 512 sources with explicit truncation notice;
  plugin-list retains its 64-KiB display ceiling. Resolution is per declared
  native editing context; effective shortcut hints use the active context only.
- Prefix forwarding and semantic macro recording remain in Slice 8.

## Files of Interest

- `src/input/keymap/contributions.rs`, `src/input/keymap/map.rs`: native admission,
  scope/user precedence, normalized conflicts and fallback.
- `src/input/plugin_keymaps.rs`, `src/input/modal.rs`, `src/input/hints.rs`:
  active native maps, current macro/Org policy and real resolver hint probing.
- `src/session/plugins/keymaps.rs`, `src/session/mode.rs`: generation ownership,
  selector matching, hidden-target inspection and lifecycle recomputation.
- `src/plugins/declarations.rs`, `src/config/resolve.rs`: atomic static admission
  against factory/merged user maps and intersecting selector checks.
- `src/plugins/lifecycle/editor.rs`, `src/plugins/protocol.rs`: typed native
  requests/results, actual generation identity and implemented capability.
- `tests/plugins/keymaps.rs`: real child RPC, both profiles, native keyboard and
  palette rendering, selector, conflict, ownership and finite-admission journeys.
- `docs/reference/plugins.md`, `CHANGELOG.md`: current experimental keymap
  behavior and explicit ownership/recovery boundaries.

## Validation

All eight frozen ordered commands passed locally:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- keymaps --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

Fresh keymap validation passed nine real native/child tests in 8.01 seconds after
compilation. It covers passive first use in both profiles; actual palette hints;
normalization aliases; local-before-global and user-before-default resolution;
competing packages in both installation/RPC orders; conflicted-local fallback;
selector intersection; hidden targets; native recovery/prefix/state/count
refusals; foreign identities; atomic unbind refusal; deactivation and the exact
128-registration ceiling. Static candidate cases also check atomic fallback for
invalid native prefixes, recovery, transitions and overlapping user choices.

SDK validation passed 12 tests. The required command selection passed 52 tests
in 260.77 seconds (265.00 seconds including compilation), and the modal selection
passed 45 tests in 67.21 seconds (69.54 seconds including compilation).
Documentation validation passed 684 commands per combined inventory, 299
configuration entries, 100 reachable pages and 684 local links/anchors.

Combined plugin regression passed all 55 protocol/declaration/lifecycle/buffer/
transaction/invocation/keymap tests in 47.84 seconds. Standalone peer formatting
passed. Exact focused commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- keymaps invocation transactions buffers lifecycle declarations protocol --test-threads=1`
- `rustfmt --edition 2024 --check tests/plugins/peer.rs`

All eight frozen commands passed exact clean-HEAD controller validation on the
final PR-reference commit 92646729268ccbc15ea7bf432c45b60d5870eb4a before handoff.
Canonical PR #7 is merged into dev at that same commit. The integrated full gate belongs to
the controller.

## Risks and Follow-ups

Native map recomputation is bounded and cached, but an individual rebuild is not
preempted halfway through publication; the cooperative service ceiling remains a
scheduling limit rather than a host latency guarantee. Counts/recording, events
and persistent package state retain their recorded later slices. Distribution,
main promotion, releases, mirrors and guide publication remain separate scopes.
