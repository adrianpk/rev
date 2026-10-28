# Slice 14: deliver Org task workflows and logging

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-14-todo-workflows`
PR: [#115](https://forge.adrianpk.com/adrianpk/rev/pulls/115)

## Purpose

Deliver native task workflows, selected logging and coherent metadata history.

## Delivered Behavior

C-c C-t and M-x use the same task handler. File TODO, SEQ_TODO and TYP_TODO
sequences override typed startup defaults. Native state selection, forward and
backward cycling, clearing and inspection recognize only admitted keywords;
unknown leading words remain title text. Fast selection annotations and state
entry/departure logging remain source data.

Selected CLOSED timestamps and state logs/one-line notes are staged before one
checked publication. C-g cancels the entire transition. Task dependencies reject
completion before mutation. Priorities, tags, properties, planning and unrelated
source survive; parent task cookies use the same recognized custom workflow.

## Implementation Notes

The existing temporary observation worker retains bounded workflow declarations
and direct metadata offsets. Editing and tag/property matching resolve the same
workflow. Lists consume this workflow for TODO cookie statistics as well.
Prompts retain their source revision and originating view, and shared points use
the existing prepare/publish transaction and ordinary history owner.

Logging captures one local operation timestamp before the note prompt. The
session owns a clock provider, with deterministic injection for native tests.
The production provider reads at most 1 MiB of regular TZif data and accepts an
explicitly covered instant or fixed UTC. Unsupported, leap-aware or uncovered
zones reject before publication. No shell, external date process, unsafe code,
new dependency, permanent worker or input/render source parser is introduced.
This is the timestamp capture needed by task logging; civil-time editing and
recurrence remain assigned to their recorded later slices.

Task commits:

- T14.1: [a1350e3dc561dee6d28c01b2ba5d7982de042676](https://forge.adrianpk.com/adrianpk/rev/commit/a1350e3dc561dee6d28c01b2ba5d7982de042676).
- T14.2: [0eb2ad95aaaacab8c321d28d4c695ee8f5c1d115](https://forge.adrianpk.com/adrianpk/rev/commit/0eb2ad95aaaacab8c321d28d4c695ee8f5c1d115).
- T14.3: [8161231d1127cc26cb6848ae2206d13fa993f8ab](https://forge.adrianpk.com/adrianpk/rev/commit/8161231d1127cc26cb6848ae2206d13fa993f8ab).

## Contracts Added or Changed

- ORG06: ordered unfinished/finished workflows, selected logging and task metadata
  publish atomically; priorities remain distinct from checkboxes.
- ORG17: default keys, M-x and configured aliases share actual task handlers.

## Files of Interest

- `src/org/task/`: bounded declaration and annotation admission.
- `src/org/visibility/tags/tasks/`: dependency, logging and source edit plans.
- `src/org/list/progress.rs`: shared custom-workflow cookie counting.
- `src/session/outline/tasks.rs`: source-qualified state/note drafts and publication.
- `src/session/outline/tasks/clock/`: bounded current local timestamp support.
- `scripts/configuration_ui/org_tasks.py`, `examples/org-task-session.rs`: owned
  native terminal and canonical read-only admission fixtures.
- `tests/org_editing/tasks.rs`, `tests/configuration/org_tasks.rs`: native behavior
  and typed configuration layering, provenance and bounds.

## Validation

Development checks passed:

- `cargo check --quiet`
- `cargo test --test org_editing --test configuration task -- --nocapture`:
  seven task/checkbox tests and two configuration policy tests passed.
- `python3 scripts/check-config-reference.py`: 102 schema/catalog entries,
  137 default bindings and 12928 exact template bytes agree.
- Focused changed-source quality check: safe Rust, parameters <= 5,
  nesting <= 3 and cyclomatic complexity <= 10.

The logging fixture initially selected its drawer line; it now targets the actual
heading. Initial source checks required separate logging policy and TZif block
helpers; the same assertions pass after that decomposition.

T14.2 boundary checks passed:

- `cargo test --lib session::outline::tasks::clock -- --nocapture`: two focused
  calendar/TZif tests passed using fixed instants and bounded synthetic zones.
- `cargo test --test org_editing task -- --nocapture`: 12 tests passed, including
  prior checkbox statistics, reopening, exact workflow caps, malformed/deeper
  rejection, shared UTF-8 EOF anchors and read-only inspection/admission.
- Focused source quality passed for the added boundary owners and tests.

Reopening removes an otherwise empty CLOSED planning line so an adjacent
property drawer retains its structural ownership. Mixed planning retains its
other fields. EOF prefix detection uses scalar boundaries, including multibyte
final titles. Only the immediate parent's applicable task cookies change;
checkbox-owned COOKIE_DATA retains its own statistics policy.

T14.3 terminal checks passed:

- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`:
  four owned sessions, editing/read-only in RGB and NO_COLOR, zero skipped.
  Native C-c C-t, M-x and an alias produce the same complete metadata. Blocking,
  note cancellation, real saves and single-step undo/redo preserve exact
  BOM/CRLF/EOF source. Every child is reaped and PTY descriptors close.

The read-only fixture sets the existing canonical buffer admission flag before
Session construction, since the normal executable has no read-only startup
option. It uses the extracted production session loop, including native input
batching/deadlines, rather than an alternate event driver. All fixture HOME/XDG
paths and cwd are owned; snapshots admit no incidental file changes. The example
initially attempted a private settings accessor; it now retains the same startup
snapshot for its explicit host-owned view. Separating fixture construction from
terminal ownership also keeps its source complexity within the existing limit.

The first required batch passed formatting/Clippy, then existing invalid-source
configuration cases exposed TODO validation projecting unrelated, unvalidated
palette/gutter values. TODO validation now projects only its own fields onto
factory task defaults, preserving the validate-before-projection contract.
Existing rejection fixtures provide the regression signal; no checks are weakened.
Final receipts are regenerated after this correction.

Canonical PR [#115](https://forge.adrianpk.com/adrianpk/rev/pulls/115) targets
`dev` with the approved literal title. Its immutable report link uses full
introduction commit `a1350e3dc561dee6d28c01b2ba5d7982de042676`.

After this PR-reference follow-up is committed and pushed, required readiness
checks will be recorded through the controller against the exact clean HEAD:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test configuration --test commands`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused same-HEAD evidence will additionally cover the task clock unit tests,
public source observations, configuration reference and the unchanged outline
terminal journey, since the shared declaration/cookie observations and native
session loop are affected. Final receipt identities belong in Acta and the
controller checkpoint, without changing a validated report HEAD.

## Risks and Follow-ups

Required exact-head controller validation and canonical PR references
will be recorded before readiness. Rebase + Fast-forward integration belongs to
the external controller.
