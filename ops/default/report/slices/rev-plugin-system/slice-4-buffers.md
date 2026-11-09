# Slice 4: Independent text buffers

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-04-buffers`
PR: pending

## Purpose

Deliver T4.1-T4.3: captured, revision-qualified multi-buffer reading and native
creation/display through the existing owned package RPC connection.

## Delivered Behavior

- List, open, create, describe, read and show operate on the existing native
  buffer/view owners. Hidden and unsaved ordinary text is addressable without
  changing focus; handles retain session and stable native identity.
- Reads return one source stamp and exact half-open Unicode scalar text. Stale
  revision observations, invalid ranges and oversized UTF-8 results fail
  explicitly. Terminal and generated/transient resources are unsupported.
- Open resolves native path identity outside input, checks for canonical reuse
  before loading and before publication, and preserves existing unsaved text.
  New and absent file visits remain hidden and never imply a filesystem write.
- Create prepares ordinary unsaved text with native unique names, read-only
  attributes and package provenance. Nonempty initial content is modified;
  text and provenance survive process retirement and native navigation. Native
  departure/context publication freezes new registry admissions with busy, so
  background results cannot bypass captured close/quit consent.
- Show validates an explicit live pane or admits the current pane, with current,
  below and right placements. Splits retain selected-pane identity. Native
  transient owners refuse display rather than losing input or confirmation.
- Invocation captures retain the original buffer, revision, pane, scalar point
  and selection after focus changes. Queued requests lose publication authority
  when their generation retires or deadline expires.

## Implementation Notes

The feature commit introducing this report implements T4.1-T4.3. Its canonical
PR and implementation commit are recorded in the PR-reference commit.

The supervisor decodes typed operations and signals the existing native wake.
Session consumes requests under the shared 32-item/2-ms cooperative service
budget, resolves live native identities, prepares results and publishes through
Buffers or Windows. Serialization, pipe I/O, path resolution and regular-file
reading remain outside native input/rendering. Native service uses try-locks and
bounded nonblocking response submission; no process wait or join is added.

One request permit spans both RPC directions, preparation, native queues and
replies: 16 per package and 64 per editor. Duplicate live peer request IDs retire
the generation. A native reply uses its original connection; restart cannot make
old queued work current. Preparation rechecks authority before creating a source,
publishing a visit or displaying a pane. Stop clears retained native requests.

New-file ingress uses native strict UTF-8/NUL policy with nonblocking regular-file
open, source observation checks and a 16-MiB per-file/aggregate retained-ingress
ceiling. Native canonical reuse avoids reloading an existing source. RAII byte
reservations release on success, failure and retirement. Open performs no disk
write. A slow filesystem can delay its supervisor; native input remains separate
and the existing bounded shutdown barrier reports unsettled cleanup truthfully.

Disk tokens encode the native observation without loss; reading/description does
not probe disk or refresh a source revision. Pagination carries the host session
and last native identity, resumes the current registry and retains no unbounded
cursor table. It is not a cross-page snapshot. Source creation and display use
native owners rather than a second text store or rendering surface.

## Contracts Added or Changed

- Runtime initialization now advertises commands, buffers and display. Only the
  six implemented package-to-host methods are admitted; save/close, edits,
  command invocation, keymaps, events, messages and state remain unavailable.
- Read/create text is at most 512 KiB UTF-8; list pages contain at most 256
  descriptors. New source names allow 1..4000 UTF-8 bytes without control
  characters; native unique suffixes remain authoritative.
- Source handles outlive a process generation while the native source exists.
  Closing/replacing a source invalidates its old handle. Foreign sessions cannot
  resolve it; display validates pane lifetime independently of buffer lifetime.
- Native buffer provenance belongs to the created buffer, not the connection.
  Initial package text has no synthetic undo edit and remains protected as
  unsaved content. Explicit mutation and file-lifecycle APIs belong to later slices.

## Files of Interest

- `src/plugins/lifecycle/editor.rs`, `worker.rs`, `lifecycle.rs`: typed native
  exchange, combined permits, ingress accounting and generation retirement.
- `src/session/plugins/buffers.rs`, `src/session/plugins.rs`, `src/session/quit.rs`:
  serialized native operations, opaque identities, source observations and native
  input/departure-owner checks.
- `src/buffer.rs`, `src/buffer/visits.rs`, `src/window.rs`: native preparation,
  persistent provenance, canonical reuse and checked pane display.
- `src/plugins/protocol.rs`: capability and implemented-method admission.
- `tests/plugins/buffers.rs`, `lifecycle.rs`, `peer.rs`: public-wire Rust child
  fixtures with real Session, buffer, view and process ownership.
- `docs/reference/plugins.md`, `CHANGELOG.md`: current API contracts and usable
  experimental behavior.

## Validation

All eight frozen ordered commands passed locally during implementation:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- buffers --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The SDK passed 12 tests, the native command selection passed 52 tests, and the
editing suite passed 114 tests. Documentation validation passed 684 commands per
combined inventory, 299 configuration entries, 100 reachable pages and 684 local
links/anchors. The ordered native command check took 254.91 seconds; editing took
113.46 seconds including compilation, with 54.97 seconds in its test binary.

Fresh focused validation after the final admission guards passed all 31 plugin
tests, including nine buffer tests and the retained protocol, declaration and
lifecycle regressions. It completed in 30.99 seconds after compilation. Coverage
includes both profiles, hidden/unsaved reads, astral/combining/CRLF ranges, byte
limits, canonical aliases, missing/binary/oversized files, stale revisions,
foreign/closed buffer and pane identities, explicit placement, readonly output,
terminal/generated exclusion, frozen captures, retirement and native departure
consent. Edited result text and provenance survive deactivation.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- buffers lifecycle declarations protocol --test-threads=1`
- `rustfmt --edition 2024 --check tests/plugins/peer.rs`

The final PR-reference commit receives one exact clean-HEAD controller validation
of the eight frozen commands before handoff. The controller owns the full
delivery-set gate and merge.

## Risks and Follow-ups

These are trusted local programs with ordinary account permissions. Native input
service limits are scheduling ceilings, not timing guarantees for one bounded
operation. Durable user buffers and JSON/value overhead are additional to
transport and ingress byte accounting; no RSS or throughput benchmark is claimed.

This increment adds no source-edit transaction, forced save/close, dynamic binding,
event subscription, semantic macro or package-state authority. Those retain their
recorded later slices. The full delivery-set gate and integration remain with
the controller.
