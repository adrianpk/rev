# Slice 8: Joint acceptance and guidance

Status: reviewing
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

### Single real-provider observation

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
providers do not establish their availability. T8.2 remains externally blocked
pending a working existing default provider/authentication and controller-directed
continuation within the approved operation budget.

## Risks and Follow-ups

The mandatory repeatable gate remains offline for model requests. Successful
live-provider acceptance is outstanding; keep its failure visible when deciding
readiness. The controller owns integrated validation and delivery-set closure.
The broader native API contract remains draft. No release or site publication
is claimed.
