# External File Changes

Status: Completed
Completed: 2026-11-03
Date: 2026-11-02
Format: OpenSpec capability specification
Source: product-owner confirmation of automatic reload, configurable notification and protection of unsaved edits
Behavior confirmed: product owner, 2026-11-02
Approved: product owner, 2026-11-02; proceed through the linked Pi and reload delivery
Model: [External File Changes Model](editor-external-file-changes-model.md)
Baseline: [rev Editor](rev-editor.md)
Configuration: [Editor Configuration](editor-configuration.md)
Related integration: [Pi Integration](editor-ai-pi-integration.md)
Plan: [Pi and Reload Delivery](../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../tracker/rev-ai-pi.md)
Delivery boundary: eight-slice Pi and reload delivery through dev under the approved plan
Readiness: implemented and validated through the completed Pi/reload delivery

## Purpose

Keep visiting buffers consistent with externally changed files according to a
user-selected policy. This applies to another editor, a command-line tool, Git
or an agent. It is ordinary editor behavior and works without AI or Pi.

The owner confirmed automatic reload as the default, an alternative that asks
the user, and preservation of unsaved edits in both modes. Detection and reload
must preserve rev's normal responsiveness, checked source identities and undo.

## Existing-state alignment

The current file owner reuses one live buffer per canonical path. Reopening
that path selects the existing buffer instead of re-reading its bytes.
`src/buffer/save/decisions.rs` detects an external destination change during
save and requests fresh overwrite consent. Neither behavior supplies a general
disk-to-buffer reload service. This proposal adds that service while preserving
the existing save protection.

## Requirements

### Requirement: EFC01 Configurable external-change policy

The existing user TOML SHALL accept `files.external_changes = "reload"` or
`"ask"`. Omission SHALL mean `"reload"`. The complete sample, typed defaults
and reference SHALL describe both choices. Invalid values SHALL follow normal
whole-candidate configuration rejection. No new configuration file is created.

`reload` SHALL automatically refresh an eligible clean visiting buffer.
`ask` SHALL retain its text and display a pending-change notification until
the user explicitly requests reload. Neither policy grants permission to
discard unsaved text. Configuration loading remains startup-based; this
capability does not introduce general configuration hot reload.

#### Scenario: Choose notification instead of automatic reload

- **GIVEN** `files.external_changes = "ask"` and an unchanged visiting buffer
- **WHEN** another process replaces the associated file
- **THEN** rev reports the pending external change without replacing the text
- **AND** the user can continue editing or explicitly invoke `revert-buffer`.

### Requirement: EFC02 Observe visiting files without scanning projects

rev SHALL observe canonical files associated with live ordinary buffers,
including files in hidden buffers. Shared views SHALL not duplicate watches or
reloads. Transient result, capture and source-edit draft buffers SHALL not be
treated as independently visiting files.

Detection SHALL handle replacement by rename, direct writes, deletion and
recreation. Parent-directory observations SHALL survive replacement of a
file's inode. Events SHALL be coalesced and source bytes verified before
publication. An event alone is not proof that a writer completed a valid file.
Kernel notification overflow SHALL schedule a bounded recheck of registered
files, not a recursive project scan. Watch exhaustion or unavailable detection
SHALL be reported; explicit reload remains usable.

#### Scenario: Observe an atomic external save

- **GIVEN** two panes share one clean visiting buffer
- **WHEN** a tool writes a temporary file and renames it over the visited path
- **THEN** rev detects the replacement and admits one reload for both panes
- **AND** observation continues for later replacements at that path.

### Requirement: EFC03 Validate a complete fresh reload candidate

File metadata observation and reading SHALL run outside the input/render loop.
Capture the buffer identity/revision, visiting path, disk observation and
effective policy. Apply the existing strict UTF-8, NUL/BOM and separator ingress
rules to complete bytes. Recheck disk and buffer observations before native
publication; stale or unstable candidates SHALL be discarded and re-observed
within finite bounds. Never publish a partially read file as a complete image.

A metadata-only change with the same serialized document image SHALL refresh
the checked disk evidence without adding a text edit or moving the cursor.
Observation of an editor-owned completed save SHALL not create a reload loop.

#### Scenario: Typing wins over a pending automatic reload

- **GIVEN** a reload read started for a clean buffer
- **WHEN** the user edits that buffer before the candidate can publish
- **THEN** automatic publication is rejected as stale
- **AND** both the user's text and the external change remain available.

### Requirement: EFC04 Preserve unsaved text and save protections

Both policies SHALL retain dirty buffer text when the file changes externally.
Display a pending conflict and keep normal editing available. A dismissed
notification SHALL not acknowledge the external version as a completed save
or bypass the existing overwrite guard. Subsequent disk changes SHALL remain
observable even after an earlier notice was dismissed.

An explicit `revert-buffer` on a dirty buffer SHALL request confirmation to
discard its unsaved contents. That consent SHALL be bound to the actual buffer
revision and observed destination. Another edit or disk change invalidates
stale consent. Canceling preserves all current text.

#### Scenario: Keep local and external edits distinct

