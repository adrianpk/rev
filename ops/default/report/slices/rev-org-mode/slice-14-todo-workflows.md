# Slice 14: deliver Org task workflows and logging

Status: in progress
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-14-todo-workflows`
PR: pending

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

## Risks and Follow-ups

T14.2 boundary/history coverage and the T14.3 owned terminal journey remain in
progress. Required exact-head controller validation and canonical PR references
will be recorded before readiness. Rebase + Fast-forward integration belongs to
the external controller.
