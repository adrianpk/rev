# Slice 3: Acceptance and guidance

Status: delivered
Delivery set: rev-command-ranking
Plan: [Command Ranking Delivery](../../../plan/rev-command-ranking.md)
Tracker: [Command Ranking Tracker](../../../tracker/rev-command-ranking.md)
Branch: `test/rev-command-ranking-03-acceptance`
PR: `#293`

## Purpose

Verify UI02 and CMR01-CMR05 together through bounded native and terminal
acceptance, complete user guidance and prepare the candidate for the
controller-owned integrated gate.

## Delivered Behavior

- Native acceptance checks actual candidate frames and selected surface colors
  for empty/short usage, overlap, four recent/four frequent groups, ties, all
  match policies, exact names, registered/removed AI names and 256-event eviction.
- Snapshot/resize, Tab, a configured accept alias, Return, cancellation, unknown
  names, session recall and internal-dispatch exclusion retain their contracts.
  Malformed, unsupported and oversized state remains intact while commands work.
- One input/render/restart journey per profile verifies passive startup, Tab
  without invocation, later argument cancellation, saved ranking across files,
  separate session recall, resize and exactly-once accepted dispatch.
- A paired real-editor check starts two executables against the same absent
  state before either accepts a command. After the first exits, the stale second
  owner rebases its accepted event onto the first's saved window, retaining both
  events once. Source/configuration bytes and terminal attributes remain intact.
- The User Guide shows an empty-query candidate preview and explains saved
  ranking versus M-p/M-n recall. References and UI02/configuration descriptions
  agree with the usage model; historical validation identities remain unchanged.

## Implementation Notes

