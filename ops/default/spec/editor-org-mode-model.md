# Org Mode Model

Status: accepted
Approved: product owner, 2026-10-27; specification and model accepted, with the explicitly listed open decisions retained.
Date: 2026-10-27
Specification: [Org Mode](editor-org-mode.md)
Existing text/view model: [rev Editor Model](rev-editor-model.md)
Configuration owner: [Editor Configuration Model](editor-configuration-model.md)

## Ownership and storage

Org documents remain ordinary canonical UTF-8 buffer/file text. Commands edit
that source through existing editor transactions. Recognition, indexes, folds,
agenda rows and calculated previews are derived observations, not another
editable document database. Normal save preserves unaffected source bytes.

User preferences use the existing TOML store. Allowed file-local Org directives
are document data and affect only their documented semantics. Runtime view and
query choices do not rewrite TOML implicitly. No new disk record/cache format
is selected by this model; add such a contract only if it becomes necessary.

## Source records in document order

| Concept | Semantic fields | Source/meaning |
| --- | --- | --- |
| Document | Identity, canonical revision, visiting file identity and source text. | Existing buffer/file lifecycle. |
| Heading | Level, optional task keyword, optional priority/comment marker, title, local tags, owned section and descendants. | One outline hierarchy from document text. |
| Drawer | Kind/name, opening/body/closing source, containing context. | Ordinary drawers and property drawers remain distinct. |
| Property | Name, stored value, additive form and local owner. | Effective inheritance is derived; special derived properties are not arbitrary stored fields. |
| List item | Bullet/counter, indentation, optional checkbox, body and nested content. | Structural source; continuation and nested ownership must be preserved. |
| Task state | Workflow identity, keyword, unfinished/finished classification and optional selected logging facts. | Keywords derive from admitted defaults/source directives. |
| Planning fact | SCHEDULED/DEADLINE/CLOSED kind and owned timestamp source. | Only in its valid heading context. |
| Timestamp | Active/inactive delimiters, date/day spelling, optional time/range, repeater, warning delay and exact text. | Invalid values remain preserved and diagnosable; query time/zone is separate. |
| Link | Form, target, optional description/search suffix and owned source. | Recognition has no external effect. |
| Table | Affiliated names/captions, rows, separator rows, cells and formula source. | Stored values and alignment/display are separate. |
| Formula | Original text, admitted target field/column and expression/format. | Unsupported syntax is preserved; evaluation is explicit. |

A source locator must identify the document, observed revision and exact target.
An existing stable source ID may help resolve a later observation; a heading
title or cached row number alone is not sufficient authority to mutate text.
Indexing does not insert IDs or normalize source merely to simplify navigation.

## Derived state and lifetime

| Record | Required evidence | Invalidation / outcome |
| --- | --- | --- |
| Effective metadata | Local owner, ancestor/file values and admitted policy. | Recompute after relevant source or setting changes; retain local/inherited distinction. |
| Outline/drawer visibility | View, document revision and requested visibility. | Independent across shared views; released on view close. |
| Agenda source observation | Document/file identity, current buffer revision or checked disk version, completeness/failure. | Stale state cannot authorize source edits; missing source is visible. |
| Agenda result | Query/source set, captured time/zone, filters/sort, checked source locators and completeness. | Derived rows refresh/re-resolve; actions target canonical source. |
| Capture session | Template choice, bounded substitutions/input, editable draft, destination evidence and lifecycle. | Abort leaves target unchanged; finalize revalidates and publishes once. |
| Table calculation | Source revision, admitted references/dependencies, numeric policy, proposed results and diagnostics. | Publish only after validation; failure preserves prior results. |
| Stored link | Checked source/target information and optional descriptive text. | Resolution must not rely on an obsolete display row alone. |

## Capture and move states

Capture has explicit choosing/collecting/editing, validating, finalized, aborted
and failed-with-draft outcomes. The exact user-visible transitions and recovery
choices must be finalized with the spec. A failed destination must not cause a
valid draft to disappear or claim a committed insertion.

Cross-document refile/archive identifies source and destination and records
which edits and durable saves actually succeeded. The future implementation
must provide an atomic or explicitly recoverable outcome; it must not collapse
partial publication into a single false success state.

## Preferences to define

Typed preferences will cover workflows/logging, tags/properties and inheritance,
calendar/time policy, agenda sources/views/limits, capture templates/destinations,
archive/refile defaults, table numeric/display policy and context keybindings.
Fields are partial under existing include/root inheritance rules. No exact new
TOML key or serialization schema is frozen here. Supported choices, defaults,
scopes, units and bounds must be mirrored in the exhaustive sample when delivered.

## Explicitly unresolved model choices

- Factory workflow/default templates and target policies.
- Admitted safe document-local directives and their precedence over defaults.
- Numeric representation, rounding, arithmetic precedence and error values.
- Agenda/source-set configuration shape and index persistence, if any.
- Task/clock/timer records only if those additional capabilities are selected.
- Refile/archive recovery protocol and any durable operation journal.

These are model decisions for review. They do not authorize an independent Org
store, a new runtime component or implementation work.
