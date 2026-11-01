# Slice 3: Semantic candidates and native fallback

Status: delivered
Delivery set: rev-language-server-support
Plan: [Optional LSP Delivery](../../../plan/rev-language-server-support.md)
Tracker: [Optional LSP Tracker](../../../tracker/rev-language-server-support.md)
Branch: `feat/rev-language-server-support-03-candidates`
PR: [#205](https://forge.adrianpk.com/adrianpk/rev/pulls/205)

## Purpose

Deliver LSP03's primary semantic proposals and CMP02 source selection through
the existing completion session, native fallback and canonical edit owner.

## Delivered Behavior

Eligible configured servers receive completion after the current unsaved
revision's open/change observations, in negotiated UTF-8/UTF-16 positions. Plain
insertion, TextEdit, InsertReplaceEdit and supported CompletionList item defaults
enter the existing popup. Insert/replace proposals explicitly choose the replace
extent and validate both ranges, point and prefix containment. Checked primary
edits preserve CRLF and Unicode and publish through normal canonical history;
preselection and navigation never edit. One undo restores the source.

The common smart-case prefix/subsequence classes use filterText or label.
Server sortText keys rank within each class, preserving wire order on equal or
absent keys. At most 200 candidates survive ranking, and the configured common
candidate limit further bounds presentation. The entire bounded response can
contribute; unsupported early entries do not hide valid later items. Labels and
primary insertions retain the common 512-byte and 64 KiB limits.

Advertised trigger characters bypass the minimum prefix and carry the protocol
trigger context. Incomplete results are requested again after changed input.
A subsequent complete empty list clears the previous incomplete state. This
shares the ordinary debounce, invocation, filter and popup rather than locally
assuming an incomplete result is exhaustive.

Valid empty, no admitted items, unsupported capability, request timeout and
unavailable connection remain distinct source outcomes. Each selects admitted
native syntax or Org fallback. Disabled/excluded buffers retain native behavior;
restoring eligibility does not enable globally disabled LSP. Semantic completion
also works for a configured language without a native grammar. A native
structural context cannot veto an explicit semantic invocation; automatic
comment/string policy remains in force where native syntax supplies that context.

Explicit snippet-format items, commands, indentation adjustment and additional
edit items are omitted from this primary profile. The client advertises false
snippet support and only the implemented insert/replace and list-default fields.

## Implementation Notes

Connection workers own JSON decoding, admission, pipe IO and request deadlines.
Completion and synchronization share one ordered observation queue and its
64-message/16 MiB permit ledger. Queued requests coalesce by canonical buffer
before reserving capacity, retaining the newest context. Pending requests have
monotonic IDs, the configured maximum of 16 per connection, and a deadline from
`lsp.request_timeout_ms` (2000 ms by default). Idle workers block on their inbox;
only active completion work retains the existing native completion polling slot.

Each request carries the connection-owned activation token, canonical stamp and
immutable rope. The shared completion worker retains one running invocation and
one newest pending context. Cancellation wakes a semantic wait immediately;
worker cleanup and request expiry send `$/cancelRequest`. Failure releases queued
and pending waiters. Buffer edits, invocation/view changes, exclusion, disable,
connection replacement or retired activation cannot revive an old popup, even
when undo restores identical bytes. Arrival and acceptance recheck the exact
synchronized document identity alongside native view/revision checks.

Semantic results reuse the ordinary candidate/edit representation with an
optional checked primary range. Native results keep their existing replacement
and parser ownership. No text, history, server installer or UI owner is added.
The observable source outcome is separate from transport readiness.

## Contracts Added or Changed

Completion now supports the optional semantic source in addition to native
syntax and Org. The existing TOML server configuration and LSP controls become
usable with primary completion; no preference, dependency or command is added.
The completion reference/how-to and Unreleased note describe this usable subset.

## Files of Interest

- `src/lsp/completion.rs`, `src/lsp/completion/admit.rs`: observation, source outcome and primary admission.
- `src/lsp/requests.rs`, `src/lsp/inbox.rs`, `src/lsp/connection.rs`: bounded ordered requests, cancellation and deadlines.
- `src/completion.rs`, `src/completion/job.rs`, `src/session/completion.rs`: shared completion and canonical acceptance.
- `tests/lsp/candidates.rs`, `tests/lsp/candidates.sh`: owned stdio fixture and semantic/native regressions.

## Validation

Successful development checks:

- `cargo test --locked --test lsp candidates::`: primary forms, ordering, fallback,
  unbundled language, stale replies, encoded replacement, range admission,
  incomplete refresh, optional source, candidate bound and semantic context passed.
- `cargo test --locked --test lsp candidates::current_list_refresh -- --nocapture`:
  complete-empty reset and both trigger contexts passed.
- `cargo clippy --locked --test lsp --test completion -- -D warnings`: passed.
- `cargo run --locked --quiet --example documentation-check`: 330-command
  inventories, 211 configuration entries and 63 reachable pages passed.

Changed production owners pass the existing safe-Rust/source-quality bounds:
parameters at most five, nesting at most three and cyclomatic complexity at most
ten. Final exact-HEAD evidence uses the configured ordered commands through the
controller after the canonical PR-reference follow-up:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test commands --test configuration --test syntax`
- `git diff --check`

The fixtures observe real client frames, unsaved synchronization before requests,
no implicit file writes, checked primary undo/redo, deterministic fallback and
late replies after cancel/edit/undo/view/exclusion/disable. Owned children are
reaped on session exit. The local peer establishes deterministic protocol
behavior; installed-server interoperability remains with its approved slice.

T3.1 is commit `1f97405488ae3213e8226fbb2337f9cc00e236e0`.
T3.2 is commit `6ebbfbf69dbbfac6775394108fb70aaacd488b72`, which introduces this report with the completed source/policy boundaries and
additional protocol regressions.

## Risks and Follow-ups

Selected-item resolve and atomic same-document additional edits remain in Slice 4.
Shared lifetime/restart and real Rust interoperability remain in Slice 5; joint
acceptance and complete optional-LSP guidance remain in Slice 6. Existing
predecessor acceptance evidence is preserved.

## Integration

Canonical PR [#205](https://forge.adrianpk.com/adrianpk/rev/pulls/205) merged
through Rebase + Fast-forward into dev at
`df2cef8db7e2065bb6c5ddf888abdf86b2d1e63a`. Exact configured receipt
`X2QWAAQE6DLNMYYRXWEUTR3HSK` passed all three ordered commands and
402 tests at that clean head. The controller recorded the merge in Acta.
