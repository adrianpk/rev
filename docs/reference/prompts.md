# Focused prompts

The bottom prompt owns keyboard input until its current stage accepts, reports
an error or is canceled. Command names, paths, buffer names and decisions use
separate focused contexts. Their text does not enter the document underneath.

## Stages

| Operation | Sequence |
| --- | --- |
| M-x | Command query, then the selected command's handler and any arguments. |
| Find file | Path, then open/reuse the file or report an ingress error. |
| Write file | Destination path, then an explicit decision if the destination is occupied or a different referent requires consent, then save. |
| Save buffer | Save the visited path, or request a destination if unnamed; external changes require a separate explicit decision. |
| Switch buffer | Live buffer name; an empty argument selects the current buffer. |
| Kill buffer | Live buffer name; clean text closes immediately. Modified text asks whether to save, discard or cancel. |
| Save before closing | Named text follows its save checks; unnamed text requests a destination. A destination conflict requires its own decision. Close occurs only after a completed save of the captured target. |

Buffer-name Tab completion uses live names and their common prefix; Return
selects an exact name. Tab in a path argument keeps the draft and displays
`Enter a path; C-g cancels`. [M-x completion and history](commands.md) operate
only in command focus.

## Editing and decisions

Argument text supports Left/Right or C-b/C-f, Home/End or C-a/C-e, Backspace
or C-h, and Delete or C-d. Movement and deletion respect whole graphemes.
Single-line Unicode paste inserts at the prompt cursor. The existing limit is
4096 Unicode scalars; control-containing and oversized payloads reject wholly.
Successful edits clear outdated input/completion feedback without changing focus.

Save decisions accept only a fresh y/n press. Invalid keys retain the question;
Return has no affirmative default, and paste cannot answer. At the modified-buffer
close question, y means save and then close, n means discard and close, and C-g
cancels. At a save conflict question, n cancels the save and any pending close.
Repeated Return or a held decision suffix cannot accept the next stage.

## Completion, failure and cancellation

Return consumes each argument once before its handler continues. A failed open,
unknown buffer or failed save returns to editing with its error and retains the
source text. A failed required save keeps the buffer open. A later independent
save does not inherit an earlier canceled close request.

C-g drops the current pending interaction, including nested path/decision stages.
It preserves the selected buffer, point and current navigation state; normal C-g
region deactivation still applies. Canceling M-x does not reset the intended
vertical column or recenter cycle. A name that is accepted runs its actual command
with the same navigation behavior as its binding.

Resize remains actionable during every stage, including tiny or zero geometry.
The prompt and pending operation survive; presentation follows the current
terminal size. A restored editing context accepts fresh input once, without
replaying canceled text or carrying save/close authority into another command.
