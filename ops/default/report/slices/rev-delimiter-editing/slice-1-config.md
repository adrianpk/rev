# Slice 1: Configuration and language policies

Status: delivered
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `feat/rev-delimiter-editing-01-config`
PR: [#55](https://forge.adrianpk.com/rev/rev/pulls/55)

Integration: canonical PR #55 merged into `dev` at
`a5507c375908f499de0a369e6e10314c79d70a0a`, the same published commit that
passed the exact three required commands and canonical PR check. Verified
against the canonical forge and fetched `origin/dev`. This closure changes
delivery Markdown only; combined acceptance remains pending later slices.

## Purpose

Deliver the approved five editing preferences and native buffer policy without
automatic source edits or future presentation settings.

## Delivered Behavior

Global editing leaves default true. Partial global and singular language tables
retain absent siblings and merge explicit values with independent origin paths.
Immutable snapshots resolve language policy independently of grammar availability.
Buffers share the resolved policy across their views. Explicit automatic-editing
choices survive settings application and save-as language changes; new buffers
inherit their own resolved defaults. Policy changes do not edit source, publish
history, change read-only state or write settings files.

## Implementation Notes

The existing typed loader, candidate validation, merge and snapshot owners retain
configuration admission. No discovery path or dependency is added.
Both pair groups require actual TOML tables: a map-only deserializer prevents
Serde from accepting an empty array as an all-absent struct. Unknown leaves and
wrong types retain the existing whole-candidate diagnostic/fallback behavior.
The buffer keeps its resolved base separate from its optional enabled override;
the existing language lifecycle recomputes the base and retains that override.

## Contracts Added or Changed

`pairs` and `language.<id>.pairs` admit only `enabled`, `wrap_selection`,
`delete_empty`, `skip_closing` and `quotes`, each a Boolean.
The packaged template and configuration catalog list only these editing leaves.
Future highlight, pulse and pair-palette settings remain rejected. No command,
binding, source-edit behavior or personal configuration file is changed.

## Files of Interest

- `src/config/pairs.rs`: partial presence, compiled defaults and leaf overlay.
- `src/config/resolve.rs`: immutable effective policy.
- `src/buffer/policy.rs`: shared buffer policy and explicit override lifetime.
- `tests/configuration/pairs.rs`: admission, inheritance and lifecycle regressions.
- `docs/reference/config.toml`, `docs/reference/defaults.md`: supported schema.

## Validation

Focused working-tree checks passed:

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration pairs -- --test-threads=1`: 11 passed, none skipped; 21.56 seconds of test execution after a 56.30-second rebuild.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration -- reference::sample_defaults template:: --test-threads=1`: 5 passed, none skipped; 2.98 seconds of test execution.
- `git diff --check`: passed.

The pair suite covers each of the five false overrides, global and language
leaf inheritance/provenance, frozen snapshots, grammar-independent resolution,
40 wrong-type leaf cases, 13 malformed/future/related candidates and 10 invalid
included assignments masked by later valid values. Three save-as cases cover
inherited, explicitly enabled and explicitly disabled state. Split views and
existing edit/mark/point/read-only/undo/redo state remain intact. Owned fixtures
check unchanged configuration bytes; the existing personal configuration's
SHA-256 remained unchanged during these checks.

The initial cold focused run compiled in 122 seconds and passed 10 tests but
failed the empty-array group case. Strict table admission corrected the defect;
the unchanged regression then passed. Existing staged-source hooks passed for
all three task commits, including workspace formatting, Clippy and native source
policy (1,921 first-party Rust files after adding the regressions).
The vendored crossterm parentheses warning is pre-existing and non-blocking.
Final clean-HEAD evidence is recorded separately through the controller validator
after report and PR-reference commits. The combined delivery gate remains
controller-owned and has not run for this slice.

The initial clean-HEAD batch passed all four listed checks, but the canonical PR
check reported `required validation command order differs from configuration`:
the optional template check interrupted the frozen three-command prefix. The PR
now lists exactly formatting, the required pair-policy test and whitespace, in
that order. The five supplemental template/default results remain recorded above.
The corrected report commit receives fresh evidence for that exact required
sequence before handoff; the earlier receipt is not reused for a changed HEAD.

## Risks and Follow-ups

Automatic typing, deletion, wrapping and command dispatch belong to later slices.

## Delivery-set closure

This slice's delivered status records its own canonical integration and focused
checks. The controller's unchanged combined gate passed at
`eb65b227dee8b42bf8d0c8a6e0ef12f5675c756b`, receipt
`QD5ALW2WJR64VCCTK4CJAOZPDK`. The
[Slice 7 closure](slice-7-acceptance.md#delivery-set-closure) records integrated
native/PTY/resource and documentation results. Later-slice editing and physical
acceptance described above are complete; conservative uncertainty/refusal policy
is preserved. Closure changes only approved Markdown paths in a later dev commit;
the tested runtime, settings, fixtures, scripts and assertions remain unchanged.
