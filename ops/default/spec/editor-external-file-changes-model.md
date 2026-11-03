# External File Changes Model

Status: Completed
Completed: 2026-11-03
Date: 2026-11-02
Approved: product owner, 2026-11-02; paired external-reload delivery approval
Specification: [External File Changes](editor-external-file-changes.md)
Existing ownership: [Editor Configuration Model](editor-configuration-model.md)
Readiness: delivered configuration and native transient state; frozen integrated gate passed

## Persistent configuration

Use the existing sole user TOML root, includes, validation and complete native
defaults. No per-project discovery, session archive or reload-state file is
introduced. The engineering defaults below make the approved delivery's bounds
explicit and match the implemented settings and packaged defaults.

| Field | Type and admitted values | Default | Meaning |
| --- | --- | --- | --- |
| `files.external_changes` | String, `reload` or `ask` | `reload` | Behavior for an external content change in a clean buffer. |
| `files.external_change_debounce_ms` | Integer, 0..=2000 | 150 | Coalesce bursts before a fresh bounded observation. |
| `files.external_change_max_bytes` | Integer, 1..=134217728 | 16777216 | Maximum document bytes admitted for one reload read. |
| `files.external_change_max_buffers` | Integer, 1..=4096 | 256 | Maximum concurrently observed visiting buffers. |

Example of the user-facing choice:

```toml
[files]
external_changes = "reload"
```

These fields are startup settings. `revert-buffer` is an explicit operation,
not a new configuration-loading mechanism. The two policies share exactly the
same dirty-buffer and stale-observation guards.

## Transient records

| Record | Fields in contract order | Invariants |
| --- | --- | --- |
| Subscription | Buffer ID, canonical visiting path, observation generation, parent-directory watch identity. | One logical subscription per visiting buffer; shared views reuse it. Registrations may share the parent watch. |
| Disk evidence | Existing file observation, existence/type, serialized image observation and last completed save identity. | Use canonical file ownership. Metadata is evidence, not proof of unchanged content or completed publication. |
| Pending change | Subscription generation, observed disk evidence, kind, notification state. | Per-buffer, coalesced by admitted version. Dismissing is not save/reload consent. |
| Read candidate | Buffer ID/stamp/state, visiting path, policy observation, disk-before/after evidence, complete validated text/BOM. | Private until both current buffer and disk checks pass; one bounded candidate per buffer. |
| Discard decision | Buffer ID/stamp, visiting association, observed destination and explicit answer. | Only an affirmative fresh decision can discard unsaved text; stale answers cannot authorize a later version. |
| Reload publication | Candidate identity, native edit transaction, new clean disk image and observation. | At most once; one revision/history event for all views and derived owners. |

No buffer text is serialized to a second persistent store by this capability.
Existing recovery and save ownership remain unchanged.

## State and transition rules

States are watching, checking, pending, conflict, unavailable and retired.
Pending means an eligible disk change awaits explicit action under `ask`.
Conflict means local unsaved text prevents automatic publication under either
policy. Unavailable retains the live document with a missing/invalid/limited
diagnostic. A fresh eligible event or explicit reload can leave unavailable.

Events carry subscription generations. Retiring or rebinding a subscription
invalidates its queued work. A clean capture that becomes dirty while reading
transitions to conflict instead of publishing. A newly observed identical image
updates disk evidence without adding a text revision or history record.

Successful reload records the new disk image as clean through the existing
native history owner. Undo changes text back without rewriting disk and leaves
it dirty. Redo follows ordinary history semantics; no private reload history is
maintained. If existing history admission cannot retain a valid reload change,
report a pending/limited outcome instead of silently clearing history.

## Detection, concurrency and bounds

Use an event-driven file-observation owner with at most one active read worker.
Queue at most 256 distinct changed buffer identities; repeated events coalesce.
Overflow causes a metadata recheck of admitted subscriptions in batches of 16.
Do not enumerate unrelated directories or repeatedly read unchanged contents.

Capture at most the configured maximum bytes, with a 5-second admission
deadline. A late or blocked OS read cannot publish after that deadline; it
also cannot justify spawning an unbounded series of replacement workers.
Allow at most two immediate stale/unstable read retries before leaving a pending
diagnostic for the next event or explicit command. The limits include deleted,
rebound and cancelled work; retry does not create another unbounded queue.

Watch parent directories so rename-over saves remain observable. Resolve and
pin the canonical visiting association; do not retarget a buffer just because
an original symlink or neighboring file name now points elsewhere. Source
ingress, permissions and regular-file checks remain with the native file owner.

The approved plan selects the already locked rustix filesystem/event APIs,
without adding a watcher crate or first-party unsafe code. A watcher event is a
hint to revalidate, not direct authority to mutate text.

## Compatibility and diagnostics

Absent fields use complete defaults, including automatic reload. Invalid
settings reject the complete candidate under existing configuration behavior.
No durable runtime data migration is needed. New settings must be represented
in typed projection/validation, sample TOML and configuration reference.

Diagnostics distinguish notification-only, local-edit conflict, source race,
missing destination, invalid ingress, size/read limit and observation failure.
They identify a buffer and useful next action without dumping its contents.
No automatic confirmation, force-save or agent write authority is inferred.

## Delivery evidence

EFC01-EFC08 completed through canonical PRs #229-#236 in the
[Pi/reload delivery](../plan/rev-ai-pi.md). The unchanged controller full gate
passed at `ecd02abcbd1f08530d8fad1395672dc6d00ee085`, receipt
`HBFDUZX54JK3RJUBGSQYIJANGO`, on 2026-11-03. The
[Slice 8 closure](../report/slices/rev-ai-pi/slice-8-acceptance.md#delivery-set-closure)
records exact commands, 578 selected top-level tests, both-profile native/PTY
acceptance and separate real-provider evidence. Existing requirement, scenario,
configuration and resource contracts remain intact.
