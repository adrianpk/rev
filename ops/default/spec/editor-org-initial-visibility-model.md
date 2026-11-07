# Org Visibility Policy

Status: Completed
Date: 2026-11-06
Updated: 2026-11-07
Specification: [Org Initial Visibility and Drawers](editor-org-initial-visibility.md)

## Configuration contract

| Field | Type | Default | Allowed values |
| --- | --- | --- | --- |
| `org.startup_folded` | string enum | `showeverything` | `overview`, `content`, `showall`, `showeverything` |

The compatibility default preserves existing ordinary-file presentation. Admit
the enum through native defaults, partial configuration, merging, provenance,
validation and the full sample. Invalid configuration rejects the candidate;
omission inherits without a document rewrite. No new state store is introduced.

## Document and view state

| Value | Owner and invariant |
| --- | --- |
| Configured policy | Existing immutable admitted startup snapshot |
| Document policy | Last structurally recognized visibility token in source order outside literal blocks and drawers; absent means inherit configuration |
| Effective initial policy | Current admitted document policy, otherwise configured policy |
| Initialization state | Existing view lifetime; pending/current native observation and one-time admission, including explicit user-choice suppression |
| Current folds | Existing view-owned checked ranges and revision/source identity |
| Runtime choice | Selected view only; no global fold preference or persisted override |

Keyword/token comparison is case-insensitive while source spelling is preserved.
Unrecognized words do not erase earlier recognized tokens. Resolve metadata
through the bounded worker's structural result, not a second foreground parser.
Partial/malformed observations do not authorize speculative fold ranges.

## Lifecycle

- A genuinely new Org view admits initial visibility once when its current
  structural result is ready. A runtime choice made earlier suppresses that
  pending initialization.
- A split clones current folds and initialization state. Peer views retain
  their own later choices.
- Redraw, resize, buffer switches and observation refresh do not reinitialize.
  Edits and external reload retire stale ranges through existing native rules;
  they do not resurrect old offsets or automatically restore startup folds.
- Explicit startup reset resolves current metadata and replaces the selected
  view's visibility through the normal checked operation.
- Existing deliberate AI REQUEST folding takes precedence over automatic
  initialization. Explicit later controls remain available.
- Closing a view discards its transient state. No session-catalog migration,
  persisted visibility record or new filesystem directory is added.

## Whole-view operations

Hide/show changes every admitted drawer's body bit while retaining heading and
ordinary body state. Toggle selects hide when any admitted drawer is open,
otherwise show. Existing native delimiter presentation and empty-target feedback
remain authoritative. Install new ranges atomically using current source stamps,
fold limits and point/reveal admission. Read-only source is still eligible for
these UI operations; source text, dirty state, mark and history remain unchanged.

## Command metadata

The specification defines four new unique canonical names and their exact
descriptions. Extend existing command/Org catalogs and the current description
inventory; retain all previous definitions and historical baseline evidence.
No aliases, new keybinding context or factory shortcut is introduced.

## Completed delivery evidence

Canonical PR #310 is integrated into dev. The unchanged controller gate passed
at `dc31493589c9a35eff79edebe1765a76a3fa4651`. The [Slice 1 closure](../report/slices/rev-org-initial-visibility/slice-1-policy.md#delivery-set-closure)
records OIV01-OIV05, model, exact command and gate log evidence.
