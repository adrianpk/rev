# Slice 17: Create absent user configuration safely

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-17-bootstrap-publication`
PR: pending

## Purpose

Provision the complete packaged configuration during native startup while
preserving every existing destination and read-only configuration helpers.

## Delivered Behavior

Normal session startup resolves the existing XDG locations and provisions an
absent implicit root. It creates only necessary configuration parents, writes
and synchronizes a complete private temporary file, then publishes with an
atomic no-clobber hard link. Existing entries and concurrent winners retain
ordinary validation. Failures retain complete built-ins and a source diagnostic.

## Implementation Notes

The native file owner's exclusive, mode-0600 temporary file and bounded cleanup
are reused. Configuration never invokes document replacement. Pure resolution,
loading, preparation and `Startup::from_env` remain read-only. Provisioning is
an explicit application startup operation before settings are frozen.

## Contracts Added or Changed

CFG01, CFG14, CFG15 and CFG24 admit one bounded absent-file startup publication.
No data, state or cache creation accompanies startup.

## Files of Interest

- `src/config/publication.rs`: complete no-clobber publication and cleanup.
- `src/config/startup.rs`: explicit provisioning followed by ordinary preparation.
- `src/session/startup.rs`: native startup ownership.
- `src/file/temporary.rs`: existing private temporary resource ownership.
- `tests/configuration/startup_publication.rs`: isolated native startup cases.

## Validation

`cargo test --test configuration startup::publication::first_start` passed
explicit-XDG and HOME-fallback native startup, complete packaged bytes, ordinary
validation, retained factory editing, repeat startup and passive other roots.
`python3 scripts/check-source.py` passed the initial publication owner.
`cargo test --lib config::publication::tests` passed four groups covering
six injected IO stages, a partial-write unwind, four synchronized real-filesystem
publishers and preservation/ordinary validation of custom or invalid concurrent
winners. Cleanup retains unrelated files. `cargo test --test configuration startup::publication`
passed absent-root XDG/HOME startup, seven existing-entry kinds and blocked or
unwritable parents. Entry identity, permissions, timestamps, exact bytes/link
referents and native editable defaults remain checked. Later sync/cleanup errors
retain the complete published file and still return a truthful failure.
Exact clean-HEAD evidence will be recorded after the remaining task.

## Risks and Follow-ups

Publication requires hard-link support within the configuration directory;
unsupported or failed operations report their stage and retain usable defaults.
