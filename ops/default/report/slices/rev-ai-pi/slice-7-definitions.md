# Slice 7: Reusable AI commands

Status: reviewing
Delivery set: rev-ai-pi
Plan: [Pi and Reload Delivery](../../../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../../../tracker/rev-ai-pi.md)
Branch: `feat/rev-ai-pi-07-definitions`
PR: pending

## Purpose

Store literal reusable AI policies in the existing configuration graph and invoke
them through native command/keymap dispatch with fresh source authority.

## Delivered Behavior

- ai.commands admits at most 64 complete typed records. Names contain 1..64
  lowercase ASCII letters/digits/hyphens, begin with a letter and reserve the
  compiled command/action namespace. Callable ai-name identities join the same
  M-x/colon matching, completion and history used by compiled commands.
- Every record requires literal prompt, input, context and output fields, with
  an optional Pi model. Prompts admit at most 16384 UTF-8 bytes and models 256
  bytes. Insert requires point input; replace requires selection or explicit
  whole-buffer input. Explanations remain read-only. No interpolation, executable
  handler, credential/endpoint override or new shortcut prefix is added.
- Complete definitions merge by name through normal include/root precedence.
  Definition admission precedes final keymap reference resolution, including
  references from an included keymap to a root definition. Existing context,
  prefix, cancellation and modal recovery restrictions remain enforced.
- Each invocation captures a fresh native point/span, revision and association.
  Input context always starts fresh no-tools/no-session RPC with no peer snapshot
  map, including after broader directory history. Directory context retains the
  guarded read/snapshot/history policy. A changed boundary stops the idle owner
  and requires an explicit retry after cleanup.
- ai-save-command captures the last successful instruction and its policy,
  asks for a new name, and publishes only that definition through SettingsEdit.
  Root/include changes, invalid graphs and competing writers reject publication.
  Includes, user comments, unrelated settings and other definitions are preserved.
- The chosen saved definition becomes callable immediately. Other disk changes
  and new persisted keybindings retain startup activation semantics. Cancellation
  writes nothing; ordinary editing/configuration admission starts no Pi child.
  Generated text, source snapshots and credentials never become a saved prompt.

## Implementation Notes

T7.1/T7.2 land together in `8f349aca77a480575a4738c4b5a43d05922e809a`. Typed record
admission, shared dynamic identity and scoped live activation form one usable
configuration-to-invocation pipeline. T7.3 native journeys land in
`9509b8d606a1933dc641e0fccd0c0ac3583a2b84`.

The bounded AiName value preserves stable copied identities without leaked
strings, global mutable registration or index reassignment when a later name
activates. Command/Binding names now borrow that value. Compiled inventories
remain explicit; only admitted saved names extend a session's palette.
Source-local parsing validates reference syntax/ownership, while complete graph
resolution rejects unresolved AI references before publishing a snapshot.

The existing target owner now admits explicit selection/point/buffer policies;
input context uses the existing fresh Pi operation, and directory context uses
its retained owner. Result acceptance/undo still use the canonical transaction.
The session retains only one bounded successful definition for explicit save.

The existing format-preserving publisher captures the complete graph, validates
its candidate, locks, compares source observations and performs checked atomic
publication. Newly absent namespace parents become ordinary implicit tables,
so subsequent definitions can use normal TOML table syntax. Existing inline
configuration remains valid and preserves its comment representation. Live
activation changes one definition and its origin; startup keymaps, interaction
profile, Pi settings and unrelated definitions are not reloaded.

## Contracts Added or Changed

PI01/PI06/PI08 and AI03/AI04/AI14/AI18 cover literal command storage, scoped
activation, fresh native target capture and unchanged checked result application.
The configuration/reference asset adds empty ai.commands defaults. There are
429 compiled commands plus at most 64 admitted saved identities.

## Files of Interest

- `src/config/ai.rs`, `src/command/ai_name.rs`: bounded typed definition data
  and stable callable identities.
- `src/config/{edit,resolve,validate}.rs`: chosen-record root publication,
  definition-before-reference admission and restricted live activation.
- `src/input/{palette,keymap}.rs`, `src/input/keymap/`: shared native catalog,
  profile keymaps and preserved context/cancellation constraints.
- `src/ai/target.rs`, `src/session/ai/{capture,save,results}.rs`: fresh native
  targets, narrower ephemeral context and successful-instruction save.
- `tests/configuration/ai_commands.rs`, `tests/ai/definitions.rs`: real
  configuration publication/conflicts and native command journeys.
- `docs/reference/{commands,bindings,defaults}.md`, reference config and
  `CHANGELOG.md`: usable definitions, bounds and startup binding semantics.

## Validation

Successful focused commands:

- `cargo clippy --locked --lib -- -D warnings`
- `cargo check --locked --tests`
- `cargo test --locked --test configuration --test ai definition -- --test-threads=1`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

Seven focused cases passed: four configuration cases and three native AI
journeys. Table-driven admission covers all input/context/output combinations,
required fields, invalid names/collisions, prompt/model/name/count limits,
merged-count excess, undefined references and restricted keymap contexts.
Real filesystem transactions cover root/include conflicts, competing captures,
comments, include bytes, inline tables and invalid-graph rejection.

Native journeys use the existing controlled RPC subprocess peer. Both profiles
exercise literal prompt/model transport, native selection/point/whole-buffer
application, one-step undo, fresh revisions and missing-selection rejection.
The narrower-context case observes actual old-child exit and a fresh no-tools,
no-session child with an empty snapshot map. Save tests isolate XDG state in an
owned test subprocess, preserve the successful instruction, cancel without a
write, reject a concurrent source change, activate only the selected definition,
and activate configured profile keybindings only after restart.

The save journey exposed sealed inline namespace creation. Creating absent
parents as normal implicit tables corrected publication while retaining the
same conflict/comment/activation assertions. No acceptance requirement or
resource limit was relaxed. External provider requests: zero. Controlled RPC
proves the stated native/transport boundaries; it does not claim authenticated
model generation or a new real-Pi compaction acceptance.

Documentation passed 429 commands, 231 configuration entries, 68 reachable pages
and 415 local links/anchors. Final clean-head evidence is recorded through the
controller after all PR-reference commits, using exactly the configured four
commands in order. The canonical PR and receipt own that candidate; focused
checks do not claim the integrated delivery-set gate.

## Risks and Follow-ups

Literal source and permitted directory context may be transmitted to the chosen
Pi model under the existing policy. Saved names do not promise identical model
results. Authenticated-provider/PTY acceptance and the integrated gate remain
with the recorded final acceptance stage.
