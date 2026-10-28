# Slice 15: record Org effort estimates and priorities

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-15-effort-priorities`
PR: pending

## Purpose

Deliver checked native priority and EFFORT edits with source-preserving history.

## Delivered Behavior

C-c comma selects a priority; S-Up/S-Down cycle the configured range and the
unmarked state. Inspection distinguishes explicit cookies from implicit defaults.
The approved single-character profile admits uppercase A-Z, factory A/C/B.
Data-local PRIORITIES declarations select a buffer range without changing settings.

C-c C-x e and M-x set local EFFORT after validating bare minutes, H:MM[:SS] or
compound min/h/d/w/m/y durations. Entered spelling stays intact. Inspection shows
its elapsed minutes. EFFORT_ALL data supplies bounded completion/cycling choices;
factory has no implicit list. Empty entry removes the local base estimate.

## Implementation Notes

The existing observation worker records at most two bounded heading words with
exact source offsets. Prompts retain source/view qualifiers. Narrow priority
plans and the existing property editor use ordinary prepare/publish and history;
no separate editable document or permanent worker is introduced.

Work-duration scales are typed positive finite minute values and remain distinct
from calendar scheduling. Parsing admits at most 128 UTF-8 bytes and 32 compound
components; results must be finite and nonnegative. Up to 64 EFFORT_ALL choices
are validated as data. No dependency, external program or code evaluation is used.

Priority leaf spelling validates per source; range ordering validates after
include layering. The first include test exposed why partial leaf overrides
cannot be compared with factory bounds before their included bounds are merged.
The existing whole-candidate owner now validates the resolved priority range
before settings projection.

## Contracts Added or Changed

- ORG06: native priority cookies retain their meaning independently of task and
  checkbox states, with configured/default/local range policy.
- ORG21: checked native EFFORT duration properties preserve source spelling.

## Files of Interest

- `src/org/duration.rs`, `src/org/priority.rs`: native bounded data admission.
- `src/org/visibility/tags/priority.rs`: observed priority source plans.
- `src/session/outline/estimates/`: qualified prompts and property publication.
- `src/config/`: typed priority/unit layering, validation and projection.
- `tests/org_editing/estimates.rs`, `tests/configuration/org_estimates.rs`:
  actual edits, history, declarations and configuration provenance.

## Validation

Development checks passed:

- `cargo check --quiet`
- `cargo test --test org_editing --test configuration estimate -- --nocapture`:
  five native editing tests and two configuration policy tests passed.
- `python3 scripts/check-config-reference.py`: 111 schema/catalog entries,
  143 default bindings and 13385 exact template bytes agree.
- The initial focused source-quality check retained a start_effort complexity
  finding; property lookup is separated into its existing ownership boundary
  during T15.2 before final readiness checks.

The initial quality check required separating Org argument dispatch and effort
prompt construction from their callers. The same behavior assertions remain.
A clean, owned Emacs batch observation verified priority cycling, duration forms,
unit scales and the absence of factory EFFORT_ALL choices; no source is copied.
Reference behavior is documented in [priorities](https://orgmode.org/manual/Priorities.html)
and [effort estimates](https://orgmode.org/org.html#Effort-Estimates).

T15.2 checks passed:

- `cargo test --test org_editing --test configuration estimate -- --nocapture`:
  eight native editing/context/matching tests and two configuration tests passed.
- Focused source quality passed after separating effort-property lookup from
  prompt construction.

Derived PRIORITY and ITEM now consume the same observed cookie and policy as
editing. A missing cookie exposes its implicit default while remaining absent
in source. Typed custom task states, EFFORT ownership and property inheritance
remain intact across task transitions and undo. Agenda keymap e/comma admit these
same operational source commands; calendar maps and prompt cancellation retain
separate admission. Revision-qualified agenda rows belong to their recorded slice.

T15.3 focused checks passed:

- `cargo test --test org_editing --test configuration estimate -- --nocapture`:
  14 native estimate tests and two configuration tests passed at initial boundary
  coverage; replacement/removal and shared-view checks were then added.
- `cargo test --test org_editing estimate_edges -- --nocapture`: all eight
  boundary tests passed, including exact 128-byte/32-component limits, finite
  overflow, malformed numeric/priority forms, cancellation, stale drafts,
  read-only source, Unicode EOF, original property spelling and peer anchors.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`:
  all four owned RGB/NO_COLOR editing/read-only sessions passed, zero skips.
  Every prior task assertion remains. New priority/effort steps verify native
  chords/M-x, invalid input, real saves, atomic undo/redo and exact BOM/CRLF/EOF
  restoration; original snapshot and child/descriptor cleanup checks remain.
- Focused changed-source quality passed with the original parameter, nesting,
  complexity and safe-Rust limits.

An independent rejection fixture first reproduced fractional colon components
being admitted. The parser now requires integer colon components, as the owned
reference observation confirmed; decimal work units remain valid. Whitespace
between a number and its unit is accepted without normalizing the stored value.
No assertion or acceptance threshold was weakened.

## Risks and Follow-ups

Required exact-head validation and PR references precede readiness.
Active-clock estimate routing remains with the recorded clock slice; no
clock-dependent command is exposed before its owner exists.
