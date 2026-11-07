---
id: TKT-20261103112506
title: Present AI explanations and proposals as Org
status: solved
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: ai, org-mode, presentation
source: chat
reported_at: 2026-11-03T11:25:06Z
ready_at: 2026-11-03T12:27:17Z
started_at: 2026-11-03T12:27:17Z
branch: dev
reviewed_at: 2026-11-03T12:34:08Z
closed_at: 2026-11-03T12:35:29Z
resolution: fixed
commits: [d65e83d921ec53ffbe2d6f55ff2492def25248c6]
---

# Present AI explanations and proposals as Org

## Reported behavior

The technical header in AI result buffers dominates the answer and makes it
difficult to identify the actual explanation. Manual review of an Org preview,
with the explanation first and request details in a trailing REQUEST drawer,
was clearer. The owner also tested ai-implement successfully and requests the
same presentation for implementation proposals.

## Requested outcome

- Use a shared native Org presentation for explanations and editing proposals:
  a short source-named heading, the substantive answer, then request metadata
  inside a REQUEST drawer that starts collapsed.
- Keep model, context policy, input scope, source revision/range and history
  inspectable. rev supplies these facts from the captured request and result;
  the model must not invent the surrounding metadata.
- Keep operation type and actionable stale-source/error information legible
  without bringing back the verbose technical header above the answer.
- Let Org headings, lists, inline markup and blocks distinguish semantic parts
  of the explanation. Use examples and subsections when they help, rather than
  imposing the same template on every input.
- Present an implementation proposal inside an appropriate source/example
  block, using the target language when available. The accepted text remains
  exactly the generated payload, without the surrounding Org markup or drawer.

## Explanation instruction

Use software engineering as the default perspective, including code,
configuration, logs, diffs and technical documentation. Explain purpose,
behavior, relationships and non-obvious details rather than merely paraphrasing.

When the material is clearly outside that domain, infer an appropriate
explanatory perspective from the supplied content. Adapt the answer without
announcing a persona or maintaining a special-case catalog of document genres.
For a whole file, start with an overview and add useful sections; a short
selection may need only a short explanation.

Approved instruction direction:

```text
Explain the supplied material using a software-engineering perspective by
default. Explain its purpose, how its parts relate, and the details needed to
understand it; do not merely paraphrase it. If the material is clearly outside
software engineering, infer the relevant domain and adapt the explanation
accordingly without announcing a persona. Adjust depth and structure to the
input. Use clear Org markup and examples only where they improve understanding.
Leave request metadata to the editor.
```

The current built-in explain instruction is minimal. The playground's saved
ai-explain-buffer definition additionally limits the answer to two English
sentences; that exercise constraint is not a general product requirement.
Preserve user-authored literal saved prompts. Define how default explanatory
guidance and format requirements coexist with those definitions before coding.

## Preserved contracts

- Retain the existing JSON version/text result contract; Org formatting belongs
  inside the explanatory text and native presentation, not in a new transport.
- Keep explanation results read-only with no apply action. Keep proposals
  read-only until explicit native acceptance, with one undo step and no implicit
  save. Source, target and revision checks remain authoritative.
- Presentation must preserve generated payload bytes even when they contain
  Org block/drawer delimiters. Do not recover accepted source through heuristic
  stripping or conversion of the displayed Org wrapper.
- Reuse native Org coloring and per-view visibility. This does not promise new
  embedded-language highlighting or an independent AI folding system.
- Use a transient editor buffer; do not require creating an Org file on disk.

## Validation direction

Exercise a function, a complete source file, technical prose and clearly
nontechnical text. Check useful adaptive explanations and a consistent Org
envelope without requiring identical probabilistic wording. Verify source names,
metadata access, initial drawer folding and ordinary editing-profile controls.
For proposals, cover multiline generated source, literal Org delimiters,
explicit apply/discard, stale-source rejection and exact one-step undo. Confirm
that applying a proposal inserts no presentation markup or request metadata.

## Relationship and delivery boundary

Coordinate native visibility behavior with
[Configure Org drawer visibility](../solved/20261103111614-configure-org-drawer-visibility.md).
That ticket owns general Org startup/runtime policy; this ticket owns AI result
presentation and the default explanation instruction. Preserve the delivered
[Pi integration](../../spec/editor-ai-pi-integration.md) evidence.

The owner clarified that this information completes acceptance of the current
Pi delivery rather than defining a later backlog item. Implement it on dev
without reopening the completed controller run or relabeling its gate evidence.
General Org startup/drawer policy remains deferred. This local ticket stays
uncommitted until the pending-ticket batch; further real-provider requests
require separate explicit invocation.

## Resolution

Implemented and pushed to dev in d65e83d921ec53ffbe2d6f55ff2492def25248c6.
The shared Org presentation, adaptive explanation guidance and original-byte
application are delivered. Focused native AI/Org tests, both offline Pi/native/PTY
profiles, documentation checks and local Spore generation passed. The existing
playground now uses the matching release binary. Acceptance evidence is recorded
separately in the [slice 8 report](../../report/slices/rev-ai-pi/slice-8-acceptance.md#acceptance-presentation-adjustment).
The original completed controller run and the deferred general drawer-policy
ticket retain their existing boundaries. This ticket remains local/uncommitted
for the later pending-ticket batch.