- **GIVEN** a modified buffer under the automatic policy
- **WHEN** another process changes its file
- **THEN** rev reports a conflict and keeps the local edits
- **AND** neither automatic reload nor later save silently replaces either version.

### Requirement: EFC05 Explicit native interaction in both profiles

`revert-buffer` SHALL be a native command available through M-x and modal `:`.
In a clean buffer its explicit invocation is sufficient consent to read the
fresh disk version, including under `ask`. In a dirty buffer it uses EFC04.
It performs no source-file write and does not save other buffers.

Notifications SHALL identify the affected buffer and the reload action. They
SHALL not steal input focus, repeatedly open a modal prompt, or cancel typing.
Repeated observations of one unchanged pending version SHALL coalesce. Switching
buffers SHALL retain each buffer's pending state. Runtime policy toggles and
new default shortcuts are not needed for this initial contract.

#### Scenario: Reload after a notification

- **GIVEN** a clean buffer with a pending notice under `ask`
- **WHEN** the user invokes `revert-buffer` through either command surface
- **THEN** rev reloads a newly checked version, not stale notification bytes
- **AND** the pending state clears only after successful publication.

### Requirement: EFC06 Preserve native editing and derived state

An admitted content reload SHALL be one native document transaction and one
undoable change. Mark the new image as the observed clean disk state only
after successful publication. Undo restores the prior text and ordinary dirty
state relative to the current disk image; automatic reload SHALL not erase it.
Do not clear preexisting history merely because the change came from disk.

All views SHALL retain their positions where valid and use the canonical
coordinate/edit mapping or nearest valid positions when text disappears.
Preserve pane focus, buffer identity, read-only policy and file association.
Readonly visiting views may refresh their disk image without becoming writable.

Syntax, native completion, bookmarks, selections, wrapping, Org projections and
LSP document versions SHALL observe the same canonical revision transition.
Pending edits, searches or AI proposals with stale source evidence SHALL not
publish against the replaced image. No feature gets a private replacement rope.

#### Scenario: Undo an automatic reload

- **GIVEN** a clean buffer whose external update was automatically published
- **WHEN** the user invokes undo
- **THEN** the prior text returns as an ordinary modified buffer
- **AND** an unchanged external file does not immediately overwrite that undo.

### Requirement: EFC07 Preserve source on missing or invalid destinations

Deletion, permission failure, a nonregular destination, an escaping or changed
association, oversized content or invalid encoding SHALL retain the live text,
history and saved-state evidence. Report the actionable condition. Never turn
a missing file into an empty buffer, recreate it in the background, or follow
a rename to another path without an explicit visiting operation.

#### Scenario: File disappears while open

- **GIVEN** a visiting buffer under either policy
- **WHEN** its file is deleted
- **THEN** rev retains the text and reports the missing destination
- **AND** recreation at the same admitted path is observed and revalidated.

### Requirement: EFC08 Bound resource use and observation lifetime

Use finite event queues, registrations, read sizes and work batches from the
paired model. A single oversized or continuously changing file SHALL not
block other editing or allocate without limit. Closing a buffer, changing its
association or exiting rev SHALL retire only its owned observations and work.
Idle observation SHALL wait on events rather than repeatedly read document
contents. Recovery after an overflow SHALL yield between finite batches.

#### Scenario: Close a buffer during an external read

- **GIVEN** an asynchronous reload candidate is being read
- **WHEN** its buffer is closed or rebound to another path
- **THEN** the stale candidate cannot install text into another buffer
- **AND** the old observation is retired without affecting other files.

## Acceptance and preparation

Cover both policies, native command surfaces, atomic rename and direct writes,
same-byte replacements, rapid changes, open/hidden/shared/read-only buffers,
dirty buffers, cancellation, save races, undo/redo, missing/recreated files,
invalid input, watcher overflow/limits and closed-buffer results. Include one
real kernel-notification fixture and one native/PTY editing journey. Controlled
events cover rare races; they do not prove operating-system delivery.

Measure idle wakeups/CPU, representative multi-buffer latency, memory and large
file limits. Keep tests at affected file/buffer/configuration boundaries plus
representative syntax/LSP/Org invalidation; do not append all historical gates.
Document short ordinary examples and reflect the canonical Markdown guidance
in the local Spore User Guide projection.

The linked approved delivery places this capability before Pi in the same
set. It is not a prerequisite for Pi's read-only discovery
and native-result path. Direct agent filesystem editing would require an
additional application/authority decision in the AI contract.

## Delivery evidence

EFC01-EFC08 completed through canonical PRs #229-#236 in the
[Pi/reload delivery](../plan/rev-ai-pi.md). The unchanged controller full gate
passed at `ecd02abcbd1f08530d8fad1395672dc6d00ee085`, receipt
`HBFDUZX54JK3RJUBGSQYIJANGO`, on 2026-11-03. The
[Slice 8 closure](../report/slices/rev-ai-pi/slice-8-acceptance.md#delivery-set-closure)
records exact commands, 578 selected top-level tests, both-profile native/PTY
acceptance and separate real-provider evidence. Existing requirement, scenario,
configuration and resource contracts remain intact.
