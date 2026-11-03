# Slice 1: One-shot commands

Status: delivered
Delivery set: rev-shell-commands
Plan: [Native Shell Commands Delivery](../../../plan/rev-shell-commands.md)
Tracker: [Native Shell Commands Tracker](../../../tracker/rev-shell-commands.md)
Branch: `feat/rev-shell-commands-01-execution`
PR: `#242`

## Purpose

Deliver explicit native one-shot command execution with bounded output, a captured
working directory, cancellation and inspectable plain-text results.

## Delivered Behavior

- Admit all five global startup shell preferences, with complete compiled/sample
  defaults, whole-array argument replacement, include provenance and whole-candidate
  fallback for invalid values.
- Run `shell-command` through the native command selector and argument prompt.
  The configured shell, startup SHELL or /bin/sh receives one literal command
  argument and immediate stdin EOF, without saving or sending source text.
- Capture the visiting file's parent directory, explicit directory for unnamed
  buffers or launch directory before prompting. Keep editing responsive while
  one owned invocation runs, and reject another until cleanup is reaped.
- Retain valid stdout separately from exit status/stderr in read-only text buffers.
  `shell-command-output` visits the retained identity; completion preserves focus
  and another prompt's echo. Re-owned or viewed older buffers are preserved.
- Cancel through `shell-command-cancel` or originating editing C-g. Deadline,
  invalid text, aggregate output overflow, leader exit and shutdown clean up
  the owned process group without admitting partial output as successful text.

## Implementation Notes

- `T1.1`: `37f9791f5516d8e9821b32c71e224031d4a9fc1f` admits configuration.
- `T1.2`: `dd6c1119bb472b8b73aab861e0ba2951ddd69a1a` adds native execution and presentation.
- `T1.3`: `15a57e4206679e4f71eafc885ab320d538b59a8b` adds real shell acceptance coverage and introduces this report.
- Promote the existing process, stream, group and cancellation owners to
  `src/process`. Existing Org callers retain their argv, resource, consent and
  cleanup policies; `org::Cancellation` remains a public re-export. LSP retains
  its own process owner and uses the promoted group-observation primitive.
- One temporary worker drains stdout/stderr fairly. The session owns native
  buffers and observes completion without blocking input. Its temporary 10 ms
  input deadline disappears after the worker is reaped.
- The native test driver waits for owned completion before inspecting a new
  result. Inspecting the last retained output while another invocation runs is
  valid behavior and does not establish that the new command completed.

## Contracts Added or Changed

- Add `shell.program`, `shell.args`, `shell.timeout_seconds`,
  `shell.max_input_bytes` and `shell.max_output_bytes` to typed admission,
  precedence, source origins, sample TOML and the default catalog.
- Add `shell-command`, `shell-command-cancel` and `shell-command-output` to the
  native catalog, handlers and command/binding references.
- Enforce one invocation, 4096 command bytes, finite configured time/output
  limits, strict UTF-8/NUL admission and checked retained result identities.
- Add one user-facing Unreleased entry for one-shot commands and results.

## Files of Interest

- `src/config/shell.rs`: typed preferences, bounds and projection.
- `src/process.rs`, `src/process/`: shared unchanged process lifetime primitives
  and their existing regression tests.
- `src/shell.rs`: explicit shell invocation and temporary worker ownership.
- `src/session/shell.rs`: captured directory, prompt, cancellation and results.
- `tests/shell_commands.rs`: real process/session behavior and isolated startup
  environment subprocesses.
- `tests/configuration/shell.rs`: defaults, includes, provenance and admission.

## Validation

Successful development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration shell:: -- --test-threads=1`: passed, 2 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo check --locked --lib`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands -- --test-threads=1`: passed, 9 tests including isolated startup environment children.
- `git diff --check`: passed.

The native checks cover exact Unicode/newline/control text, stdin EOF, captured
file directory and unsaved-source preservation; status and separate stderr;
aggregate boundary/overflow and invalid UTF-8/NUL; command admission; simultaneous
streams; cancel/deadline/descendant cleanup; session drop; prompt ownership and
return to idle; startup shell fallback, explicit override, argv and environment.
An initial process promotion compile identified one remaining viewer import;
the import was updated before the successful compile.

The first controller batch on `77a8726806973ae06029c574fb33e49a462eac27`
passed formatting, strict all-target Clippy, 151 configuration tests, 9 shell
tests and 7 process-filtered library tests. Its documentation check identified
three missing command inventory rows in `docs/reference/bindings.md`; those rows
now describe named invocation without assigning the later slice's default keys.
The failed receipt is retained; successful checks are not relabeled for a newer
candidate.

The final ordered Slice 1 prefix is recorded through `looper validate` after the
PR-reference follow-up commit and push. The canonical PR Validation section and
controller receipt own the exact clean candidate and ordered successful checks.
This report does not claim the integrated delivery-set gate.

Canonical PR #242 was integrated by the controller into dev at
`5e1622b82ec342e502b12a08792ce45e50d41532`. Controller validation
`2SHYGPQSSZE6UZAXEFY7T3LYUK` passed the exact clean candidate before integration:

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands --test configuration -- --test-threads=1`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`: passed.
- `git diff --check`: passed.

The focused suites passed 151 configuration, 9 shell and 7 process-filtered
library tests. The canonical PR contract check also passed.

## Risks and Follow-ups

The remaining approved slices add region/edit publication, keys/history and joint
native/PTY/resource acceptance. This slice adds no dependencies or Cargo.lock
changes. Process groups enforce lifetime ownership rather than a sandbox.
