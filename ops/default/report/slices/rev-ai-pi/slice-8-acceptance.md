# Slice 8: Joint acceptance and guidance

Status: delivered
Delivery set: rev-ai-pi
Plan: `ops/default/plan/rev-ai-pi.md`
Tracker: `ops/default/tracker/rev-ai-pi.md`
Branch: `feat/rev-ai-pi-08-acceptance`
PR: `#236`

## Purpose

Connect the delivered native AI/file owners to reproducible installed-Pi and
terminal acceptance, and guide users through reload and explicit AI editing.

## Delivered Behavior

The offline driver exercises both startup profiles through native sessions and
real controlling PTYs. Installed Pi 1.3.0 performs RPC, guarded read/list/search,
compatible process reuse and private JSONL reconstruction with a local provider
that makes zero HTTP/model-provider requests. Tool assertions compare actual
returned source bytes, list/search contents and six access denials. A visiting
unsaved Zig source overrides disk; ordinary editing starts no Pi or AI store.

Atomic replacement triggers real kernel notifications. Native reload advances
the source revision, rejects an earlier AI proposal and permits a fresh proposal
as one undoable native insertion without saving. PTYs retain another command's
draft during reload, reject stale application and restore terminal attributes.
Private restart restores earlier instruction context, reset replaces only owned
history, and idle expiry/disconnect/editor exit reap owned children. Missing Pi
keeps ordinary edit/undo available.

The User Guide adds ordered reload and Pi chapters, connected to existing
references and the Spore navigation. Commands, configuration, Unreleased and
developer acceptance guidance describe the delivered behavior and prerequisites.

## Implementation Notes

The Bash entrypoint pins a native installed executable, refuses updater wrappers,
builds with two jobs and uses temporary HOME/XDG/Pi roots. Internal JavaScript
runs inside Pi; no separate Python/Node/npm contributor build requirement or
package installation is introduced. PTYs reuse the existing eight-second and
1 MiB owner; native journeys have a 35-second outer guard. Only owned fixture
processes and roots are cleaned up after checks.

The driver initially advanced after a child PID disappeared but before its
supervisor released the operation slot. It now waits for both owned exit and
native cleanup, and asserts instruction-input admission before submitting text.
The product's stopping/retry contract and source assertions remain unchanged.
Actual initial history creation also exposed open-before-create/stat-after-create:
the first file can appear between those observations. The history owner now
reopens through the same no-follow, current-user, private-mode, single-link
admission. Already observed history identities and required final history remain
strictly checked; no corrupt or substituted history is adopted.

The separate explicit live mode is excluded from the repeatable gate. It uses
fresh input-only history, no tools, a four-turn/eight-call guard, a 45-second
request deadline and an outer one-minute guard. It never prints authentication
or response content.

History correction: `47928e8b376385049bf249ff76fd6e8299a6447d`.
Acceptance/report introduction: `b8fadd21cef5f9b8c34f3737965cc0ca908f289a`.

