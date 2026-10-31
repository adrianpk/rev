# Slice 3: Persistent collection

Status: implementing
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-03-persistence`
PR: pending

## Purpose

Deliver BMK05 and the storage/limit aspects of BMK07 through the existing
configuration, file observation, staged publication and session owners.

## Delivered Behavior

First bookmark use lazily admits one strict version-1 TOML collection. Reads
allocate at most 16 MiB of store bytes. Names and context validate as a whole;
native paths round trip losslessly, including non-UTF-8 Unix paths. Loaded
records receive fresh session identities and remain unresolved until visited.
Malformed, unsupported and over-limit stores remain unchanged.

The store compares exact loaded identity and bytes under a nonblocking standard
library file lock. It stages, syncs and atomically replaces in the destination
directory through the existing concrete Output boundary. Publication failures
retain memory and old disk; failures after rename retain an unresolved outcome
that can only reconcile matching actual candidate bytes. Newly created data
directories and files use user-only permissions.

## Implementation Notes

The canonical collection owns its dirty evidence and configured admission and
context limits. Text transactions mark evidence dirty without performing IO.
Executable startup supplies the existing XDG data destination; embedded sessions
perform user data IO only when their host configures an explicit destination.
Typed startup settings merge by leaf and reject invalid paths, modes or limits.

## Validation

Development check: `cargo test --locked --lib bookmark::store -- --nocapture`
passed five store tests, including two real writer processes, a separate held
interprocess lock, strict admission, native path round trips and staged versus
post-publication faults. Final exact-HEAD controller receipts are pending.
