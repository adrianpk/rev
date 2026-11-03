# Slice 2: Checked text filters

Status: delivered
Delivery set: rev-shell-commands
Plan: [Native Shell Commands Delivery](../../../plan/rev-shell-commands.md)
Tracker: [Native Shell Commands Tracker](../../../tracker/rev-shell-commands.md)
Branch: `feat/rev-shell-commands-02-filters`
PR: `#243`

## Purpose

Send captured canonical regions to native shell commands and publish complete
successful stdout to the original checked region or insertion point.

## Delivered Behavior

- `shell-command-on-region` captures a nonempty linear selection before prompting,
  supplies its exact unsaved UTF-8/newline bytes as stdin followed by EOF, and
  retains output without changing source. Absent, empty and rectangular selections
  reject; byte admission precedes snapshot allocation and staging.
- `shell-filter-region` uses the same snapshot and replaces only the captured
  range. `shell-command-insert` uses null stdin and inserts at its captured point.
  Valid complete successful stdout uses one ordinary native edit and undo step.
- Empty replacement deletes the region. Empty insertion preserves revision and
  history. Unicode, BOM data, CRLF and trailing whitespace are preserved exactly.
- Before launch and publication, check source identity/revision/range, visiting
  association and writable policy. Closure/reopening, edits and undo, reload,
  pending discard consent and external disk changes revoke automatic application.
- Retain valid stdout separately from status/stderr and rejection diagnostics.
  Completion maps live source views without taking focus or restoring an obsolete
  point; ordinary undo restores the command's original initiating source point.

## Implementation Notes

- `T2.1`: `494aa35190f30f4b80210ce6fb1a6f3437a87f2e` adds region capture,
  bounded stdin and native display. Its insertion/replacement portion completes
  together with T2.2.
- `T2.2`: `6f491a31ce9aa89d547951e757b1e0bd16c03d53` adds checked publication,
  source/reload fences and the real-child readonly-transition regression.
- `T2.3`: `071b89040dd1833f4b79779d1186d6192c3a7c9e` adds the complete native filter cases and introduces this report.
- Anonymous sealed memory-file staging reuses the existing process helper's
  file-input path. The input descriptor reaches finite EOF without a concurrent
  pipe writer, implicit save or filesystem persistence. Existing dependencies,
  process groups, cancellation and aggregate stdout/stderr bounds remain in use.
- The worker performs strict result decoding and a final visiting-file observation.
  The session rejects changed disk evidence and pending observations/reload consent;
  native external-change policy keeps its existing authority.
- The observed-edit helper accepts an explicit originating view. Shell publication
  supplies its captured view for undo; the AI caller supplies its existing selected
  view. Live pane points are mapped by the shared owner without replacing focus.
- Complete stdout is retained before publishing source. Only stdout enters the edit;
  status, stderr and failure explanations remain in diagnostic buffers.

## Contracts Added or Changed

- Add the three approved region/insert commands to the native command and binding
  inventories, with the existing native prompt and named invocation paths.
- Enforce `shell.max_input_bytes` on canonical region bytes before copying/staging.
- Extend checked result publication with original-view undo identity and explicit
  source-specific pending observation/reload guards.
- Keep the selected six-command/five-preference profile and existing dependencies.
  Revise the existing Unreleased entry to describe checked region and insertion use.

## Files of Interest

- `src/shell/input.rs`, `src/shell/target.rs`: finite stdin and native target capture.
- `src/shell.rs`: invocation snapshot, result decoding and final disk observation.
- `src/session/shell.rs`, `src/session/shell/publication.rs`: native capture,
  completion, checked edits and retained results.
- `src/file/watch.rs`, `src/session/observation.rs`, `src/session/reload.rs`:
  source-specific observation and reload fencing without consuming consent.
- `src/window/points.rs`, `src/session/ai/results.rs`: explicit undo origin through
  the shared observed-edit owner and its existing AI call semantics.
- `tests/shell_commands/regions.rs`, `tests/shell_commands/filters.rs`: real
  shell/session regression cases.
- `src/session/shell/publication/tests.rs`: actual-child completion after a host
  readonly-policy transition, which requires access to the session's owned buffer.

## Validation

Successful development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands regions:: -- --test-threads=1`: passed, 3 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands filters:: -- --test-threads=1`: passed, 8 tests before the final two regression cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands -- --test-threads=1`: passed, 22 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands regions::region_snapshot -- --test-threads=1`: passed with real sort/tr and exact unsaved text.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands filters::output_edits -- --test-threads=1`: passed with real tr, empty and literal-output variants.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::shell::publication -- --test-threads=1`: passed, 1 actual-child readonly-transition test.
- `git diff --check`: passed.

The cases cover exact unsaved input and shell-looking data, Unicode/newlines,
byte/shape admission, duplex drainage, sort/tr, partial range preservation,
readonly display/edit rejection, empty/literal output, nonzero/invalid/overflow/
deadline/cancellation failures, prompt cancellation, original-pane undo and moved
points, edited/undone/closed/reopened/reloaded targets, a command writing its
visiting file and pending reload-discard consent. Real processes and native source
owners exercise these contracts without protocol stubs.

An early pane driver assumed split changed focus. It now explicitly selects the
other pane; assertions for distinct view identity, retained focus, original-source
publication and ordinary undo remain in place.

The first clean-candidate readonly receipt `6EHRQ6MZZ6LG7HXYZ5SPH2OAGF`
passed on `29a38140dc9c1e2e97351c4f8c9255085c29dcaa`. The configured batch
`TVQGAUKRGZDS545K7G7FTZIK3E` then failed strict Clippy because the captured
shell request made `Pending::Shell` a large enum variant. Store that prompt request
in `Box`, matching existing large native prompts, without changing admission or
publication. Both controller receipts are regenerated after this correction;
prior results remain tied to their original candidate.

Canonical PR [#243](https://forge.adrianpk.com/adrianpk/rev/pulls/243) merged
into dev at `ae496d8f79ea864233d64c8c7d43e2a898e0d7bc`.
Controller receipt `4PQQM2NGXFIIZ6A2R6WYQG4VUX` passed all six configured
commands in order on that exact clean head: formatting, strict all-target Clippy,
22 shell and 21 external-change tests, 5 native AI publication tests, documentation
validation and `git diff --check`. Focused receipt `LSFGO3J6QTLAMXN5UZML5KAFFD`
passed the real-child readonly-transition case on the same head. The immutable
report introduction is `071b89040dd1833f4b79779d1186d6192c3a7c9e`.
The integrated delivery-set gate remains controller-owned.

## Risks and Follow-ups

The remaining approved slices add default keys/history and joint profile, PTY and
resource acceptance. The existing disk-observation model remains conservative
metadata evidence; it cannot eliminate changes after an observation. No dependency
or Cargo.lock change is introduced.