Guidance commit: `297a8314f9d1` (full identity in Git).
Canonical PR: [#236](https://forge.adrianpk.com/adrianpk/rev/pulls/236).

## Contracts Added or Changed

T8.1 connects EFC01-EFC08 and PI01-PI08 at their actual notification, native
editing, subprocess, tool and private-history boundaries. Shared AI targets and
saved definitions keep their existing owners. The documentation checker admits
the two new ordered chapters and still validates exact catalogs, reachability
and local anchors. No new runtime dependency or provider fallback is added.

## Files of Interest

- `scripts/check-ai-pi.sh`, `examples/ai-pi-acceptance.rs` and
  `examples/ai_pi_acceptance/`: isolated offline journeys, real Pi local-provider
  assertions, measurements and separate bounded live mode.
- `src/ai/pi/history.rs`: strict admission across initial file creation.
- `docs/tutorials/user-guide/{13-file-reload,14-ai-editing}.md`: ordinary guided
  reload, proposal acceptance/undo, context, history and saved instructions.
- `docs/how-to/terminal-journeys.md`, reference/index links and
  `publication/userguide/index.sptree`: reproducible checks and canonical guide
  projection; no deployed site or copied Markdown owner.

## Validation

Successful focused commands:

- `cargo check --locked --example ai-pi-acceptance`
- `cargo clippy --locked --example ai-pi-acceptance -- -D warnings`
- `cargo test --locked --test pi sessions:: -- --test-threads=1`
- `cargo run --locked --quiet --example documentation-check`

Six session cases passed, including actual installed-Pi retained history under
the pinned executable. Owned debug native/PTY journeys passed in both profiles.
Documentation passed 429 commands, 231 configuration entries, 70 reachable pages
and 434 local links/anchors. Final release/script and exact ordered clean-head
checks belong to the controller receipt recorded after PR-reference commits.
No integrated delivery-set gate is claimed here.

Development observations on this host, separate from the final release receipt:

| Observation | Non-modal | Modal |
| --- | --- | --- |
| Native startup | 105065 us | 198820 us |
| First actual-Pi offline result | 1208 ms | 1529 ms |
| Native atomic notification/reload | 54 ms | 42 ms |
| First-result editor peak RSS | 28324 KiB | 29432 KiB |
| Separate Pi peak RSS at first result | 172048 KiB | 169736 KiB |
| PTY startup | 267 ms | 413 ms |
| PTY input-preserving reload | 42 ms | 63 ms |
| PTY output | 50089 bytes | 50099 bytes |

Native/PTY editor idle windows of 350 ms used zero CPU ticks and PTYs emitted
no output. Separate Pi idle observations used two/one CPU ticks and eight/zero
voluntary context switches. These are measured process activity, not invented
kernel wakeup counts or an RSS/latency guarantee. The existing editor idle
admission stays unchanged. Actual persistent Pi children report the 33554432-byte
OS file-size limit; the production index/history bounds remain in force.

### Initial real-provider observation

One operation was attempted on 2026-11-03 with the installed Pi and its existing
configured `openai-codex/gpt-5.6-sol` default. Authentication form: Pi-owned OAuth;
read-only metadata showed its recorded expiry predating this attempt. The fresh
owned project contained only a generated toy Rust function. Pi returned
`Pi provider did not complete; check Pi authentication/model and retry`.
The guard recorded one model turn and zero tool calls, within the operation
limits. No second request, alternate-provider matrix, credential export, account
change or manual credential mutation was performed.

This is a failed real-provider observation, not successful authenticated model
acceptance. The error does not by itself distinguish authentication, model or
provider-service failure. API-key variants were not exercised; controlled local
providers do not establish their availability. This attempt left T8.2 externally
blocked. The operator-authorized continuation below resolves that blocker while
retaining this failed observation.

### Authorized real-provider continuation

The operator decision for retained incident 1 authorized one additional operation
following renewed OAuth, preserving the one-minute, four-turn and eight-tool-call
limits, existing configured model, generated toy input and no fallback/further
requests. The original incident anchor and failed observation remain intact.

On 2026-11-03T08:49:48Z the unchanged release acceptance driver ran on integrated
commit `dbdb11b78c76f3d39c7cb3df6542b064f90a83ba`. The real configured
`openai-codex/gpt-5.6-sol` operation passed with Pi-owned OAuth in 2873 ms:
one model turn, zero tool calls, 44 response bytes, fresh input-only history and
unchanged generated toy source. Owned Pi cleanup completed with no remaining
fixture processes. No fallback or further provider operation was used. API-key
variants remain unexercised. Acta event `01M6G9M4ZBQFX4G7C93FD0VXXJ` records the
completed observation. This completes T8.2 independently of the offline gate.

## Risks and Follow-ups

The controller-owned frozen integrated gate passed and the delivery set is
complete. Its repeatable model evidence stayed offline. The broader native API contract
remains draft; its transport and context construction stay deferred.

## Integration

Canonical PR [#236](https://forge.adrianpk.com/adrianpk/rev/pulls/236) merged
into dev at `dbdb11b78c76f3d39c7cb3df6542b064f90a83ba` on 2026-11-03T07:45:06Z.
Receipt `RJLI2OQ42YTAU7K373I7PFAIKG` passed that exact clean head and all seven
configured commands in order:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --test external_changes --test ai --test pi -- --test-threads=1`
- `bash scripts/check-ai-pi.sh`
- `cargo run --locked --quiet --example documentation-check`
- `bash scripts/build-user-guide.sh`
- `git diff --check`

All 61 suite cases passed (25 AI, 21 external-change, 15 Pi), plus the owned
save child case. The release joint script passed native and PTY journeys in both
profiles in 241 seconds including build. Release native startups were 33950/60806
us; first-result editor RSS was 13716/14848 KiB and separate Pi RSS was
172492/171332 KiB. Native reload measured 10/10 ms; PTY startup 72/86 ms and
input-preserving reload 22/22 ms. The release editor measured 21474384 bytes.

Gate prerequisites are available: pinned native Pi 1.3.0, locked Cargo cache,
Rust/Cargo, Bash, util-linux PTYs, GNU timeout and Spore with local guide assets.
The gate's own fixtures create isolated roots and reap their owned children.
The successful live operation is recorded separately and will not be repeated
by the frozen gate. Slice 8 is delivered; the full gate passed as recorded below.

## Delivery-set closure

The frozen controller full gate passed at immutable candidate
`ecd02abcbd1f08530d8fad1395672dc6d00ee085` in receipt
`HBFDUZX54JK3RJUBGSQYIJANGO`, from 2026-11-03T08:58:36Z to
2026-11-03T09:04:09Z (approximately 333 seconds). Its exact successful command was:

- `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --lib --test files --test configuration --test external_changes --test ai --test pi -- --test-threads=1 && cargo test --locked --test completion --test lsp --test bookmarks -- --test-threads=1 && cargo test --locked --test modal contexts:: -- --test-threads=1 && cargo test --locked --test commands palette::catalog_reference -- --exact && bash scripts/check-ai-pi.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check`

The selected top-level suites passed: 129 library, 75 files, 147 configuration,
21 external changes, 25 AI, 15 Pi, 57 bookmarks, 44 completion, 46 LSP,
18 modal contexts and one command-catalog case (578 total, zero failed/ignored).
Six owned child subcases also passed. Format, strict all-target lint and both
profile variants of the real installed-Pi offline/native/PTY script passed.
The script measured 18 seconds including its warm build; the full gate did not
repeat either real-provider observation.

| Integrated release observation | Non-modal | Modal |
| --- | --- | --- |
| Native startup | 38923 us | 42170 us |
| First offline Pi result | 955 ms | 939 ms |
| Native notification/reload | 10 ms | 8 ms |
| First-result editor peak RSS | 13660 KiB | 15068 KiB |
| Separate Pi peak RSS at first result | 180520 KiB | 171884 KiB |
| PTY startup | 64 ms | 76 ms |
| PTY input-preserving reload | 22 ms | 22 ms |
| PTY output | 50090 bytes | 50100 bytes |

Editor idle windows retained zero CPU ticks and PTY silence. Separate Pi
350-ms observations used two/one CPU ticks and five/four voluntary context
switches; full kernel wakeup counts remain unavailable. The release editor
measured 21474384 bytes. Catalogs passed 429 commands and 231 configuration
entries, with 70 reachable pages and 434 local links/anchors. Spore produced
the local User Guide. Owned native/PTY/Pi fixtures completed their cleanup;
unrelated services and processes were preserved.

All eight reports are delivered and tickets solved. The plan/tracker and the
Pi/external-file specification/model pairs are Completed for EFC01-EFC08,
PI01-PI08 and selected shared AI behavior. The original failed provider
observation, incident and authorized successful OAuth continuation stay distinct.
Native API transport/context support remains deferred in its unchanged draft
contract/model. Only configured documentation closure paths differ from the
tested candidate.
