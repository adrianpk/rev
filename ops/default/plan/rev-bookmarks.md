# Native Bookmarks Delivery

Status: Implementing
Date: 2026-10-31
Delivery set: rev-bookmarks
Slice strategy: behavior-first
Scope direction: product owner, 2026-10-31; named persistent bookmarks using Emacs as the behavior reference, navigation and native gutter markers
Approved: product owner, 2026-10-31; complete scope, six-slice map, autonomous implementation, canonical PR merges into dev, final gate and documentation closure; explicit startup request
Planning baseline: `3083e6467756755b56ac5e753dab9625c1d336da`
Spec: [Native Bookmarks](../spec/editor-bookmarks.md)
Model: [Bookmarks Model](../spec/editor-bookmarks-model.md)
Spec: [Editor Gutter](../spec/editor-gutter.md)
Model: [Gutter Model](../spec/editor-gutter-model.md)
Concern: [Pending Work Index](../README.md)
Tracker: [Bookmarks Tracker](../tracker/rev-bookmarks.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-bookmarks.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 6; canonical PR #190 open, exact-HEAD validation and integration pending

## Outcome and authority

Deliver named file-position bookmarks with ordinary Emacs commands/bindings,
checked edit/reopen behavior, persistent user data, native listing and file-local
stepping, and terminal gutter markers. One ordered set and one Looper lifecycle
cover all six slices. The owner approved this concrete profile and explicitly
requested startup on 2026-10-31. This authorizes preparation commits/push to dev,
autonomous implementation and the controller-owned slice merges into dev.

After startup, the configured Looper owns task progress, slice review,
exact-HEAD checks, Rebase + Fast-forward integration into dev, the final gate
and documentation closure. It does not wait for individual human slice merges.
Prepare the initial tracker once; do not maintain a competing progress ledger.
Main promotion, releases, mirrors, installation and website publication remain
separate actions. The pending tree-highlight and M-x-hint tickets are not part
of this delivery.

## Strategy and prerequisites

Six behavior-first increments give each cross-owner behavior a working native
path. Slice 1 is usable within a session; later slices supply retained locations,
storage, list navigation and gutter projection. Each report describes its actual
boundary rather than calling partial work the whole bookmark capability.

Delivered foundations: canonical buffer/file identity and checked visit/save,
prepared edits and source history, retained structural mappings, native command
catalog, vertical prompts, complete startup TOML, split views, logical gutter,
soft wrap and Org visibility. No LSP, modal profile, provider registry, new crate
or external database is needed. Existing rectangle chords under C-x r remain.

All artifacts are English. Use ordinary Conventional Commits without task IDs
in commit subjects. New settings and available commands update the sample,
reference and catalog in the slice that introduces them. Update Unreleased
through the established changelog workflow when a usable outcome ships. Keep
the core User Guide independent of Org and use short examples.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Named bookmark commands | `feat/rev-bookmarks-01-commands` | `feat(bookmarks): set and visit named file locations` | `ops/default/report/slices/rev-bookmarks/slice-1-commands.md` |
| Slice 2 | Checked location lifecycle | `feat/rev-bookmarks-02-locations` | `feat(bookmarks): retain checked locations through edits` | `ops/default/report/slices/rev-bookmarks/slice-2-locations.md` |
| Slice 3 | Persistent collection | `feat/rev-bookmarks-03-persistence` | `feat(bookmarks): persist the user bookmark collection` | `ops/default/report/slices/rev-bookmarks/slice-3-persistence.md` |
| Slice 4 | List and ordered navigation | `feat/rev-bookmarks-04-navigation` | `feat(bookmarks): list and navigate saved locations` | `ops/default/report/slices/rev-bookmarks/slice-4-navigation.md` |
| Slice 5 | Gutter markers | `feat/rev-bookmarks-05-gutter` | `feat(gutter): show native bookmark markers` | `ops/default/report/slices/rev-bookmarks/slice-5-gutter.md` |
| Slice 6 | Joint acceptance and guidance | `feat/rev-bookmarks-06-acceptance` | `test(bookmarks): validate persistent navigation and markers` | `ops/default/report/slices/rev-bookmarks/slice-6-acceptance.md` |

## Slice 1: Named bookmark commands

Prerequisites: approved preparation committed to dev and the delivered foundations.
Acceptance: BMK01, the set/jump/delete part of BMK04 and native BMK05 naming.
Owners: native buffer/file identity, command registry, session argument prompts,
effective keymaps and vertical completion.

- T1.1: Introduce the bounded native collection and stable record IDs; implement
  set, no-overwrite, jump and delete through real session commands and checked
  file visits. Initially reject stale revision targets until Slice 2 supplies
  mapping/resolution; do not silently clamp them.
- T1.2: Add the applicable C-x r bindings, name completion and cancellation,
  complete current catalog/reference coverage and native tests/bookmarks.rs.

Check same-name replacement, duplicate refusal, more than one name per file,
Unicode names/paths, intended new files, unnamed/synthetic rejection, canonical
aliases, read-only sources and exact text/dirty/undo preservation. Do not install
bindings to later unimplemented commands merely to complete a static table.

Validation: `cargo fmt --all -- --check`,
`cargo test --locked --test bookmarks --test commands --test files --test configuration`,
then `git diff --check`.

## Slice 2: Checked location lifecycle

Prerequisite: Slice 1 integrated. Acceptance: BMK02 and BMK06.
Owners: src/buffer/edit.rs and prepared mappings/history, src/window/points.rs,
file visit/rename/save-as and native folded/wrapped point reveal.

- T2.1: Integrate atomically published anchor outcomes, insertion affinity,
  deletion/stale transitions, retained moves and source undo/redo with stable
  record identity. Cover records created/deleted/replaced after old transactions.
- T2.2: Implement context capture, bounded cancellable reopening, canonical live
  buffer preference, rename/save-as/close/discard transitions and failure-preserving
  jumps. Do not read or modify other source files to resolve an unavailable target.

Check insert-at-point, deletion boundaries, multi-replacement and structural
moves, undo/redo and eviction, split panes, unsaved live text, small external
changes, repeated context ambiguity, BOF/EOF/empty files, missing targets and
Org fold/wrap reveal. Native correctness tests carry most variants.

Validation: `cargo fmt --all -- --check`,
`cargo test --locked --test bookmarks --test core --test editing --test rectangles --test files --test windows --test wrapping --test org_editing`,
then `git diff --check`.

## Slice 3: Persistent collection

Prerequisite: Slice 2 integrated. Acceptance: BMK05 and storage aspects of BMK07.
Owners: existing XDG resolver, startup preferences, file observation/staged
publisher, native quit/save continuations and bookmark IO boundary.

- T3.1: Implement strict version-1 TOML read/write, lossless path encoding,
  lazy admission and checked atomic publication under a standard-library
  interprocess lock; retain malformed/conflicting stores and unsaved memory.
- T3.2: Implement on-exit/on-change/manual policy, explicit save/reload and quit
  retry/discard/cancel. Add the storage/limit settings and complete defaults,
  merge/validation/reference coverage without rewriting existing user config.

Use real isolated directories and two real processes for the writer conflict.
Cover restart, intended missing files, unsaved source/discard, permissions,
unknown versions/fields, duplicate names, resource caps, lock contention,
non-UTF-8 path round trips and staged/publication failure outcomes. Fault
injection is limited to the existing concrete IO boundary.

Validation: `cargo fmt --all -- --check`,
`cargo test --locked --test bookmarks --test files --test recovery --test configuration --test commands`,
then `git diff --check`.

## Slice 4: List and ordered navigation

Prerequisite: Slice 3 integrated. Acceptance: the remaining BMK04 behavior.
Owners: native result buffers/list navigation, current command routing and
context-specific keymaps. Reuse facilities, not an extension framework.

- T4.1: Implement the read-only bookmark list with stable row IDs, status,
  Emacs-style row navigation and staged deletion/unmark/execute/return; revalidate
  generation and return location. Add its local keymap and complete samples.
- T4.2: Implement next/previous within the current canonical file, including
  wrap, same-position names, invalidation and clear empty/unresolved outcomes.

Check large lists without visiting every file, context overrides, cancellation,
stale rows, deletion/recreation of a name, two names at one point, origin-buffer
close, split focus and collection mutation while a list remains open.

Validation: `cargo fmt --all -- --check`,
`cargo test --locked --test bookmarks --test commands --test configuration --test windows --test display --test panels`,
then `git diff --check`.

## Slice 5: Gutter markers

Prerequisite: Slice 4 integrated. Acceptance: BMK03 and marker parts of BMK07.
Owners: native gutter geometry and view policy, frame projection, hit testing,
wrap layout, semantic palette and configuration schema.

- T5.1: Integrate the optional fixed marker cell and revision-current logical
  row index independently of number visibility. Preserve both sides, current
  number behavior, text protection, fold/wrap mapping and pointer geometry.
- T5.2: Add marker visibility/glyph/palette settings, per-view native toggle,
  split inheritance and catalog/default/reference coverage. Coalesce multiple
  names on one line without discarding their independent records.

Check absolute/relative/off numbering, both sides, blank/continuation/folded
rows, Unicode glyph rejection, narrow/zero panes, pointer selection, search
highlighting, resize, view close/reopen and zero filesystem IO during paint.

Validation: `cargo fmt --all -- --check`,
`cargo test --locked --test bookmarks --test gutter --test wrapping --test pointer --test search --test panels --test configuration`,
then `git diff --check`.

## Slice 6: Joint acceptance and guidance

Prerequisite: Slice 5 integrated. Acceptance: BMK01-BMK07 jointly.

- T6.1: Add one native acceptance example and scripts/check-bookmarks.sh using
  the established Bash/Rust PTY approach. Exercise set/edit/undo/save/restart/
  jump/list/delete and gutter behavior together. Measure bounded relocation,
  1024-record rendering, large-source reversal/resize and idle work.
- T6.2: Complete concise how-to/reference guidance, link bookmarks from the
  appropriate existing core learning page, include focused text feedback
  examples, and reconcile the changelog, sample/catalog and operational index.
  Update directly related gutter clauses with this delivered extension while
  preserving their earlier gate evidence and numbering identity.

The shell wrapper owns all temporary homes, XDG roots, sources and child
processes; do not touch installed binaries or user stores. Bound each PTY child
and restore terminal attributes. No Python runtime or new validation framework.
New docs paths are delivery outputs, not currently implemented guidance.

Validation: `cargo fmt --all -- --check`,
`cargo test --locked --test bookmarks --test gutter --test wrapping --test commands --test configuration`,
`bash -n scripts/check-bookmarks.sh`, `bash scripts/check-bookmarks.sh`,
`cargo run --locked --quiet --example documentation-check`, then `git diff --check`.

## Final gate and execution cost

Run the metadata Full gate once on the clean integrated candidate after all
six slice PRs. Shared edit/history publication, file IO, quit flow, view geometry
and input admission justify the all-target native suite here. Do not run that
suite after every task or append historical Org/export or wrap benchmark scripts.
The final bookmark journey covers the combined affected behavior itself.

Expected warm-run cost is a few minutes, dominated by compilation and the release
PTY fixture. Record actual time and measurements; this is an estimate, not a
successful execution claim. Keep focused failures cheap to reproduce, then use
controller-owned exact-candidate repair/retry mechanisms. Do not alter a frozen
gate, reset run budgets, replace another run or create parallel controllers.

A gate repair, if needed, requires the controller's explicitly pinned bounded
repair authority and its canonical PR path. Do not patch primary dev manually
to bypass a blocked validation. Startup must inspect the installed Looper help
again and select the new set explicitly; historical registered configurations
retain their identities and are not part of this delivery.

## Closure

Before startup, commit the approved spec/model, plan/tracker, index and direct reference updates to dev.
Create the new local Looper configuration outside the repository, with its
explicit plan/tracker paths, ordered per-slice checks and exact gate.
After startup Looper alone records active task/slice state and verified merges.

After the gate passes, the controller may close only this plan, tracker,
spec/model, pending-work index, the six configured reports and directly related
gutter specification/model metadata. Record gate SHA/receipt separately from
historical gates. No code, tests, scripts, dependency or product-doc edits belong
to the post-gate documentation closure. A pass does not authorize main promotion.
