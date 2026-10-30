# Slice 58: import native Org feed entries on demand

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-58-feeds`
PR: pending

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

Development checks passed:

- `cargo test --test commands --test configuration -- --test-threads=4`
- `cargo test --test org_io --test configuration --test org_source feed -- --test-threads=4`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-feeds`
- `git diff --check`

All 285 command/configuration tests passed before the final boundary additions;
16 focused feed tests then passed. Cases cover RSS/RDF/Atom namespaces and bases,
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
schema/catalog entries, 54 active RGB roles and 261 default bindings. Required
and focused exact-head controller validation remains pending.

## Risks and Follow-ups

Feed identities share the selected drawer's scope. Configure separate headings
or drawers when independent feeds can reuse IDs. Native status history rejects
new imports at its finite ceiling instead of forgetting identities.
