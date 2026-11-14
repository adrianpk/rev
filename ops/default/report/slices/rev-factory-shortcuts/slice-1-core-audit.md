# Slice 1: Core and Profile Audit

Status: reviewing
Delivery set: rev-factory-shortcuts
Plan: [Factory Shortcut Alignment Delivery](../../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../../tracker/rev-factory-shortcuts.md)
Branch: `docs/rev-factory-shortcuts-01-core-audit`
PR: `#47` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/47))

## Purpose

Establish the evidence-backed core/profile portion of the approved shortcut
audit before any factory mapping changes or mode-family decisions.

## Delivered Behavior

The [semantic matrix](../../rev-factory-shortcuts/semantic-matrix.md) accounts
for all 707 fixed public commands and six prompt actions across 28 contexts.
It reviews 333 core/profile rows and explicitly leaves 374 Org, Markdown and
Git rows unresolved for Slice 2. It proposes only P01-P04, rejects nine
approximate reference substitutions and retains deliberate native conventions.
The editor's shipped behavior and configuration bytes are unchanged.

## Implementation Notes

T1.1 re-extracts 2,349 raw factory entries, including inherited copies, at audit
candidate `2f5cfa485fb0169c0f4dcdea475a33eb4a80eabd`. The catalog stores source
tree `442765af9faac4351c208b2bfc5ca02c1b896331`, compiler identity, lockfile digest,
operation variants, canonical aliases and the six prompt-only actions.
The extraction compiles current native source and uses prebuilt dependencies
only as read-only inputs; build outputs stay in the owning checkout.

T1.2 records operation effects, profiles, routes, reference families,
dispositions and family-specific source/validation boundaries. In particular,
`universal-argument` and `quit` retain canonical alias access, and macro abort
is reachable through a contextual `C-]` interception despite lacking a raw map
entry. Reference macro-menu quit/refresh/help have different public names.

T1.3 uses isolated native sessions and GNU Emacs 31.1 / Org 9.8.7 under `-Q`:

- Eight first-text cases agree in nonmodal and all six modal states (56
  sessions). Source, stamp, history, mark, selection shape and state remain
  unchanged with explicit count four.
- Nested pane widths change from 8/20/70 to 32/33/33, retaining source and
  selected pane. A path prompt rejects balancing.
- Native join uses the following line; reference default join uses the
  preceding line. Native undo and redo restore/reapply the native edit.
- Pair motion reaches offset 3 on `(ab)`; reference forward-sexp reaches 4.
  Plain paragraph separation agrees, while an explicit reference-local `%%`
  paragraph rule stops at 4 and native blank-line motion reaches 19.
- Both profile selections admit the isolated P01-P04 proposals in their stated
  owners. Editor overrides stay separate from modal maps. Search and state-
  specific C-o controls remain intact; protected C-g/modal M-x overrides fail.
  C-? equals Backspace and C-i equals TAB after native normalization.
- Project switching without host-configured storage refuses safely and retains
  edited source after cancellation. Registered-project restoration is a later
  runtime acceptance case, not a result of this probe.

## Contracts Added or Changed

Adds an audit catalog, reproducible evidence and core dispositions under
FSA01-FSA03/FSA07-FSA08 and the core portion of D03. No accepted mapping
manifest, factory default, runtime source, configuration schema, hook policy,
historical acceptance rule or product decision changes. D01/D02 and mode-owned
D03 coverage remain pending.

## Files of Interest

- [Catalog](../../rev-factory-shortcuts/catalog.json): complete raw inventory
  and native operation identities, preserving inherited coverage.
- [Semantic matrix](../../rev-factory-shortcuts/semantic-matrix.md): reviewed
  core families, all per-command dispositions and explicit Slice 2 handoff.
- [Audit runner](../../rev-factory-shortcuts/audit.py): source-compiled native
  extraction and isolated stock-reference comparison.
- [Native probe results](../../rev-factory-shortcuts/probe-results.json) and
  [core reference](../../rev-factory-shortcuts/reference-core.json): asserted
  effects and separate reference observations.

## Validation

- `python3 ops/default/report/rev-factory-shortcuts/audit.py --dependency-dir "$REV_AUDIT_DEPS"`: PASS. The variable identifies compatible prebuilt Cargo dependencies; the runner owns its library output and temporary sessions. Evidence comparison writes no tracked output.
- Configured frozen audit coverage/document consistency command: PASS (`Audit coverage and document consistency passed`). Corrected command SHA-256: `7b5e753f6c395dcc4329adca0137602e45bb40cbab593af3a9436970bc160ff7`. The injected correction uses hexadecimal regex escapes so the exact command fits the PR's single-backtick grammar; assertions and ordering remain unchanged. Its exact text belongs to the controller receipt and PR validation list; duplicating its identifier-scan expression here would match the report itself.
- `git diff --check`: PASS; rerun for the final candidate.

These are focused audit checks. No full Cargo suite, delivery-set gate,
registered-project restore journey or physical terminal acceptance ran.
The final clean slice HEAD still requires the controller's exact-command
validation receipt before handoff. No owned fixture or probe process remains.

## Risks and Follow-ups

The proposed mappings remain unaccepted. Slice 2 must finish all 374 mode-owned
rows and active language/profile overlays, settle D01-D03 and obtain the mapping
decision before its PR submission. Later runtime slices must verify actual
binding dispatch, source/projection targets, overrides, hints, undo/count and
consent boundaries, and physical terminal delivery. This report is review
evidence, not delivery-set acceptance.
