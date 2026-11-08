# Keyboard Macro Ownership

A keyboard macro repeats an editing interaction through the same native owners
that handle ordinary input. It stores declarative data. It is not a program or
a promise that arbitrary asynchronous work can be replayed synchronously.

## Keys and accepted command names

Ordinary key steps use the effective keymap when replayed. If a binding changes,
the key follows that new meaning. Accepted M-x and modal colon invocations retain
the canonical command name instead: changing ranking or selector presentation
cannot select a different command during playback.

The accepted command also owns its typed prefix and role-qualified prompt values.
A file path cannot be reassigned to an unrelated prompt after a rebinding. Manual
completion retains a checked literal edit and expected source text, rather than
a popup row or a dependency on the original provider. A missing command or failed
precondition stops before the affected transformation.

Native consent still belongs to the person using the editor. Save, overwrite and
replacement decisions pause for fresh answers. Queries and manual pauses likewise
return input to their native owners. Waiting introduces no macro polling timer.
Raw terminals, context replacement and external AI, shell or Git operations stop
macro capture/playback before launch because their effects have other owners.

## Independent snapshots and explicit saving

The ring is a bounded session convenience. Naming snapshots the selected value;
changing or evicting that ring entry cannot change the named body. An anonymous
binding also owns an independent snapshot. Calls pin admitted revisions while
running, so another definition update cannot redirect a retained call frame.

Saved definitions have another lifetime. Only an explicit save writes the private
TOML library. Replaying a saved name changes source and working counters in the
current session, without rewriting disk. Reload preserves unsaved local values.
Fresh record-revision checks prevent one editor from silently overwriting another
editor's changed record while allowing independent concurrent records to survive.

Definition drafts pin their session target. Invalid or stale acceptance leaves
the old definition and editable text available. The macro menu owns identities,
marks and flags, with no authority to edit source data. Step editing owns a pending
candidate while its actual executed edits still belong to normal source history.

## History, cancellation and finite work

Each outer replay iteration groups its edits once per affected buffer. Nested
named calls share that group and the outer budgets. A multi-buffer macro is not
a transaction across buffers. Cancellation retains completed and partial source
edits; undo acts on real history rather than pretending to reverse all work.

Finite bounds cover definitions, retained snapshots, source-line targets, calls,
generated text and active automatic work. Human pauses suspend the active clock.
The host yields cooperatively between bounded batches and continues reading C-g;
a single native atomic edit is not a hard realtime operation. Once automatic work
finishes or stops, it retains no macro idle deadline or background worker.

For a first recording, use the [User Guide lesson](../tutorials/user-guide/21-keyboard-macros.md).
For exact controls and limits, use the [macro reference](../reference/keyboard-macros.md).
