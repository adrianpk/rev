# Native Bookmarks Model

Status: Completed
Completed: 2026-10-31
Validated candidate: `41ba5f89465b2be02adc613cd796271169c245e0`
Gate validation: `7WOA4YIWYISUYIDTDT3ZC5SI5A`
Date: 2026-10-27
Reviewed: 2026-10-31
Specification: [Native Bookmarks](editor-bookmarks.md)
Text/view ownership: [rev Editor Model](rev-editor-model.md)
Plan: [Bookmarks Plan](../plan/rev-bookmarks.md)
Approved: product owner, 2026-10-31; concrete profile approved with explicit Looper startup request
Decision state: BMK01-BMK07 implemented and validated through the completed six-slice delivery and three authorized repairs

## Ownership and identity

One native bookmark collection belongs to a user data file, not a tree root,
view or directory registry. A record has a stable session identity and a unique
case-sensitive name. Updating a name retains that record's identity. Deleting
and recreating the name allocates a different identity. Menu rows, iteration
and source-undo observations must not address records by their display index.

| Field | Meaning and bounds | Durable representation |
| --- | --- | --- |
| `name` | Nonempty UTF-8, at most 256 bytes, no controls; collection-wide unique. | String. |
| `path` | Absolute native path, at most 4096 encoded bytes; canonical existing identity or normalized intended destination. | UTF-8 string when representable. |
| `path_bytes` | Exact Unix path bytes when `path` cannot represent them. | Byte array; exactly one path representation is admitted. |
| `position` | Zero-based source scalar boundary, not byte offset, terminal cell or wrapped row. | Nonnegative checked integer. |
| `before` | Up to configured context scalars immediately before the point. | UTF-8 string. |
| `after` | Up to configured context scalars immediately after the point. | UTF-8 string. |
| `at_start` | The captured prefix reached BOF. | Boolean. |
| `at_end` | The captured suffix reached EOF. | Boolean. |

Only path encoding is platform specific; lossless names must not be inferred
from display labels or lossy conversions. The Linux delivery uses the existing
Unix/native path boundary. Reject NUL and invalid absolute/length encodings.
The scalar location and context describe the live source at capture/checkpoint;
they do not promise that unsaved text is present on disk.

## Live records and resolution

Transient fields: stable record ID, explicit target-mutation generation,
collection generation, optional buffer ID, revision-qualified anchor,
resolution state and the last persisted record.
Resolution states are `resolved`, `unresolved`, `stale` and `unavailable`:

- Loaded records are unresolved until a source is actually available.
- A resolved live anchor refers to the current buffer revision and a valid
  grapheme boundary. Derive its logical line through existing coordinates.
- Removing its source position makes it stale; source undo can restore the
  recorded outcome while the corresponding transaction remains retained.
- An inaccessible or missing file is unavailable. It is not deleted automatically.

Store anchor outcomes against stable bookmark identity and native transaction
identity. Reuse prepared-change mappings and retained structural spans. Compute
fallible outcomes before text publication; observe them atomically with the
source result. Do not introduce another authoritative copy of source text or a
second editing history. Records created, deleted or replaced after an older
transaction must not be overwritten by replay of that transaction's observations.

Pure insertion at the anchor uses left affinity. A replacement whose half-open
removed range contains the point makes it stale; the excluded end boundary maps
normally. Successful explicit set replaces stale evidence. Close/discard/rename
and save-as obey BMK06. A source save updates durable location evidence only
for currently resolved anchors; do not refresh an unresolved record from a
guessed point. Unresolved/stale records retain their last usable evidence.

Reopen checks the original position and context boundaries, then performs the
bounded BMK06 search if needed. An exact match is observed evidence, not a
cryptographic assertion that identical text has unique historical identity.
Multiple candidates, absent evidence or an incomplete search do not resolve.
Source revisions and target file observations must still match on publication.
No directory-wide search, synthetic-file creation or implicit source save occurs.

## Stored document

Version 1 is TOML with a top-level integer `version` and an array of `bookmarks`.
Each element contains the durable fields in the table's order, with `path` or
`path_bytes` as mutually exclusive alternatives. Transient IDs, source revisions,
menu marks, view policy and lookup indexes are not serialized. Unknown versions,
unknown fields, duplicate names, malformed records and overflow are rejected as
a complete candidate before replacing live state. No migration from an earlier
rev bookmark format is required because none has shipped.

```toml
version = 1

[[bookmarks]]
name = "Parser"
path = "/work/demo/parser.rs"
position = 0
before = ""
after = "fn main() {}\n"
at_start = true
at_end = true
```

The path is a neutral illustration; real records retain their actual native
file identity. The example is a whole short source beginning at point zero.
Records are serialized in lexical name order for inspectability, independent
of traversal order. The current configured entry cap and hard 16 MiB admission
bound apply to loading and writing; exceeding a cap never truncates the store.

## Persistence lifecycle

Default location: existing XDG resolver's data root plus `bookmarks.toml`.
An absolute configured path replaces that destination. Missing storage is an
empty first-use collection; malformed or inaccessible storage is a blocked
persistence state with a diagnostic, not an empty replacement candidate.

After lazy load retain the exact observed file identity/content evidence or an
expected absence. Save acquires a nonblocking standard-library file lock on a
sibling lock file, validates that observation, stages the complete bounded
candidate in the same directory, syncs and atomically publishes it through the
existing file IO conventions. Lock contention is an immediate retryable result;
never wait inside input/render. Update the expected observation only after
publication is verified. A failed directory sync or uncertain publication keeps
an explicit unresolved outcome; retry must reconcile it rather than overwrite.

Use the documented [nonblocking standard-library file lock](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock),
without an additional locking crate or a blocking wait in the editor loop.

Concurrent store changes are not automatically merged. A stale writer keeps memory and disk unchanged;
retry succeeds only after safe observation reconciliation. Explicit reload
admits a fresh disk candidate and confirms before discarding dirty in-memory
records; it never replaces a valid collection with a failed parse. Invalid/unreadable
stores remain protected from automatic overwrite. Do not claim that an explicit
save command alone is permission to replace unknown or conflicting contents.
A sibling lock file may remain as the stable lock inode; releasing the OS lock
is distinct from deleting that file. Create user-only directories/files where
newly owned, since context may contain source fragments.

On-change saves checkpoint explicit collection mutations; text edits only mark
changed resolved evidence dirty for the next selected save boundary. On-exit
checkpoints occur after source save/discard decisions. Manual save persists
regardless of automatic policy; manual quit leaves uncheckpointed changes
unsaved and never claims them stored. Failed automatic saves retain the dirty
collection; quit offers retry, explicit discard or cancellation.

## Derived queries and limits

Maintain name lookup, per-buffer source-order navigation and a current logical
line-to-marker projection. Update/invalidate on record mutations, canonical
text transactions, file identity changes and load/save resolution. Rendering
queries only current visible source rows. No serialized index or database is
required. A current file with no resolved bookmarks yields an empty marker map.

The native list holds record IDs and a collection generation, separate deletion
marks and a return location. Next/previous iteration holds the last record ID,
point/revision and collection generation so same-position ties progress without
confusing unrelated cursor movement with iteration. All are bounded by the
configured collection limit and discarded when invalidated.

Marker visibility is view-owned; the collection is shared across panes. Splits
inherit the originating effective visibility. Closing the last view does not
remove records. Use the existing gutter side and text protection, and a fixed
one-cell semantic role; there is no generic provider registry or marker lane API.
