# Slice 6: Private Pi sessions

Status: delivered
Delivery set: rev-ai-pi
Plan: [Pi and Reload Delivery](../../../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../../../tracker/rev-ai-pi.md)
Branch: `feat/rev-ai-pi-06-sessions`
PR: `#234`

## Purpose

Retain explicitly isolated directory history through Pi's actual session owner,
with native controls for root/model selection and bounded process lifetime.

## Delivered Behavior

- Managed directory operations restore only history matching the canonical root,
  root identity, literal configuration/grant observations and resolved provider/model.
  Pi owns the private JSONL transcript and reconstruction; native code owns only
  the strict schema-1 binding index and source/result authority.
- Session-local ai-set-directory and ai-select-model use native path/name prompts.
  Models come from Pi's reported catalog. Neither control writes configuration,
  workspace roots, source text or authentication data. M-x and modal colon share
  the five new controls, including ai-reset-session, ai-disconnect and ai-cancel.
- Compatible directory children remain available until the configured idle
  deadline; the supervisor blocks on a notification/deadline, without an editor
  AI idle polling timer. Disconnect, expiry and editor exit stop/reap owned
  children. A later explicit invocation restores compatible private history.
  Input-only operations always use fresh --no-session children.
- Each new invocation admits a fresh native snapshot map, nonce, deadline and
  tool/turn/return budgets. Historical observations remain historical. Compaction
  and an internal retry of the identical envelope cannot reset those budgets.
  Root/model/grant changes retire the child before a new explicit request;
  active/stopping work never hides a queued replacement.
- At most 16 bindings, a 64 KiB index and 32 MiB per history are admitted.
  Malformed/unknown schemas, unsafe paths/modes, oversized state, competing leases
  and conflicting publication fail explicitly and preserve unknown history.
  Disabling persistence permits in-memory directory context without guessing a
  different persisted session. Explicit reset replaces only the matched owned
  binding and retires its verified old history after publication.

## Implementation Notes

T6.1/T6.2 land together in `ad97bb9e569c5858eb0f3e33a3ef36fd1618d581`: private-history
admission, the RPC mailbox and native active/idle ownership form one usable
process transition. T6.3 cases land in `7ef7b03396602642ca99a408101e867d2b017c78`.
The implementation uses existing standard-library/rustix/toml_edit owners;
no dependency or credential store is added.

The index records binding ID, canonical root, literal grant fingerprint,
resolved model, relative session path and created/last-used Unix timestamps.
Current-user private 0700 directories and 0600 regular single-link files are
checked through retained directory handles and no-follow opens. A short index
lock serializes compare-before-publish atomic sync/rename; a per-binding lease
covers active and idle history use across cooperating editor instances.
The persistent owned Pi child also receives a 32 MiB OS file-size limit.
Native code inspects JSONL metadata only, never parses or edits its transcript.

Startup resolves the actual model in a fresh private candidate, chooses the
matching binding, asks Pi to switch history when required, and verifies its
reported model/path and effective tool admission before generation. Fresh
input-only/catalog operations use no retained history. Cancellation uses the
existing finite abort/stop/reap owner, and cleanup failure blocks replacement.
A completed result can stay inspectable after disconnect or idle expiry.

## Contracts Added or Changed

PI01-PI04/PI07/PI08 and AI16-AI20 cover private binding identity, process lifecycle,
Pi-owned model/history behavior, per-invocation bounds and source authority.
The existing native result target, explicit application and one-step undo owners
remain authoritative across reused history. The reference catalog now contains
428 commands; persistence/idle settings now govern managed directory operations.

## Files of Interest

- `src/ai/pi/{history,worker,io,protocol}.rs`: private metadata/leases, restored
  history admission, blocking idle mailbox and fresh generation accounting.
- `src/ai/pi/discovery.js`: renewed grants and unchanged-envelope retry handling
  without resetting compaction-era budgets; private lazy history creation.
- `src/session/ai/{controls,capture,results}.rs`: native prompts, session-local
  root/model choices, reset/disconnect ownership and truthful result provenance.
- `tests/pi/sessions.rs`, `tests/ai/sessions.rs`: isolation, storage bounds,
  concurrent leases, native input ownership and actual child cleanup.
- `tests/ai/discovery-provider.js`: test-only local provider driving real Pi
  context reconstruction/tool execution, with no external provider request.
- `docs/reference/{commands,bindings}.md`, `docs/reference/config.toml`,
  `CHANGELOG.md`: delivered controls, private paths and idle/persistence policy.

## Validation

Successful focused commands:

- `cargo clippy --locked --lib -- -D warnings`
- `cargo test --locked --test ai --test pi sessions:: -- --test-threads=1`
- `python3 tests/ai/discovery.py "$REV_PI_EXECUTABLE"`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

All nine focused session cases passed: six Pi and three editor cases. They cover
changed roots/root identities, configured and resolved models/providers, grants,
16-binding capacity/reset, 64 KiB/32 MiB bounds, corrupt/schema/permission/link
rejection, competing history leases, independent index updates, controlled
compaction notifications and non-resetting turn limits, publication conflict,
owned idle/explicit exits and native prompt ownership in both profiles.

REV_PI_EXECUTABLE selected the literal installed compiled Pi 1.3.0 binary,
without its updating launcher. The retained-history native case uses actual Pi
with a validation-only local provider: two operations retain one PID; disconnect
reaps it; a new child restores the same owned JSONL and its earlier instruction
in the model context. Each operation executes 19 real tools, proving renewal of
per-invocation tool budgets across reuse/restart. The child reports its 32 MiB OS
limit. This evidence proves actual local Pi history reconstruction and owned
exit, rather than inferring semantic reuse from a file path alone.

The separate actual offline Pi resource probe passed 27 checks, including an
identical-envelope hook revisit halfway through the tool-call budget followed by
rejection of a 33rd call. Compaction notification tests use a controlled peer;
these checks do not claim a forced full compaction with an authenticated model.
External provider requests: zero. Documentation passed 428 commands, 230
configuration entries, 68 reachable pages and 414 local links/anchors.

Final clean-head evidence is recorded through the controller after all
PR-reference commits, using exactly the four configured commands in order.
The canonical PR and receipt own that candidate; focused checks do not claim
the integrated delivery-set gate.

## Risks and Follow-ups

Admitted source/history may be transmitted to the chosen Pi model; private path
exclusions do not classify arbitrary contents. Locks coordinate cooperating
instances, and the configured executable/resource remain trusted process code.
Authenticated-provider acceptance and the integrated gate remain with their
recorded delivery stages. Saved reusable command definitions remain Slice 7.

## Integration

Canonical PR [#234](https://forge.adrianpk.com/adrianpk/rev/pulls/234) merged
into dev at `d4a5c4c90dda580fdad612c91478189ae14ba644` on 2026-11-03T01:01:25Z.
Receipt `Y5YWQGRNOAKJ3ADM3Q6VBHY532` passed the exact clean head and configured
ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --test pi --test ai -- --test-threads=1`
- `git diff --check`

All 37 native tests passed (22 editor AI, 15 Pi). The actual offline Pi evidence
remains focused local evidence; this closes Slice 6, not the integrated gate.
