# Slice 58: import native Org feed entries on demand

Status: drafting
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

Bounded native RSS/RDF/Atom observations retain literal stable identities and
prepare one canonical entry/status transaction. Repeated observations deduplicate
against FEEDSTATUS in unsaved canonical source. Hostile XML, ambiguous identities
and executable reader forms reject without changing the target.

## Implementation Notes

Native XML has no entity resolver or evaluator. It admits UTF-8, namespaces,
CDATA and numeric/predefined references under fixed token, node and depth bounds.
Existing capture target resolution and canonical edit/history admission own the
destination. FEEDSTATUS retains literal handled records and existing checksums;
new optional checksum fields are nil. No cross-parser hash equivalence is claimed.
Feed text is rendered as child titles, literal description text and checked links.

See the [feed reference](../../../../../docs/reference/org-feeds.md) for supported
source forms, finite resource limits and status semantics.

## Contracts Added or Changed

- ORG29: bounded RSS/Atom observations and compatible FEEDSTATUS identities.
- ORG18: finite bytes/items and cancellable native work.

## Files of Interest

- `src/org/feed.rs` and `src/org/feed/`: XML, item/status and target observations.
- `src/config/feeds.rs`: declared closed feed data and bounded transport argv.
- `tests/org_source/feeds.rs`: RSS/RDF/Atom and hostile XML cases.
- `tests/org_io/feed_import.rs`: real target admission and repeat-import behavior.

## Validation

Development checks passed:

- `cargo check --lib`
- `cargo test --test org_source --test org_io feed -- --test-threads=4`
- `python3 scripts/check-config-reference.py`

Four focused cases passed, including namespace/base resolution, hostile XML,
canonical BOM/CRLF transactions and FEEDSTATUS deduplication. References agree
on 176 schema/catalog entries, 54 active RGB roles and 261 default bindings.
The unchanged source checker accepts the changed Rust. Four additional native
session cases pass: local import/save/repeat, inbox navigation, real confirmed
shell transport and failed source/tool/destination preservation. Repeat updates
use ordinary save when there is no proposed source edit, preserving history.
All 285 command/configuration tests passed. Transport/session evidence, command
identities and release notes are integrated;
remaining validation expands failure/resource and terminal coverage. Required
exact-head validation remains pending.

## Risks and Follow-ups

Feed identities share the selected drawer's scope. Configure separate headings
or drawers when independent feeds can reuse IDs. Native status history rejects
new imports at its finite ceiling instead of forgetting identities.
