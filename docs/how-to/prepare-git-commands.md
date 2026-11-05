# Prepare Repository Commands

Use these routes when the basic Status actions do not cover your task. They
use native editable drafts and exact reviews. Installed Git supplies repository
semantics; optional subcommands such as subtree must be available in that Git.

## Review a prepared family

1. Invoke the required `git-` command with M-x or modal colon, or its contextual sequence below.
2. Edit the seeded JSON array. Fill the literal operands while keeping the family's required options and positions. No shell expansion occurs.
3. Submit with `C-c C-c` / `C-Enter` to prepare the exact executable, directory, arguments and target review. Preparation may read local/remote facts but does not launch the changing operation.
4. Inspect the complete review. A separate `C-c C-c` / `C-Enter` accepts that captured plan. Return does not consent. `C-g` cancels composition; q / `C-g` cancels review.
5. Inspect `git-processes` and actual repository/output state after completion. Changed evidence requires new preparation; a failure never causes an automatic retry or rollback.

| Goal | Status sequence / named family | Required choice |
| --- | --- | --- |
| Clone | C C / `git-clone`; C s / `git-clone-shallow`; C b / `git-clone-bare`; C m / `git-clone-mirror` | Explicit source and absent destination with an ordinary existing parent; shallow selects depth one. |
| Initialize | I I / `git-init` | Explicit absent destination; also callable from ordinary editing or a chosen directory without an existing repository. |
| Ignore a path | i t / `git-ignore`; i p / `git-ignore-exclude` | Visit actual .gitignore or common info/exclude, edit and save through ordinary source controls; no invented pattern. |
| Manage submodules | o a/l/u/s/d / `git-submodule-add`, `git-submodule-list`, `git-submodule-update`, `git-submodule-sync`, `git-submodule-deinit` | Explicit metadata name/source/path for add, exact gitlink path for changing existing state; update checks out the indexed commit, deinit is non-forced. |
| Manage a subtree | O a/m/f/p/s / `git-subtree-add`, `git-subtree-merge`, `git-subtree-pull`, `git-subtree-push`, `git-subtree-split` | Literal prefix and exact source/commit; add/pull pin a full source branch, push reviews an explicit endpoint and full destination branch. |
| Change notes | T a/r/p / `git-notes-add`, `git-notes-remove`, `git-notes-prune` | Full refs/notes name and exact target/message or complete missing-object prune set; add does not overwrite a note. |
| Select sparse directories | > e/d/r/s/a / `git-sparse-checkout-enable`, `git-sparse-checkout-disable`, `git-sparse-checkout-reapply`, `git-sparse-checkout-set`, `git-sparse-checkout-add` | Clean saved state and literal relative directories; supported cone mode without sparse index, with private linked-worktree patterns. |

A mirror clone does not later push automatically. Sparse routes do not force
dirty materialization, accept non-cone patterns or represent newline directory
names. Submodule deinit protects dirty child state and retained native owners.
Subtree push is separately scoped to its reviewed subtree endpoint; it does
not inherit the dedicated single-upstream branch push action.

See the [exact prepared-family contracts](../reference/commands.md#prepared-clone-and-init-arguments)
for argument positions, capability checks, refusal cases and postconditions.

## Save patch or request output

Use W c / `git-format-patch`, W s / `git-save-diff` or W r / `git-request-pull`.
These drafts are JSON objects containing `arguments` and an explicit absent
`output` file. For example, in a repository with two commits:

```json
{"arguments":["diff","--binary","--full-index","--no-ext-diff","--no-textconv","--no-color","HEAD~1","HEAD","--"],"output":"review.diff"}
```

Preparation captures complete saved-commit output and shows its byte count and
blob hash. Acceptance regenerates it and requires exact byte equality before
non-overwriting local publication. An oversized result rejects as a whole.
Use ordinary `find-file` to inspect the saved output.

Format-patch selects one local BASE..END range. Request-pull needs explicit
local commit anchors and a full remote branch already naming the local end
commit; it saves text locally without publishing commits or sending mail.
Unsaved source is never included implicitly. See [output contracts](../reference/commands.md#prepared-patch-and-request-output).

## Apply a local patch or mailbox

Use w p / `git-apply-patch` or w a / `git-apply-mailbox` and fill the final
operand with one complete ordinary local input file. Review its exact path,
bytes and hash. Clean tracked state, saved visiting sources, an existing HEAD
and no active sequence are required. The accepted child consumes the captured
bytes, not a later reread of that pathname.

Plain patch application checks both index and working files using
`apply --index --binary`. Mailbox application creates the checked ordered
commit chain, preserving decoded authors, author dates and messages. A mailbox
From label alone does not prove semantic equivalence. A failed mailbox can
leave applied commits or Git-owned progress: inspect `git-operation-status`
and actual history before taking another action. The [application reference](../reference/commands.md#prepared-local-patch-application)
retains the exact format, signing and verification boundaries.

## Run an explicit arbitrary command

`!` / `git-run-command` opens a literal Git argv draft, for example:

```json
["branch","topic","HEAD"]
```

Only the separately selected `git-run-shell-command` interprets shell syntax.
Its JSON array contains exactly one complete shell string:

```json
["printf 'one\\n' | tr a-z A-Z"]
```

Inspect mode, executable, canonical cwd, argv/input and limits in the first
submission's review, then explicitly accept execution. Stdin is closed.
All file-backed buffers must be saved because arbitrary filesystem effects
cannot be inferred. The shell route uses configured `shell.program`/`shell.args`
and the smaller Git-operation/shell time and output bounds. Git's own aliases
and helpers retain their semantics even in the literal Git route.

Arbitrary effects remain unknown after exit zero. Processes retains the actual
exit/cancel/timeout/output-limit facts and bounded output; compatible Status is
refreshed after possible effects. No output becomes a source edit or another
command automatically. See [direct-command contracts](../reference/commands.md#direct-git-and-explicit-shell-commands)
and [Git limits](../reference/git.md#startup-preferences-and-resource-bounds).
