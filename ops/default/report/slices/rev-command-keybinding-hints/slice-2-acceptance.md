# Slice 2: Acceptance and guidance

Status: delivered
Delivery set: rev-command-keybinding-hints
Plan: [Command Keybinding Hints Delivery](../../../plan/rev-command-keybinding-hints.md)
Tracker: [Command Keybinding Hints Tracker](../../../tracker/rev-command-keybinding-hints.md)
Branch: `test/rev-command-keybinding-hints-02-acceptance`
PR: [#295](https://forge.adrianpk.com/adrianpk/rev/pulls/295)
Integrated: `3952b5915ec7048ec7892e4e51c61ec3094e7f5d`

## Purpose

Verify KBH01-KBH05 together through native acceptance and one real terminal
journey per editing profile. Document effective hints in the existing command
reference and learning path while preserving UI02 and CMR01-CMR05 evidence.

## Delivered Behavior

- Native cases check factory and configured maps, normalized Meta spellings,
  shortest/lexical choice, eight-chord admission, unbound and registered AI
  identities, all Org owners and inherited/shadowed leaves and prefixes.
- Advertised sequences are replayed through the real input resolver in the
  same origin. Session cases cover all six modal states, Visual cancellation,
  tree aliases, local Git ownership and both-profile live terminal consumption.
- Presentation checks cover Unicode cells, the exact two-cell gap, complete
  hint omission, zero geometry, resize, scrolling, secondary colors and
  NO_COLOR reverse video. Names alone control filtering, Tab and usage.
- One Linux PTY journey per profile verifies configured hints, narrow/restore
  layout, named and bound argument dispatch, an unbound command, unchanged
  source/configuration, exactly-once named usage and terminal restoration.
- Canonical Markdown explains primary and absent hints with annotated rows.
  The existing Spore flow projects the guidance locally. UI02 incorporates
  the approved amendment without replacing its historical completion evidence.

## Implementation Notes

T2.1 is implemented in `2a66698097f1a66ec1efbf18dc61c9f33c4990f7`.
The new acceptance example reuses the existing completion surface, input and
PTY helpers. Variants remain native; only the two profile journeys use real
editor processes. The Bash entrypoint builds before journey deadlines and
isolates HOME and all XDG roots in owned temporary fixtures.

The shared terminal helper retains its stricter eight-second lifetime and
1 MiB observed-output limit. Each entrypoint run additionally has a 30-second
outer deadline. Journeys wait for observable frames, check terminal attributes
and reap owned children. Native Git initializes an owned temporary repository;
no live repository, external provider, language server or private file is used.
The script removes its fixture on exit and verifies cleanup.

T2.2 is implemented in `266f6384283bbdc9c791d1c2c9250d510bfd3023`.
The commands reference owns the detailed contract, the existing search-and-
commands chapter teaches discovery, and the prompts reference links the
changed annotation behavior. UI02 retains its original candidate/gate and the
separate ranking evidence. Generated Spore output stays under ignored target
paths; no publication or installation is performed.

T2.3 introduces this report in `543c5324ca45fbdf83c31fe272a3516b0c781f94`.
Canonical [PR #295](https://forge.adrianpk.com/adrianpk/rev/pulls/295) targets `dev` and
links that full immutable introduction; its following reference commit records
the canonical review without changing runtime or acceptance assertions. The previous slice was closed and this slice activated in
`134b5daa3e9bbc1e46a8b6a8b2eb5f3797b6e7dc`, after live Forgejo confirmed
PR #294 merged at `05365b43a51f5aaa7217c489117afc78289b0abe`.

## Contracts Added or Changed

- `scripts/check-command-keybinding-hints.sh` owns the bounded combined
  acceptance entrypoint selected by the controller's final gate.
- Existing command-usage tests compare canonical candidate names independently
  of annotation padding, retaining all ordering, persistence, startup, Tab and
  dispatch assertions.
- UI02 now references effective originating-context hints, shortest/lexical
  choice, complete cell-safe annotations, identity separation and bounded
  snapshot inspection. This reconciles the approved amendment.
- No runtime, dependency, configuration, persistence or model contract changes
  are introduced by this slice.

## Files of Interest

- `examples/command-keybinding-hints-acceptance.rs` and
  `examples/command_keybinding_hints_acceptance/`: native matrix and PTY journeys.
- `scripts/check-command-keybinding-hints.sh`: isolated fixtures and bounds.
- `tests/command_usage/session.rs`, `tests/command_usage/startup.rs`: canonical
  usage assertions with annotated selector rows.
- `docs/reference/commands.md`, `docs/reference/prompts.md`,
  `docs/tutorials/user-guide/04-search-and-commands.md`: contextual guidance.
- `ops/default/spec/editor-interaction.md`: UI02 reconciliation.

## Validation

The six required Slice 2 commands passed during implementation; the final
clean-HEAD ordered receipt belongs to the PR-reference commit:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test command_usage -- --test-threads=1`
- `bash -n scripts/check-command-keybinding-hints.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

All 15 command-usage tests pass, including real startup/shutdown, retained
ranking snapshots, corrupt-state fallback and bounded/concurrent publication.
All-target Clippy passed in 70.279s during preparation. The successful usage
rerun took 4.825s including compilation; its test runtime was 3.75s.
Formatting, Bash syntax, documentation and whitespace preparation checks took
3.850s, 0.002s, 1.638s and 0.006s respectively.

Focused build and guide commands also passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked --bin rev --example command-keybinding-hints-acceptance`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`

Documentation validation passed 596 commands per combined inventory, 289
configuration entries, 85 reachable pages and 578 local links/anchors. The
local Spore build projected all 85 pages with strict link validation in 3.767s.
Generated command reference and learning chapter contain the annotation text,
row examples and working effective-keybinding-hints anchor.

The final isolated native run passed in 8.780s, including a strengthened
retirement assertion over all command identities after a hint-only query has
no candidates. The two focused PTY journeys passed as follows:

| Profile | Outer elapsed | Journey output | Idle CPU ticks | Peak RSS |
| --- | --- | --- | --- | --- |
| non-modal | 0.977s | 29,637 bytes | 0 | 29,804 KiB |
| modal | 1.061s | 29,637 bytes | 0 | 31,848 KiB |

Both journeys restored terminal attributes and all owned fixtures were removed.
These are local Linux measurements, not portable resource guarantees.

Initial fixture corrections preserved the acceptance contract: Insert/Replace
cases now override inherited one-chord competitors before expecting their
chosen hint; PTY candidate lookup compares complete fields because a narrow
query label can scroll. The required usage suite initially failed five old
full-row comparisons; name-field assertions fix those expectations without
changing persistence or dispatch checks. The successful reruns retain the
same bounds and do not disable tests.

## Risks and Follow-ups

The controller passed the frozen integrated full gate, including the combined
acceptance script, as recorded below. Evidence is local Linux native/PTY and
Markdown/Spore validation; it does not establish behavior on every physical
terminal or deployed guide. Successor startup remains controller-owned.
No main promotion, release, tag, mirror or public guide publication is included.


## Integration

Canonical PR #295 merged into dev on 2026-11-06 at
`3952b5915ec7048ec7892e4e51c61ec3094e7f5d`. Receipt
`AFRA7O7QCHCSKEGJ3IXL7ZQO43` passed the six ordered Slice 2 commands on that
exact clean candidate. Its immutable report introduction remains
`543c5324ca45fbdf83c31fe272a3516b0c781f94`.

Both recorded slices are delivered. The tracker records tool, target, syntax,
input and output prerequisites verified before the controller gate. Closing
these document entries changes the dev candidate; no prior slice receipt is
claimed as full-gate evidence for that later commit. The controller subsequently
validated the integrated candidate below; the delivery is Completed and the
ticket solved with resolution fixed.


## Delivery-set closure

Completed: 2026-11-06. Canonical PRs #294 and #295 are merged into dev and both
reports are delivered. The controller passed the unchanged full gate on clean
immutable candidate `ca39b8f14e03d3753f071811b1151b8299bc3560`, receipt
`BV6KPQKMFAI37JMO2HBOQOISIM`. The supplied receipt and log retain the exact
successful command:

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 && cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --test commands --test modal --test display --test command_usage -- --test-threads=1 && cargo test --locked --test configuration keymaps -- --test-threads=1 && bash scripts/check-command-keybinding-hints.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check
```

The receipt spans 19:50:39.338071521Z to 19:58:21.398665094Z on 2026-11-06:
462.061 seconds total. All 362 selected tests passed, with no failures or
ignored tests. The 149 other configuration tests were intentionally filtered
by the approved keymaps selection. Cargo reported these stage timings:

| Stage | Result | Reported time |
| --- | --- | --- |
| All-target Clippy | passed | 55.28s |
| Four-suite test compilation | passed | 52.04s |
| Command usage | 15 passed | 3.75s |
| Commands | 172 passed | 217.53s |
| Display | 44 passed | 11.47s |
| Modal | 120 passed | 95.38s |
| Configuration compilation | passed | 2.58s |
| Configuration keymaps | 11 passed | 3.85s |
| Acceptance editor/driver build | passed | 1.22s |

Formatting, native acceptance, documentation, Spore and whitespace also passed;
the log does not provide separate elapsed measurements for those stages. Their
previous implementation timings are not reused as measurements of this gate.

The native map/origin/identity, actual dispatch, modal-state, Visual-cancel,
Unicode geometry, color and NO_COLOR matrix passed. Both real input/render
journeys passed with their original eight-second helper lifetime, 1 MiB
observed-output cap and 30-second outer deadline excluding compilation:

| Profile | Journey time | Output | Idle CPU ticks | Peak RSS |
| --- | --- | --- | --- | --- |
| non-modal | 986ms | 29,637 bytes | 0 | 29,792 kB |
| modal | 1069ms | 29,637 bytes | 0 | 31,484 kB |

Each journey retained source/configuration bytes and exactly one canonical
named usage event; direct shortcut dispatch added none. Both idle samples
observed no output over 350ms. Ordinary finish restored terminal attributes
and reaped owned children. The wrapper removed its temporary fixture root.
Closure inspection found no acceptance temporary roots or active gate editor/
driver processes. No persistent service or preview was started.

Catalog validation passed 596 commands in each combined inventory and 289
configuration entries. Documentation passed 85 reachable pages and 578 local
links/anchors. Spore projected and built all 85 Markdown pages locally with
strict link validation. Generated output remains ignored local build output;
these checks establish no public deployment or universal physical-terminal
rendering guarantee.

Immutable report introductions:

- [Slice 1](https://forge.adrianpk.com/adrianpk/rev/src/commit/844ec6c5fb9d0c6f71d1de2ca6f98538423d8d12/ops/default/report/slices/rev-command-keybinding-hints/slice-1-effective.md), delivered by PR #294 at `05365b43a51f5aaa7217c489117afc78289b0abe`.
- [Slice 2](https://forge.adrianpk.com/adrianpk/rev/src/commit/543c5324ca45fbdf83c31fe272a3516b0c781f94/ops/default/report/slices/rev-command-keybinding-hints/slice-2-acceptance.md), delivered by PR #295 at `3952b5915ec7048ec7892e4e51c61ec3094e7f5d`.

KBH01-KBH05 are delivered with UI02 and CMR01-CMR05 preserved. UI02 was
reconciled before the gate and its historical validation identities remain
unchanged. The plan, tracker, specification and pending-work entry are
Completed; TKT-20261031095151 is solved with resolution fixed. Only the eight
configured closure Markdown paths differ from the tested candidate. Runtime,
tests, guides, changelog and every required acceptance assertion retain their
tested content. No main promotion, tag, release, installation, guide deployment
or mirror publication was performed.
