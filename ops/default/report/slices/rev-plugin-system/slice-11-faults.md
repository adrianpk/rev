# Slice 11: Fault and resource acceptance

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `test/rev-plugin-system-11-faults`
PR: #11 - https://forge.adrianpk.com/rev/rev/pulls/11
Implementation: 56d4e0b62ce5d32b539c799c7507f6e591655794

## Purpose

Verify real package-process isolation, finite resource ownership and ordinary
native input through failures and bounded work, including representative PTYs.

## Delivered Behavior

The faults selection consolidates the existing malformed/oversized handshake,
startup deadline, stderr, descendant and failed-exec cases. It adds EOF after
initialization, repeated successful/restarted-generation descriptor checks,
a peer that stops reading and a finite valid-request flood that reaches the
outbound queue ceiling. Native text and an independent editor remain available.

Two native PTYs use the actual non-modal and modal selectors. The Rust package
creates and shows an ordinary result buffer, binds F8 locally, receives a held
command and then crashes on a new generation. Manual text survives cancellation,
retirement, native buffer navigation and explicit saving. Terminal attributes,
terminal mode restoration and owned editor exit are checked by the existing PTY
owner. Lower-level variants retain their existing tests.

## Implementation Notes

All changes are acceptance fixtures and delivery records. Runtime source,
dependencies, API/SDK identities and the finite profile remain unchanged. Five
existing lifecycle tests move into faults instead of being duplicated. Shared
fixture helpers retain their existing ownership and finite waits.

The descriptor case runs in an isolated test subprocess, samples the native FD
baseline, repeats failed execs and successful/failed restarts, and requires the
same FD count after settlement. The exec peer records only descriptors 0/1/2;
host eventfds, pidfds and unrelated files cannot cross exec. Descendant checks
include TERM-resistant members while a second editor's package remains alive.
Cleanup still reports escaped descendants as uncertain; it does not claim a
sandbox or ownership of unrelated processes.

Five fresh native PTYs measure baseline, installed-unused, active-idle,
large-read and event-load profiles. They reuse the existing util-linux PTY owner,
8-second lifetime, 1 MiB terminal-output ceiling, frame projection and Linux
CPU/RSS observations. Isolated child-only HOME/XDG roots avoid personal state.
The large-read case submits four 512 KiB reads while native typing continues;
the event case delivers 128 changed events. Both save their manual edits before
normal shutdown. Active idle uses a blocking SDK peer. Workload peers also use
a finite file-script sender; its polling overhead is included in their metrics.

## Contracts Added or Changed

- No product or wire contract changes. Existing limits and failure policies are
  exercised through real child pipes and the native executable.
- The 16-request ceiling and 4 MiB queued-output refusal remain enforced while
  ordinary input and independent editor instances continue.
- Settled failure waits include request-permit release and owned group cleanup;
  entering Failed alone is not used as proof that settlement has completed.
- PTYs validate qualified command discovery, local bindings, unsaved-result
  retention and native navigation/save through real terminal bytes.

## Files of Interest

- tests/plugins/faults.rs: consolidated failures, descriptor and process-group
  isolation, finite backpressure and unaffected independent editor.
- tests/plugins/faults/resources.rs: five native resource profiles using the
  existing bounded PTY and Linux observations.
- tests/plugins/faults/pty.rs: two representative terminal journeys.
- tests/plugins/peer.rs and lifecycle.rs: shared real Rust executable, inert
  descriptor observations and controlled failure/journey modes.

## Validation

All eleven frozen ordered commands passed locally:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — passed.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings` — passed.
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk` — passed.
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- faults --test-threads=1` — passed.
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — passed.
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1` — passed.
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1` — passed.
8. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1` — passed.
9. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1` — passed.
10. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check` — passed.
11. `git diff --check` — passed.

Results: 12 SDK tests; eight fault tests; 52 command, 114 editing, 110 macro,
45 modal and four completion regressions. Documentation checked 684 commands,
299 configuration entries, 100 pages and 685 local links. The frozen local run
took 618.27 seconds, including initial static/test compilation; the catalog
sweep (254.16 seconds) and modal selection (100.88 seconds) dominated runtime.
The fault target took 22.36 seconds, including existing real deadlines.

Focused checks also passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- protocol declarations lifecycle buffers transactions invocation keymaps recording events state faults --test-threads=1 --nocapture` — 84 tests passed in 95.84 seconds.
- `rustfmt --edition 2024 --check tests/plugins/peer.rs` — passed.

During fixture development, `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- faults --test-threads=1 --nocapture` initially failed.
Successful restart cycles were incorrectly using the missing-executable fixture; eight 512 KiB responses could exceed the 4 MiB queue budget; and an
early Failed observation did not yet prove request/cleanup settlement. The
fixtures now use a valid process for successful cycles, four bounded reads, and
explicit permit/group settlement waits. The valid-request flood still requires
the unchanged outbound-limit refusal. No runtime limit or assertion was relaxed.

Final clean-HEAD controller validation is a handoff prerequisite recorded outside
this report. No integrated full gate, merge or publication was performed.

Local resource sample from the successful 84-test focused run: Linux x86_64,
Rust 1.98.0, unoptimized Cargo profiles with debug information and incremental compilation disabled, one build job. The host
source is the unchanged integrated commit
2be4e880daf567c6aa52a127ccab4b851dd48953; API rev.plugins/0.1 and SDK/package
versions 0.1.0. The Rust peer is compiled against the same Cargo-built public SDK.
The final exact PR candidate receives separate clean-HEAD controller evidence.

| Profile | Launch to first frame, ms | Editor RSS / peak, KiB | Editor FDs | Peer RSS, KiB / FDs | Input to first marked frame, ms |
| --- | --- | --- | --- | --- | --- |
| Baseline | 378.309 | 35236 / 35236 | 8 | 0 / 0 | — |
| Installed-unused | 361.256 | 36136 / 36136 | 8 | 0 / 0 | — |
| Active-idle | 357.100 | 36684 / 36684 | 13 | 3996 / 3 | — |
| Four concurrent 512 KiB reads | 382.205 | 39016 / 40372 | 13 | 6092 / 3 | 13.702 |
| 128 changed events | 358.881 | 36948 / 36948 | 13 | 4036 / 3 | 14.640 |

Each fresh 350 ms quiet interval emitted no terminal output and consumed zero
editor CPU ticks (CLK_TCK=100). Across the preceding quiet barrier plus that
interval, editor samples were 0/0/1/0/0 ticks in table order; peer samples were
0/0/0/1/0 ticks. Installed-unused spawned no peer and used the same eight editor
FDs. Active idle used five additional editor FDs and a separate three-FD peer.
The event profile separately waited for all 128 event receipts. Input latency
ends at the first completed frame containing the typed marker, rather than at
completion of the entire workload. The two user journeys displayed native input
while a command was pending in 23.787 ms (non-modal) and 22.511 ms (modal). These are individual observations;
startup ordering, caches, fixture IO and unoptimized builds affect the values.

## Risks and Follow-ups

Resource observations are local unoptimized-build samples, not performance
thresholds or production promises. CPU counters have 10 ms resolution; a zero
sample does not establish zero cost. Linux /proc, util-linux script and the
existing shell PTY wrapper are fixture prerequisites. Unknown escaped
descendants remain explicitly uncertain. User guidance and independent-client
acceptance stay in their recorded slices. The controller owns integration,
the integrated full gate and successor startup.
