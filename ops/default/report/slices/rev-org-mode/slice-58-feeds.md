# Slice 58: import native Org feed entries on demand

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-58-feeds`
PR: `#159`

## Purpose

Deliver explicit native feed imports under the approved
[plan](../../../plan/rev-org-mode.md#slice-58-import-native-org-feed-entries-on-demand)
and [tracker](../../../tracker/rev-org-mode.md#slice-58-import-native-org-feed-entries-on-demand).

## Delivered Behavior

Explicit updates import bounded local RSS/RDF/Atom XML or fetch a declared HTTP
source after transport confirmation. Each feed prepares and saves one canonical
entry/status transaction. Repeats deduplicate against FEEDSTATUS, including
unsaved source; inbox navigation works independently of fetch. Invalid feeds,
read-only/stale destinations and failed tools retain previous source/history.
Visiting and idle sessions never fetch.

## Implementation Notes

Native XML has no entity resolver or evaluator. It admits UTF-8, namespaces,
CDATA and numeric/predefined references under fixed byte, token, node and depth
bounds. Existing capture target resolution and canonical edit/history admission
own the destination. FEEDSTATUS retains literal handled records and existing
checksums; new optional checksum fields are nil. No cross-parser hash equivalence
is claimed. Feed bodies remain fixed-width literal text, with checked links.

Commands update all declared feeds serially, select one feed, open an inbox or
inspect per-feed publication outcomes. Native file reads retain canonical
precedence; configured transport reuses owned process groups, private temporary
paths, finite time/output ceilings, cancellation and cleanup. Successful imports
save through the existing file owner. Empty repeat edits retry ordinary save
without introducing a history entry; uncertain completion remains distinct.

See the [feed reference](../../../../../docs/reference/org-feeds.md) for syntax,
configuration origins, supported formats, finite limits and status semantics.
The reference behavior follows the [Org feed manual](https://orgmode.org/manual/RSS-Feeds.html),
[RSS specification](https://www.rssboard.org/rss-specification) and
[Atom specification](https://www.rfc-editor.org/rfc/rfc4287).

Task commits: T58.1 `408e378b14c2c1bb14db98c9d2ae9252149d5db5`,
T58.2 `0e8c7d1d48bda010dcb02cf5fa03aafdf7517258`,
T58.3 `aba31dbcc6f12ff537709143757c7c6095a04e30`.

Canonical PR: [#159](https://forge.adrianpk.com/adrianpk/rev/pulls/159), targeting `dev`.

## Contracts Added or Changed

- ORG29: bounded RSS/Atom observations and compatible FEEDSTATUS identities.
- ORG18: finite bytes/items and cancellable native work.

## Files of Interest

- `src/org/feed.rs` and `src/org/feed/`: XML, literal status, checked target and fetch.
- `src/config/feeds.rs`: closed declarations and typed transport argv.
- `src/session/feeds.rs`: explicit commands, confirmation and retained outcomes.
- `tests/org_source/feeds.rs` and `tests/org_io/feed*.rs`: native/parser/session cases.
- `scripts/configuration_ui/org_feeds.py`: owned real HTTP and terminal journey.

## Validation

Required ordered commands passed on the clean implementation candidate:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_source --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

All 399 native tests passed: 129 configuration, 189 I/O and 81 source tests;
zero failures, ignored or filtered tests. Required implementation receipt
`3MNSBQZMTWAUPBRKPOJDCBSAIB` identifies candidate
`4a52e1ba10814650a94e8662e7c73c187c5c8db6`.

Focused commands passed:

- `cargo test --test commands --test org_editing -- --test-threads=4`
- `cargo test --lib org::io::process -- --test-threads=1`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-feeds`
- `git diff --check`

All 387 focused tests passed: 158 commands, 226 editing and three shared process
cases. The library filter intentionally excludes 46 unrelated tests. Focused
implementation receipt `VNOOPHFHOGIYO265DPGNN4GZYM` identifies candidate
`aba31dbcc6f12ff537709143757c7c6095a04e30`. Final readiness requires both ordered
lists again for the clean PR-reference head; controller receipts and Acta retain
that candidate and the final validation IDs.

Sixteen focused feed tests passed before the ordered checks. Cases cover RSS/RDF/Atom namespaces and bases,
hostile entities, exact byte/item limits, literal FEEDSTATUS compatibility,
BOM/CRLF source, repeat imports, batch outcomes, stale disk contents, read-only
admission, invalid UTF-8, failed exits, output limits, cancellation, timeout,
session drop and owned leader/descendant cleanup. HTTP configuration now admits
URLs independently of local destination checks. Unterminated CRLF inbox creation
lands on the inserted heading and remains one unsaved undo unit.

Both owned RGB/NO_COLOR sessions passed, zero skips. Real goserve/curl requests
prove no fetch on visit, decline or inbox navigation; explicit consent fetches,
imports exact BOM/CRLF bytes and supports one-step undo/redo. Repeats skip the
handled ID; actual HTTP 404 preserves source and reports NotPublished. The driver
waits for ordinary source coloring after redo before issuing another command;
all request-count, file-byte and history assertions remain unchanged. Each owned
HTTP server and editor is stopped/reaped in finally cleanup.

The unchanged source checker accepts the changed Rust. References agree on 176
schema/catalog entries, 54 active RGB roles and 261 default bindings.

## Risks and Follow-ups

Feed identities share the selected drawer's scope. Configure separate headings
or drawers when independent feeds can reuse IDs. Native status history rejects
new imports at its finite ceiling instead of forgetting identities.