T3.1 is implemented in `1a46d3e6f49baef20453597dc1a1298910c6efb6`.
T3.2 is implemented in `e467e2de7f2fcdba65669a6d6324ad516f0e6b16`.
T3.3 records this report and the tracker handoff; final set closure remains
controller-owned. Canonical [PR #293](https://forge.adrianpk.com/adrianpk/rev/pulls/293)
merged into `dev` at `676c3d9e2b80754fc01a7de0fc4cce1f2c9daf01`.
The report's immutable introduction is
`601f204ee78c67107287901d9b28603a467b7d3c`, which also records the T3.3 report/tracker handoff.

The Bash wrapper builds the editor and driver together once, then runs the
native matrix, two serial restart journeys and one paired publisher check.
HOME and every XDG root are isolated. The existing frame/surface, native session
and PTY helpers are reused without a new dependency or public runtime API.
At most two fixture editors are live. Each retains the shared eight-second and
1 MiB stream limits; each whole journey is additionally capped at 30 seconds,
excluding compilation. Ordinary finish verifies terminal mode restoration and
reaps the owned fixture. The EXIT trap removes and verifies its temporary root.
Idle checks observed zero CPU ticks and no output over their 350 ms sample.

The guide preview was projected from canonical Markdown into an ignored local
Spore project, strictly link-checked, rendered and checked for the ranking text.
There is no second authored guide source or publication step. Slice 2 is closed
against merged PR #292 at `ce48b3a699970eec94375ba116f100c4eedb949c`;
`cd5588ac6805ab51b3ee97d6be388b6ff3909d11` recorded that transition and activated Slice 3.

## Contracts Added or Changed

- `scripts/check-command-ranking.sh` is the bounded joint acceptance entrypoint;
  ranking variants stay native instead of spawning a PTY per variant.
- UI02 uses the shared recent/frequent ordering within text-match ranks. CFG17
  and the configuration model distinguish visible rows, distinct session names
  and the separately persisted fixed invocation window.
- Runtime commands, settings, dependencies and persistence schema are unchanged
  by this acceptance slice. The existing Unreleased product outcome remains the
  single command-ranking entry.

## Files of Interest

- `examples/command-ranking-acceptance.rs`,
  `examples/command_ranking_acceptance/native.rs`,
  `examples/command_ranking_acceptance/pty.rs`,
  `scripts/check-command-ranking.sh`: bounded native and real executable checks.
- `docs/tutorials/user-guide/04-search-and-commands.md`,
  `docs/reference/commands.md`, `docs/reference/prompts.md`,
  `docs/reference/defaults.md`: candidate preview and ranking/history guidance.
- `ops/default/spec/editor-interaction.md`,
  `ops/default/spec/editor-configuration.md`,
  `ops/default/spec/editor-configuration-model.md`: baseline reconciliation.

## Validation

All six required commands passed locally, in order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test command_usage --test commands --test modal -- --test-threads=1`
- `bash -n scripts/check-command-ranking.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 301 tests: command_usage 15, commands 168 and modal 118.
Successful command times were 2.85s formatting, 49.05s Clippy, 316.80s tests,
less than 0.01s Bash syntax, 1.22s documentation and 0.01s whitespace.
Documentation passed 85 reachable pages and 576 local links/anchors, with 596
commands and 289 configuration entries reconciled against live catalogs.

Focused checks also passed:

- `bash scripts/check-command-ranking.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example userguide-publication -- target/command-ranking-guide`
- `spore --project target/command-ranking-guide check --strict --links`
- `spore --project target/command-ranking-guide render --force --recursive`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example userguide-publication -- target/command-ranking-guide --finish`

The successful acceptance run took 7.64s including a 1.12s warm build. Profile
journeys took 1320 ms and 1495 ms, with 10,582 + 21,948 output bytes each;
paired publication took 437 ms with 2744 + 7922 bytes. All are below their
unchanged limits. No fixture directory remained after the run. Spore projection,
strict check, render and finalization took 0.53s, 0.56s, 0.77s and 0.10s.

The first cold acceptance build took 1m12s. Its initial PTY assertion assumed
that the command query occupied the bottom row; the actual vertical panel places
candidates below it. The helper now locates the rendered query and verifies the
candidate directly below. The corrected complete check passed without changing
runtime behavior, deadlines or assertions about dispatch and persistence.

Canonical clean-HEAD evidence belongs to Looper receipts for the final
PR-reference commit. These focused slice checks do not establish the
controller-owned integrated gate or final delivery acceptance.

## Risks and Follow-ups

Forced termination can lose unpublished events. Existing real-boundary tests
cover state contention, byte/name/event bounds and publication failures; these
representative PTY journeys do not multiply those variants into terminal runs.
All three canonical slice PRs are integrated. The controller passed the frozen
integrated gate; the permitted eight-path final closure is recorded below.
The slice-validation evidence above retains its separate candidate and scope.
Main promotion and publication remain separate.

## Delivery-set closure

Completed: 2026-11-06. Canonical PRs #291, #292 and #293 are merged into dev and
all three reports are delivered. The retained controller passed its unchanged
full gate on clean immutable candidate `fea4fffdfa3e66d8a7d7f1cc031a6637199424af`
under receipt `342LB2K3MYK2Z2JS5ESVI4WSNP`. The supplied receipt and log retain
the exact successful command:

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 && cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --test commands --test command_usage --test modal --test configuration --test display -- --test-threads=1 && bash scripts/check-command-ranking.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check
```

All five suites passed 503 tests with no failed or ignored tests: command_usage
15 (3.48s), commands 168 (206.77s), configuration 160 (98.04s), display 42
(10.84s) and modal 118 (91.18s). Logged Clippy time was 51.29s and test compilation
50.41s. The acceptance editor/driver build took 1.15s with cached artifacts.
The supplied log records stage timings, not an aggregate gate duration; no total
is inferred from these observations.

The native ranking/frame/surface matrix and both real input/render/restart
journeys passed. Non-modal/modal journeys took 1357/1469 ms and emitted
10,582 + 21,948 bytes each. Both idle samples observed zero CPU ticks and no
output over 350 ms; peak RSS was 29,468/30,640 kB. Paired real publication took
411 ms and emitted 2744 + 7922 bytes, retaining both instances' events once.
Each fixture remained below the unchanged eight-second and 1 MiB stream caps;
the wrapper additionally caps each entire journey at 30 seconds excluding
builds and allows at most two live editor fixtures. Ordinary finish verified
terminal attributes, cursor, alternate-screen and bracketed-paste restoration
and reaped owned children. The wrapper removed its temporary root; closure
inspection found no owned gate editor or acceptance directory. No persistent
service or preview was started.

Documentation validation passed 85 reachable pages, 576 local links/anchors,
596 commands in each combined inventory and 289 configuration entries. Spore
projected and built 85 Markdown pages locally. The output remains an ignored
local build; no deployment, physical-terminal rendering claim or publication
follows from this evidence. Forced termination can still lose unpublished events.

Immutable report introductions:

- [Slice 1](https://forge.adrianpk.com/adrianpk/rev/src/commit/1be68be2cb5ca095139d1cd9364f699e4a241e09/ops/default/report/slices/rev-command-ranking/slice-1-session.md), delivered by PR #291 at `17e16f07580cc32d743e6f8d34510dda14c85c2e`.
- [Slice 2](https://forge.adrianpk.com/adrianpk/rev/src/commit/397907bd342d83d54d83c13e7e0cb067fa2cb757/ops/default/report/slices/rev-command-ranking/slice-2-persistence.md), delivered by PR #292 at `ce48b3a699970eec94375ba116f100c4eedb949c`.
- [Slice 3](https://forge.adrianpk.com/adrianpk/rev/src/commit/601f204ee78c67107287901d9b28603a467b7d3c/ops/default/report/slices/rev-command-ranking/slice-3-acceptance.md), delivered by PR #293 at `676c3d9e2b80754fc01a7de0fc4cce1f2c9daf01`.

UI02, CMR01-CMR05 and the usage model are implemented. The plan, tracker,
specification/model and pending-work entry are Completed. Only the eight
configured closure Markdown paths differ from the tested candidate. Runtime,
tests, guides, samples, changelog and every gate assertion retain their tested
content. The separate keybinding-hint ticket remains open. No main promotion,
tag, release, installation, guide deployment or mirror publication was performed.
