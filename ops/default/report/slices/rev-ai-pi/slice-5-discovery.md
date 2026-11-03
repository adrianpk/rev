# Slice 5: Guarded directory context

Status: delivered
Delivery set: rev-ai-pi
Plan: [Pi and Reload Delivery](../../../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../../../tracker/rev-ai-pi.md)
Branch: `feat/rev-ai-pi-05-discovery`
PR: `#233`

## Purpose

Let Pi select relevant project context through a strictly admitted read-only
resource while native buffers and checked result acceptance retain authority.

## Delivered Behavior

- Builtin explanations, prompts and implementation requests use the explicitly
  opened tree root, visiting file parent or launch directory, without repository
  ascent. Results name the admitted root and captured source revision.
- The embedded JavaScript resource registers only rev_read, rev_list and
  rev_search. Pi's automatic extensions, MCP, skills, templates and context-file
  discovery remain disabled. No shell, write tool or separate runtime install
  is admitted. Directory operations currently use fresh ephemeral history.
- A native directory descriptor pins root identity across scheduling and version
  probing. The resource checks the canonical root, no-follow component opens,
  descriptor containment and source identity before/after IO. Symbolic/hard links,
  special files, invalid UTF-8, binary/NUL data and files over 1 MiB reject.
- Current visiting source and dirty peer snapshots within the root are captured
  at instruction submission. At most 16 complete snapshots fit the native input
  budget; credential/private paths reject before transmission. Reads and search
  prefer these revisions over disk, including admitted snapshots of missing
  files. Subsequent operations refresh them. Later edits still fence acceptance.
- Listing offers literal path matching and finite recursion. Search is literal,
  uses no shell or relevance engine and labels native versus disk observations.
  Results expose ranges, completeness and skipped/bounded work. Pi chooses
  discovery requests and owns the model/context loop.
- Each invocation keeps its eight-turn, 32-tool and 4 MiB returned-text budgets
  through compaction. Tools reserve 256 KiB before IO, return bounded chunks,
  and explicitly abort exhausted grants. Native correlation/accounting also
  rejects broader tools, duplicate/incomplete executions and invalid content.

## Implementation Notes

T5.1/T5.2 land together in
`bee7889f191cd961de5f275b24ca7cc26d375aa7`: resource, effective-profile admission
and native envelope/snapshot capture form one usable process transition. That
commit also includes focused native boundary cases. This report introduction
adds T5.3 actual-runtime probes and the final root provenance assertion.

Only an explicit operation extracts the embedded resource into a unique owned
private XDG cache directory. File/directory modes are 0600/0700, code bytes are
checked before launch, and verified owned extraction retires after child cleanup.
An independent handled-command acknowledgement and nonce-bearing notification
must both attest the three-tool profile before fresh state and a model prompt.
This uses documented RPC prompt/extension notification records, not new RPC fields.

Linux descriptor paths anchor component opens; each regular disk read checks
its retained and reopened observations. Known credential components, private
editor directories and Pi state paths are excluded. These path rules do not
classify arbitrary source contents as safe to transmit. The configured executable
and packaged resource are trusted code; cwd is not an OS sandbox.

Read chunks contain at most 32768 Unicode scalars. Traversal visits at most 4096
entries and 16 nested levels; listing returns at most 256 entries. Search scans
at most 128 files/4 MiB and returns bounded literal-match excerpts. The 4 MiB
return limit includes encoded observation text, and conservative capacity
reservation may end work earlier. Native revision strings preserve integer
identity across the JavaScript boundary. No native index/provider transport,
source writer or retained directory-session owner is added here.

## Contracts Added or Changed

PI02-PI05/PI07 and AI06/AI09/AI11-AI13/AI17/AI20 cover enforced directory
read/transmit grants, source provenance, input/result/resource bounds and
native target authority. The public input-only operation still starts without
extension/tools/history. Automatic resource discovery never repairs a denied
request. Cancellation and child cleanup retain the earlier finite RPC owner.

## Files of Interest

