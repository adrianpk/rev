---
id: TKT-20261114010915
title: Filter commands by multiple name fragments
status: open
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: commands, search, completion
source: chat
reported_at: 2026-11-14T01:09:15Z
commits: []
---

# Filter commands by multiple name fragments

## Required behavior

M-x must filter commands by multiple name fragments separated by whitespace.
Typing `mark ind it` must retain `markdown-indent-item`. Apply the same behavior
to the shared modal command selector.

## Acceptance

- Match the individual fragments against the command name instead of requiring
  the entire query to occur as one contiguous substring.
- Handle leading, repeated and trailing whitespace consistently. A fragment that
  does not match must exclude the candidate.
- Preserve existing full-name and single-fragment behavior, exact-name priority,
  usage ranking, completion and invocation.
- Keep filtering separate from document editing and command execution.

Current matching owner: `src/input/palette.rs`.

Capture only; no implementation is authorized in this task.
