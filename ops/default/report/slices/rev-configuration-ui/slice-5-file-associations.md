# Slice 5: Identify buffer languages

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-05-file-associations`
PR: [#54](https://forge.adrianpk.com/adrianpk/rev/pulls/54)

## Purpose

Identify a buffer's language from deterministic filename associations while
keeping editing preferences independent of grammar availability and canonical
file destinations independent of visiting names.

## Delivered Behavior

- Recognizes all 23 built-in basename/suffix rules in the accepted model;
  unmatched names select plain. Matching is literal and case-sensitive.
- Gives any matching user rule priority over built-ins, then selects an exact
  basename or the longest suffix. Identical patterns retain normal include/root
  precedence; distinct overlapping suffixes remain valid.
- Resolves identity without opening a file, enumerating directories, following
  symlinks or discovering configuration. Lossless OS-name bytes prevent lossy
  labels from changing a match.
- Retains the first visiting basename and its display label when a symlink
  resolves to a differently named canonical destination. Further alias visits
  reuse that buffer without changing its identity, text or language selection.
- Keeps unknown identities and Go editing overrides independent of parsers.
  Snapshot editor/grammar queries inherit global siblings and default to plain
  syntax except for the existing bundled Rust identity.
- Selects built-in syntax from the visiting identity and preserves alias names
  across ordinary saves without adding a disk-change confirmation.

## Implementation Notes

- T5.1: `35b6faeb78529d279ebbbbd01a53499e38f030d7` adds the complete
  built-in table and deterministic association selection with two focused tests.
- T5.2: `0cd0ebf2291267ad7bd509f20a7cfaaae16bbbaa` retains visiting names,
  exposes buffer language queries and tests alias/grammar/save behavior.
- T5.3: `4c59dd552f6a0ac3cfcbefe45c7c80b3ccae7d83` adds source-layer,
  literal Unicode/raw-name and observation-free
  resolution cases and this report. All seven new focused tests pass, plus the
  adapted syntax association regression.
- Rule selection scans the admitted mapping and a fixed built-in table. Exact
  matches rank above suffix lengths; different equally long suffixes cannot
  match one basename. Selection does not depend on map iteration order.
- Canonical paths remain the buffer index and save destination. Locations
  separately retain lossless basenames; generated display-label suffixes do not
  participate in association matching. Fresh save observations retain the first
  basename when the destination identity is unchanged.
- Directory relocation preserves the visiting basename; a changed destination
  filename updates it. Effective editing policy installation and path-change
  policy transitions remain assigned to their recorded slices.

## Contracts Added or Changed

- `Snapshot::language(&Path)` selects an identity from the final visiting
  basename without filesystem access or grammar installation.
- `Location::basename()` exposes the lossless visiting basename separately from
  `path()`, the canonical destination used for buffer reuse and saves.
- `Buffer::language(&Snapshot)` queries a supplied frozen configuration without
  mutating text, history, grammar state or the snapshot.
- Built-in syntax selection follows visiting identity. The prior syntax test
  that required canonical-target recognition now asserts the accepted alias
  rule while retaining rename, save and history invariants.
- The Unreleased changelog records the observable alias behavior.

## Files of Interest

- `src/config/associations.rs`: built-ins and user/exact/suffix selection.
- `src/file/location.rs`: visiting basename and canonical identity separation.
- `src/buffer/visits.rs`, `syntax.rs`, `save.rs`, `save/decisions.rs`: alias
  labels, snapshot language queries and name retention during save admission.
- `tests/configuration/associations.rs`: catalog, rule layers, literal names
  and queries against absent, directory and dangling-symlink names.
- `tests/files/associations.rs`: Go/unknown policies, cross-suffix aliases,
  shared identity, generated labels and unchanged-target saves.
- `tests/syntax/edges.rs`: visiting-name syntax and destination changes.

## Validation

Development checks:

- `cargo test --test configuration associations::`: passed; five tests.
- `cargo test --test files associations::`: passed; two tests.
- `cargo test --test syntax association_modes`: passed; one adapted regression.
- `cargo clippy --lib --test configuration --test files -- -D warnings`: passed.

The controller recorded all seven ordered checks passing on clean head
`95923e869370a44de581ed34003fc228549049ef`, validation
`YT7IW6CWG4D6XZCCRM2HQUUHXW`: 54 configuration, 62 file and 21 syntax tests.
Canonical PR #54 integrated that exact head into dev by Rebase + Fast-forward.
Required commands precede the affected syntax regressions and whitespace check:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `cargo test --test files`
- `python3 scripts/check-source.py`
- `cargo test --test syntax`
- `git diff --check`

## Risks and Follow-ups

Buffer editing/display policy installation remains assigned to Slice 6 and
path-change policy transitions to Slice 9. Configured syntax preferences remain
assigned to Slice 15; runtime configuration activation remains assigned to
Slice 20. This slice resolves supplied snapshots without loading user settings
into the running session.