- `src/ai/pi/{discovery.js,resource.rs,directory.rs}`: guarded filesystem tools,
  private extraction and native transmission exclusions.
- `src/ai/pi.rs`, `src/ai/pi/protocol.rs`: pinned root, effective-profile
  handshake, correlated tool events and non-resetting native budgets.
- `src/session/ai/{capture,context,results}.rs`: fresh native observations,
  context admission and original-root/result provenance.
- `tests/pi/discovery.rs`, `tests/ai/discovery.rs`: owned subprocess/native
  transitions, source-map admission, stale disk precedence and root replacement.
- `tests/ai/discovery.py`, validation-only JavaScript peers: actual Pi 1.3.0
  registration/tool execution, deterministic filesystem races and local-model
  event generation without external provider requests.
- `docs/reference/commands.md`, `CHANGELOG.md`: current directory behavior,
  exclusions, budgets and native acceptance.

## Validation

Successful focused commands:

- `cargo clippy --locked --lib -- -D warnings`
- `cargo test --locked --test ai --test pi discovery:: -- --test-threads=1`
- `cargo test --locked --test pi discovery::canonical_root_admission -- --exact --test-threads=1`
- `python3 tests/ai/discovery.py "$REV_PI_EXECUTABLE"`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

Six focused native cases passed (two editor, four Pi), including hidden dirty
sources, refreshed revisions, excluded/overfull maps, private byte-exact resource
extraction/retirement, rejected admission/tool/result boundaries and replacement
of a root during delayed startup. A canonical-parent-alias case also rejects
before any child/extraction; canonicalization cannot redefine the granted root
while a supervisor starts. The literal installed compiled Pi 1.3.0 binary
was selected through REV_PI_EXECUTABLE, without invoking its updating wrapper.

The offline probe passed 27 checks. It uses actual Pi registration/runtime and
real owned filesystem IO. Command probes cover traversal, links, private stores,
missing/binary/invalid/large sources, Unicode/CRLF data, native precedence, file/root
replacement while a descriptor is open, call/return limits and owned exit.
A separate validation-only local provider drives Pi's actual agent/tool runner:
three admitted tools execute, a final result settles, resource markers stay out
of the effective prompt, and continuation stops before a ninth model call.
External provider requests: zero. This is actual local Pi/tool evidence with
controlled model events, not authenticated model-generation evidence.

The probe found that throwing from turn_start alone only emits an extension
error and permits continuation. The resource now calls the actual context abort
owner on a poisoned grant; the eight-call runtime assertion passes. An initial
no-follow duplicate of a descriptor path rejected its proc symlink; opening its
final directory component preserves no-follow source checks and passes the
actual runtime probe. Focused strict lint findings were corrected before success. Late canonical
resolution was additionally fenced by requiring the original canonical name
both at pinning and worker admission; its focused rejection test passed.
Catalog validation passed 423 commands, 230 configuration entries, 68 pages and
414 links/anchors.

Final clean-head evidence is recorded through the controller after PR-reference
commits, using exactly the four configured commands in order. The canonical PR
and controller receipt own that candidate; focused checks do not claim the
integrated delivery-set gate.

## Risks and Follow-ups

The admitted root must contain source suitable for the chosen model. Private
path exclusions are not content classification. Persisted isolated directory
history remains with Slice 6; authenticated provider acceptance remains with
the recorded joint acceptance. No such evidence is inferred from local fixtures.

## Integration

Canonical PR [#233](https://forge.adrianpk.com/adrianpk/rev/pulls/233) merged
into dev at `9d16dc77b4371c161b1ea6716f31fea767b5393a` on
2026-11-03T00:11:56Z. Receipt `OLNWBRUOJRSR4SWXPZISRJSPLN` passed the exact
clean head and ordered configured commands:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --test pi --test ai -- --test-threads=1`
- `git diff --check`

All 28 native tests passed (19 editor AI, 9 Pi); the 27 actual offline Pi checks
remain focused local evidence. This closes Slice 5, not the delivery-set gate.
