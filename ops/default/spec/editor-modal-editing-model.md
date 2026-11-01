# Modal Editing Model

Status: approved
Approved: product owner, 2026-11-01; paired promotion with the specification
Date: 2026-10-28
Reviewed: 2026-11-01
Version: 0.1
Specification: [Modal Editing](editor-modal-editing.md)
Existing models: [Editor](rev-editor-model.md), [Configuration](editor-configuration-model.md)

## Ownership

One admitted startup profile selects input behavior for the session. Native
buffers, revisions, text positions, selections, rectangles, text transfer,
history, search and layouts retain their existing ownership. A modal grammar
does not own a second rope, undo stack, command registry or register store.

Profile is not modal state: choosing Vim is a startup configuration decision;
moving between Normal, Insert, Replace and Visual states is ordinary runtime
editing. There is no runtime operation that changes the selected profile.

## Persistent preference

| Field | Meaning | Persistence contract |
| --- | --- | --- |
| Interaction profile | Emacs-like direct editing or Vim-like modal editing. | One validated startup preference. Omission resolves to Emacs; the admitted value is immutable until the process exits. |

Approved TOML spelling for the implementation delivery:

```toml
[editor]
# Startup interaction profile: "emacs" or "vim". Restart rev after changing it.
interaction_profile = "emacs"
```

This is an approved extension, not a key implemented by the v0.2.0 runtime.
Use the existing root/include precedence, provenance and invalid-candidate
handling. Do not introduce project/language/view profile overrides or a
separate preference file. Missing-config provisioning and the complete sample
gain the key only when the feature is supported. Modal state is not written
back to TOML. No preference reload or runtime setter is introduced.

Extend the existing keys owner with vim_normal, vim_insert, vim_replace,
vim_visual_character, vim_visual_line and vim_visual_block maps. Existing
keys.editor overrides remain scoped to Emacs; Vim factories explicitly retain
compatible native modified-key controls rather than inheriting arbitrary Emacs
overrides. Local component maps apply whenever they own input in either profile.
Inactive profiles cannot shadow the active editing map.

## Ephemeral records

| Record | Fields and contract |
| --- | --- |
| Session profile | Admitted enum value, startup origin and native factory/override resolution. Read-only for the session. |
| Editing state | Normal, Insert, Replace, Visual Character, Visual Line or Visual Block; current editing/focus owner and checked resume observation. Restore only a compatible origin; otherwise retire its transient state into a valid selected context. |
| Pending composition | State, count/operator/character-target kind, finite input, checked arithmetic and originating view/buffer/revision. Cancel or reject when the target is no longer valid. |
| Visual observation | Canonical mark/point or rectangle reference and selection shape. Native owners compute ranges; no private modal selection is authoritative. |
| Command entry | Shared catalog/query/candidate/history owner plus source state, focus and compatible selection observation. Colon is an entry route, not another command identity. |
| Search observation | Existing search session/pattern/direction/revision ownership. Modal bindings select its operations rather than maintain parallel matching state. |
| Jump list | At most 256 ordered locations and a traversal position per view, using native document/location evidence. Session-local and independent of undo/bookmarks. Splits copy the bounded observation, then evolve independently. |
| Jump location | Buffer/file identity, checked logical position or native anchor, source observation and resolution status. Stale locations must never be silently reused as another target. |
| Semantic repeat | Last admitted repeatable operation, target descriptor, checked count and at most 1,024 retained text bytes. An explicit repeat count overrides the recorded count. No raw event, consent or provider/tool replay. |
| One-command Insert continuation | Original compatible insertion state and one bounded Normal-command lifetime. It cannot become a nested command loop or redirect input into a newer prompt. |

These are semantic records, not mandated Rust types or a public extension API.
The interaction implementation should use the existing native owners and
introduce only missing state with a demonstrated lifetime.

## Positions, selection and editing

Canonical byte, character, grapheme, logical-line and display-cell coordinates
remain distinct. Familiar Vim cursor/motion behavior must map to valid native
positions; it does not justify byte-oriented cuts or a second cursor model.
Vim word/WORD and motion-end rules may need new native semantic operations;
reusing a similarly named Emacs motion is insufficient evidence of equivalence.

The owner's Odra parity direction fixes Vim Visual Block endpoints in logical
grapheme columns. Existing Emacs rectangles retain display-column semantics.
Native selection ownership must distinguish those coordinate policies and
preserve checked tab and wide/combining-character boundaries without a private
modal selection store. Linear and block changes use prepared atomic edits and
ordinary history; failure does not publish earlier lines of a multi-line operation.

Vim Normal and Visual line motions use logical source lines while the Emacs
profile retains its existing visual-line behavior. Both project through the
same native wrapping, gutter and pointer coordinate owners.

Colon uses the complete shared interactive catalog initially, with the same
query matching and contextual availability as M-x. No persisted colon-specific
allowlist, denylist or alternate command identity is introduced.

Copy/delete/change/paste and repeat must preserve required selection shape and
linewise intent through shared native ownership. Do not add a second modal
kill ring or an implicit full Vim register namespace.

## Lifetimes and recovery

Startup prepares and validates the complete selected profile before editing.
Invalid settings follow the existing complete-default fallback. After startup
the profile cannot change; only editing state and transient contexts evolve.

Prompts, candidates, tree and confirmations own their input while active.
Suspension/resumption must check the originating view, buffer and revision.
Changing buffer cannot leave a Visual selection or pending operator targeting
the previous document. A profile-specific grammar cannot consume a local
confirmation response or treat completion Tab as jump navigation.

C-g retains native cancellation. Escape handling must distinguish decoded
events from terminal ambiguities and keep the Emacs path unchanged. There are
no idle modal scans or background services. Counts and pending data use the u32
count ceiling, 16-stroke grammar and 96-byte target bound. Jump/repeat storage
obeys its limits above; large counts cannot become count-sized UI loops.

Jump recording/traversal needs its own declared event and invalidation policy.
Reuse checked native locations where possible without making the separate
user-bookmark feature mandatory. Closing, renaming, editing or reopening a
file must not turn an old location into an unrelated current one. Split
inheritance and retention follow the bounded view policy and plan acceptance cases.

## Readiness

The approved specification and [delivery plan](../plan/rev-modal-editing.md)
define finite coverage, keymaps, search, location ownership and bounds.
Implementation must verify cursor/Escape, selection edges and state restoration
against native tests and the inspected reference. There is no persisted
jump/repeat archive, runtime profile setter or executable configuration.
The owner approved the implementation plan and authorized controller startup
and canonical slice integration into dev on 2026-11-01.
