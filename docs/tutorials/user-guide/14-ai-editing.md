# Edit with Pi

Ask Pi for a small change, inspect the proposal, apply it explicitly and undo
it. This chapter describes the Unreleased development version. Ordinary editing
and file reload work without Pi.

## Prepare Pi and a toy file

Use an already installed Pi 1.3.0 executable with working authentication and a
model selected in Pi. Complete Pi's own setup outside rev. rev uses Pi's existing
provider authentication; it does not ask you to paste credentials into editor
configuration. A model request may transmit the supplied source and permitted
directory context to that provider.

Create an empty practice directory, add a small source file and open it:

```sh
mkdir -p ai-practice
printf 'fn add(a: i32, b: i32) -> i32 { a + b }\n' > ai-practice/toy.rs
rev ai-practice/toy.rs
```

If Pi is not available as `pi` on PATH, set its installed executable in your
existing configuration, then restart rev:

```toml
[ai.pi]
command = "/absolute/path/to/pi"
```

Merge this into an existing `[ai.pi]` table. `command` is one executable, without
shell arguments. You can set `model = "provider/model"`; omitting it keeps Pi's
resolved default. See the [configuration catalog](../../reference/defaults.md#optional-pi-transport)
for deadlines, context/result limits and private history policy.

Use `M-x` for each named command in non-modal editing. In modal Normal, use `:`
instead. Both selectors expose the same AI commands without a factory key prefix.

## Inspect and accept a proposal

Move to the end of the practice file. Run `M-x ai-implement` and enter:

```text
Insert a short comment explaining the add function. Return only that comment.
```

Continue navigating or editing while Pi runs. A completed `*AI Result*` buffer
shows the captured source, context, model and proposed bytes. If you moved to
another view, use `switch-to-buffer` to inspect it. Generation does not save or
change the source.

With that result selected, run `M-x ai-apply-result`. The comment is inserted at
the original captured point, even if focus moved. Undo once (`C-/` or modal
Normal `u`) to remove the whole generated insertion. The file on disk stays
unchanged until you save explicitly. Model output varies; inspect its contents
before acceptance.

To decline a proposal, use `ai-discard-result`. To cancel a running operation,
use `ai-cancel`; `C-g` and the owning modal Escape also cancel through the active
input context. A separate command prompt owns its own cancellation.

For replacement, select a nonempty linear range and run `ai-prompt`. Without a
selection, `ai-prompt` inserts at point; `ai-implement` always inserts. To read
an explanation without editing authority, select text and run `ai-explain`.
The resulting `*AI Explanation*` has no apply action.

## Keep context and source current

Built-in requests can ask Pi to read, list and search the chosen directory.
The root is the open tree root when available, otherwise the source's parent
(or the launch directory for an unassociated buffer). Use `ai-set-directory` to
choose an explicit root for this editor session. Current captured editor snapshots take
precedence over disk, including unsaved visiting text. Pi chooses relevant
reads inside that grant. Shell, edit and write tools, automatic project
instructions and third-party extensions are disabled for these requests.
Private paths, escaping links, binary data and oversized sources are rejected.
This is bounded source discovery, not unrestricted access to a project.

A proposal belongs to its captured source revision. If you type, undo, reload
from disk or change that file's association before acceptance, applying the old
proposal fails as stale. Discard it and request a fresh proposal. A clean
[external reload](13-file-reload.md) therefore cannot silently receive an older
AI edit. A history entry is also historical context, not a current source index.

## Reuse an instruction

After a successful operation, run `ai-save-command` and enter `add-comment`.
The literal instruction and policy become callable as `ai-add-comment`. Each
call captures a fresh target; saving does not retain the old selection or source
text. Existing comments and unrelated configuration are preserved. Resolve a
reported configuration conflict before retrying the save.

You can also add a definition directly to configuration and restart:

```toml
[ai.commands.explain-buffer]
prompt = "Explain the supplied buffer in two short sentences."
input = "buffer"
context = "input"
output = "explanation"
```

Run `ai-explain-buffer` to use it. This definition sends only the captured input,
with fresh ephemeral history and no directory tools. Other supported policies
and keybinding examples are in [Reusable AI commands](../../reference/commands.md#reusable-ai-commands).

## Manage models and history

`ai-select-model` lists identifiers reported by Pi and changes the choice for
this editor session. Directory work retains isolated private history by default;
input-only definitions always start fresh. `ai-disconnect` stops the owned child
and preserves completed results/history. `ai-reset-session` makes the next
explicit directory request start fresh history for that binding. Idle expiry
and quitting rev also stop the owned child.

When Pi is missing, authentication fails or the selected model is unavailable,
rev reports the failure and keeps your source editable. Fix Pi's own setup or
the executable setting, discard a pending result if necessary, then retry
explicitly. Corrupt or unsafe private history is preserved and reported; consult
[Pi sessions and models](../../reference/commands.md#private-pi-sessions-and-controls).

You have completed the User Guide. Continue with a
[task guide](../../how-to/index.md) or the
[AI reference](../../reference/commands.md#selection-explanations-through-pi).

[Previous](13-file-reload.md) · [User Guide](index.md)
