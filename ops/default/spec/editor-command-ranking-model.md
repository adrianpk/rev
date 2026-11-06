# Command Usage Model

Status: Completed
Completed: 2026-11-06
Validated candidate: `fea4fffdfa3e66d8a7d7f1cc031a6637199424af`
Gate validation: `342LB2K3MYK2Z2JS5ESVI4WSNP`
Date: 2026-11-06
Approved: product owner, 2026-11-06; specification/model and one autonomous Looper delivery through dev
Specification: [Command Recency and Frequency](editor-command-ranking.md)
Plan: [Command Ranking Delivery](../plan/rev-command-ranking.md)
Tracker: [Command Ranking Tracker](../tracker/rev-command-ranking.md)
Baseline model: [Configuration](editor-configuration-model.md)

## Location and scope

The normal executable resolves the existing XDG state root. The file is
`$XDG_STATE_HOME/rev/command-history.toml`, falling back to
`$HOME/.local/state/rev/command-history.toml` under the current root rules.
It is user-wide operational state, not configuration or project content.

Resolution and loading do not create directories or files. After a recorded
invocation, the state owner may create missing parents and publish pending
events. Embedded sessions and tests receive an explicit isolated state owner;
pure Input/Session construction must not discover or write the real user's
history. A stable sibling lock file may serialize publication without a daemon.

## Version 1 representation

```toml
version = 1
commands = ["find-file", "toggle-file-tree", "find-file"]
```

| Field | Shape and meaning |
| --- | --- |
| `version` | Integer `1`; an unknown version is not loaded or overwritten. |
| `commands` | Array of zero to 256 canonical command-name strings, oldest first and newest last. Repetitions represent separate accepted invocations. |

Each name is bounded to 128 ASCII bytes and uses the current command-name
syntax: a lowercase letter followed by lowercase letters, digits or hyphens.
Implementation acceptance must verify that all compiled command names fit this
storage bound. Current AI names, including their `ai-` prefix, also fit it.
Deserialization rejects unknown fields, invalid types and malformed names.
Bound file input and serialized output to 64 KiB before publication. Oversized
input is a rejected record, not an unbounded parse followed by truncation.

Names need not currently resolve: a removed command is retained as bounded
historical data and ignored when ranking admitted candidates. Persist no enum
discriminants, command arguments, prompt drafts, paths, buffer contents, model
responses, timestamps or separate permanent frequency totals.

## Derived values

For each admitted command, derive its count and last position in the retained
window. Recency uses descending last position; frequency uses descending count,
then descending last position, then canonical name. Each ranking pass emits a
command identity once. Count is at most 256; position comes from the bounded
array, so no persistent sequence counter or clock is required.

The four recent and four frequent slots are compiled policy values. A query
first partitions the admitted candidates by the specification's match ranks.
The selector captures usage when it opens and applies ranking within those
ranks. Publication cannot mutate that selector's chosen command identity.

## Session-history compatibility

`minibuffer.history_limit` continues to bound distinct accepted names in the
existing M-p/M-n session history, with current default 128. It does not control
the new 256-invocation frequency window. `minibuffer.candidate_limit`, default
eight, still limits visible rows rather than stored events.

Session history contains distinct names; the usage window contains repeated
events. They have different purposes and units. This capability does not persist
argument recall or change existing session-history navigation into cross-session
replay. No new configuration leaf or migration of configuration files is needed.

## Publication and failure model

The in-memory usage window and unpublished event batch each have a 256-event
ceiling; a single state owner coalesces publication rather than accumulating
unbounded jobs. If unpublished events exceed the ceiling, retain their newest
256, which also displace older saved events when published.

For one publication, acquire the stable advisory lock without waiting, read the
latest valid bounded record, append the pending batch in order, retain the
newest 256 events and atomically replace the file from an owned sibling temporary
file. Preserve ordinary user-only permissions. Serialization establishes batch
ordering between concurrent publishers, without pretending to recover a total
wall-clock order between independent sessions.

Clear a batch after confirmed replacement; do not append it again on a later
publication. Ambiguous replacement outcomes require verification or suspension
of persistence for that instance, rather than blind retries that duplicate
counts. A busy lock leaves the batch pending for a later bounded attempt. A
malformed or unsupported disk record blocks publication without discarding or
rewriting that record. Ordinary editing and in-memory usage remain available.

An absent file starts empty. A complete default-only editor startup need not
create the state root. An error diagnostic must not reproduce prompt contents
or introduce a modal decision before ordinary command use or shutdown.

## Completed delivery evidence

Three canonical slices through PRs #291, #292 and #293 implement the version 1 usage model.
The controller passed its unchanged full gate at `fea4fffdfa3e66d8a7d7f1cc031a6637199424af`,
receipt `342LB2K3MYK2Z2JS5ESVI4WSNP`: 503 selected tests, native frame/surface
ranking, both-profile input/render/restart journeys, paired real publication,
complete catalogs and the local Markdown-derived Spore guide build.
The [Slice 3 closure](../report/slices/rev-command-ranking/slice-3-acceptance.md#delivery-set-closure) records the exact command,
immutable report introductions, stage/resource observations and evidence limits.
The interaction and configuration baselines were reconciled before that gate;
their earlier validated candidates and historical results remain unchanged.
The separate keybinding-hint ticket, main promotion and publication retain their
existing boundaries.
