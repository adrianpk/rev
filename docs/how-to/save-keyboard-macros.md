# Save a Keyboard Macro

Record and test the macro first. Naming and binding keep independent snapshots
for the current editor session; saving is a separate explicit operation.

## Name and optionally bind the current recording

1. Press C-x C-k n, enter a name such as `mark-task`, then Enter.
2. Run M-x mark-task to check the snapshot at another suitable source position.
3. Optionally press C-x C-k b, then a digit such as 7. This selects C-x C-k 7.

Names use 1..64 lowercase ASCII letters, digits or hyphens and start with a
letter. Built-in, prompt and admitted AI command names stay reserved. Replacing
a session name or occupied source binding requires confirmation. A bound named
macro follows that name; an unnamed binding owns its own independent snapshot.
Later ring append, eviction or deletion does not rewrite the named snapshot.

## Save only the selected definition

Run M-x kmacro-save, enter `mark-task`, then confirm the save. To include its
session binding, supply a prefix before opening the command: C-u M-x kmacro-save.
The raw presence of the prefix controls inclusion, not its numeric value.

Without a prefix, replacing an earlier saved record removes its saved binding;
the confirmation announces that consequence. The current session binding remains.
Source-file saving, naming, binding and editor exit never save macros implicitly.

Restart the editor. The saved command is available through M-x, or modal colon,
without running automatically. A binding included in the save is admitted only
when it does not collide with a protected or configured owner. A missing binding
does not remove the saved command.

## Edit, reload or remove a saved definition

C-x C-k e selects the named or bound snapshot for an editable TOML draft.
C-c C-c validates and applies it to that session target. Invalid or stale drafts
stay editable; normal buffer closing discards one. Applying never saves. Run
kmacro-save again when you intend to replace the saved record.

M-x kmacro-load rereads the fixed library without execution. It preserves unsaved
local bodies, descriptions, counters and bindings and reports conflicts. It is
not an implicit discard command. If another editor changed the same saved record,
a stale save fails; retain the local draft and reconcile the independent values
before explicitly saving again. Changes to unrelated records remain intact.

M-x kmacro-delete selects and confirms removal of one saved record. Its current
session name and independent ring copies remain available. Deleting a row in the
macro menu only removes that ring entry and leaves saved records untouched.

## Back up or recover the library

Back up `macros/library.toml` beneath the application state root. Its usual location
is `~/.local/state/rev/macros/library.toml`; XDG_STATE_HOME can select another root.
This is valuable saved content. Session-only recordings are memory-only and cache
cleanup does not remove the saved library.

An invalid or unsupported library is left untouched while ordinary editing stays
available. Keep its original bytes for inspection; do not replace it with an empty
file to silence a diagnostic. See the [state format and publication rules](../reference/macro-library.md)
for exact fields, bounds, permissions and conflict behavior.
