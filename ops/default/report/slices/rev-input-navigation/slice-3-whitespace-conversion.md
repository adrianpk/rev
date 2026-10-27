# Slice 3: Add explicit tabify and untabify

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-03-whitespace-conversion`
PR: [#83](https://forge.adrianpk.com/adrianpk/rev/pulls/83)
Integrated: `bc94816edd96ef245310b892df24fe57edc8589a`

## Purpose

Convert explicitly selected whitespace under the effective buffer tab policy
without changing other text, settings or visual text columns.

## Delivered Behavior

- `M-x tabify` and `M-x untabify` use the existing admitted region. Missing
  regions report the ordinary region error; an empty region is a no-op.
- Untabify expands selected tabs at their true display columns. Tabify replaces
  selected ASCII whitespace runs only with a shorter tab/space representation.
  One-cell runs cannot shorten and stay unchanged.
- Conversion publishes disjoint replacements as one canonical transaction.
  One undo restores every run and redo restores conversion, with ordinary
  shared-view point/mark mapping and clean/modified state.
- Nonempty read-only regions reject conversion. Missing, empty and already
  unchanged regions preserve source, revision and history; no-op feedback is
  `No whitespace changed`. C-g retains normal M-x cancellation.

## Implementation Notes

The buffer owner scans canonical graphemes from each selected line's beginning
so Unicode and unselected prefixes contribute their real columns. Only wholly
selected ASCII space/tab glyphs join a run. Line separators and all other
characters stay outside replacement spans. The configured 1..=32 tab width bounds
expansion per selected tab; tabify output never exceeds its input run.
A following zero-width cluster can join the last expanded space. One additional
space anchors that cluster's existing visible cell when required, preserving
its and subsequent text columns without changing Unicode bytes. This uses one
following grapheme of lookahead and at most one anchor space per run.
Temporary output is proportional to selected whitespace, produced spaces and
that following grapheme, with checked allocation and position arithmetic; no document-wide copy or
retained per-line cache is introduced.

The session reuses active-region admission and rejects nonempty read-only
conversions before preparing output. It captures all shared points, computes the
initiating region endpoint, prepares every replacement against one revision and
publishes once through the ordinary view/history owner. No-op planning returns
before publication.

The historical configuration gate no longer requires the captured conversion
ticket to remain future/open. Its guard retains that ticket's exact capture
identity and requires the canonical Slice 3 branch, report, PR and three unique
ancestor task commits. Readiness admits a reviewing outcome; full acceptance
requires solved/fixed metadata plus the delivered report's integrated ancestor.
Positive and negative fixture repositories exercise these provenance checks;
conversion semantics are established separately by native and PTY assertions.

## Contracts Added or Changed

WSP01 exposes both native names on the admitted marked region with preserved
visual columns. WSP02 uses one checked transaction and ordinary safety/history.

## Files of Interest

- `src/buffer/whitespace.rs`: checked region/run projection and conversion.
- `src/session/whitespace.rs`: admission and atomic publication.
- `src/command.rs`: both compiled command identities.
- `tests/commands/whitespace.rs` and `whitespace/`: actual M-x conversion,
  display-column policy, atomic history, admission and saved-byte fidelity.
- `scripts/configuration_ui/whitespace.py`: isolated controlling PTY journeys.
- `scripts/configuration_ui/delivery.py`, `evidence.py` and
  `scripts/test_acceptance.py`: retained, reconciled conversion provenance.
- `docs/reference/commands.md`, `bindings.md` and `defaults.md`: intended region
  selection, effective tab policy and both native command names.

## Validation

- PASS: `cargo test --test commands whitespace::` (eight native groups).
  The policy cases include 13 explicit byte expectations and 36 width/prefix/run/
  direction cases with preserved non-whitespace columns and idempotence.
- History cases exercise both directions and both region orientations, disjoint
  runs in one edit, undo from another shared view, redo and clean/modified state.
  Admission cases compare complete snapshots for missing/empty/read-only/no-op
  regions; another buffer and canceled command retain their ordinary state.
- Saved-byte cases retain BOM, mixed LF/CRLF, final-newline presence/absence,
  Unicode and the combining-mark anchor across actual save, undo and redo.
- PASS: `python3 scripts/check-pty.py --suite whitespace` (two owned PTYs).
  Both native M-x commands, explicit selection, missing-region feedback, exact
  saved bytes, one undo/redo, no-op and cancellation pass with tab widths 4/8.
- PASS: `python3 -m unittest discover -s scripts -p test_acceptance.py -k conversion_provenance`.
  A real fixture Git history proves ancestry; missing/duplicate/wrong capture,
  task, PR, branch, report and integration metadata are rejected. Reviewing is
  admitted only for readiness, while ordinary acceptance still requires delivery.
- PASS: `python3 scripts/check-config-reference.py` (111 defaults retained).
- PASS: focused source analysis of both owners and policy/history tests retains
  safe Rust, five-parameter, nesting-three and cyclomatic-ten limits.
- A regression reproduced a lost presentation cell after a tab followed by a
  combining mark. The selected replacement now anchors that existing cell,
  retaining the WSP01 column assertion and original Unicode source bytes.
- The width grid uses the existing 1..=32 admission range; no policy limit changes.
- Initial canonical Clippy validation rejected a redundant reference in the new
  publisher. Passing the existing borrowed span directly removes that warning;
  no lint or acceptance limit changes.

PASS: canonical controller evidence `ZH4LE3KVA6JWX6JFD3WHREEGRO` validates
the integrated commit with this exact ordered list (337 native tests):

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test editing --test commands --test coordinates --test files`
- `python3 scripts/check-source.py`
- `git diff --check`

PASS: focused controller evidence `JUHABZU63SQ2BX4KTIRZQHHWLB` records
this list on the same integrated commit (13 acceptance tests, two owned PTYs):

- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-pty.py --suite whitespace`
- `git diff --check`

## Risks and Follow-ups

None recorded.
