# Slice 17: Create absent user configuration safely

Status: reviewing
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
- `tests/configuration/startup_publication.rs`: isolated native startup and process races.
- `src/config/publication/tests.rs`: synchronized publication, IO faults and unwind cleanup.
- `scripts/check-pty.py`: owned HOME/XDG isolation and standalone working-directory support.
- `scripts/configuration_ui/support.py`: exact startup filesystem expectations.
- `docs/reference/defaults.md`: current creation, preservation and fallback behavior.

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
`cargo test --test configuration startup::` passed all 11 startup groups,
including four independent child processes released together before native
startup. They load one complete winner; the fixture has a five-second rendezvous
and reaps all owned children. Existing read-only helper assertions remain intact.

Development connected configured/missing/rejected/workspace cases passed all
six sessions. Configured and rejected sources retain their original complete
filesystem snapshots; missing-root cases permit exactly the two configuration
parents and the complete 10,013-byte sample. Text/cursor checks account for the
existing six-cell gutter and its separator without dropping source, Unicode,
mode, geometry, shared-state or resize assertions. Startup observations await a
complete frame. Project fixtures whose tree order is tested keep XDG publication
outside that project. The eight-second PTY deadline, 1-MiB output ceiling,
32-entry/64-KiB snapshot bounds, idle CPU checks and terminal cleanup remain.

`python3 scripts/check-startup.py` passed four sessions, including a copied
standalone editor run from an owned directory containing no references.
Development gutter-geometry and all six pointer suites passed 28 sessions,
retaining source/save/history and event ownership assertions. All native process
fixtures isolate HOME and all XDG roots; the shared PTY owner supplies owned
locations when a caller supplies none. Native performance producers receive
explicit isolated locations without changing their workloads or budgets.

Final canonical and focused evidence is recorded through the controller on the
clean candidate after task and PR-reference commits. Required ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration --test files --test recovery --test commands`
- `python3 scripts/check-configuration-ui.py --suite terminal`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused final commands:

- `cargo test --lib config::publication::tests`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-startup.py`
- `python3 scripts/check-pty.py --suite gutter-geometry`
- `python3 scripts/check-pty.py --suite pointer-lifetime`
- `python3 scripts/check-pty.py --suite pointer-focus`
- `python3 scripts/check-pty.py --suite pointer-scroll`
- `python3 scripts/check-pty.py --suite pointer-selection`
- `python3 scripts/check-pty.py --suite pointer-autoscroll`
- `python3 scripts/check-pty.py --suite pointer-controls`
- `git diff --check`

## Risks and Follow-ups

Publication requires hard-link support within the configuration directory;
unsupported or failed operations report their stage and retain usable defaults.
