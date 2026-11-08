# Apply a Keyboard Macro

Use a short recording whose movement leaves the cursor ready for the next
iteration. Start with the [recording lesson](../tutorials/user-guide/21-keyboard-macros.md)
if you have not recorded one yet. Work on a copy of important text until you
have checked the first result.

## Repeat a known number of times

With a macro recorded, C-u 3 F4 performs three additional iterations. C-x ) with
a positive count includes the recording pass: C-u 4 C-x ) ends capture and adds
three executions. F4 alone ends recording without replay; C-x e ends and replays.
After C-x e, bare e repeats again. Another key leaves that temporary repeat map
and is processed normally. C-g cancels the pending count and active run.

Zero or a negative replay count continues until a native failure, cancellation
or a finite run bound. It does not reverse the recording. Prefer a known positive
count while learning; movement that simply stays at a boundary may still succeed.

## Apply once to each selected logical line

Record a macro that edits the current line without moving to another one. For
example, start at a line beginning, F3, C-a, type `- `, then F4. Undo that practice
edit before selecting your real target lines.

1. Move to the first target line beginning and press C-SPC to set the region.
2. Move to the beginning of the line after the last target.
3. Run M-x apply-macro-to-region-lines, or C-x C-k r.
4. Inspect the result, then save the source file normally.

Only original logical line beginnings inside the half-open region are targets.
Wrapped screen rows add none. Inserted lines do not become new targets; deleted
original lines are skipped. Each target has an undo unit. C-g keeps completed and
partial edits, and ordinary undo removes the last actual iteration.

## Keep several recordings or append input

C-x C-k C-n/C-p selects an older/newer ring entry; C-x C-k C-t swaps the first
two. Run list-keyboard-macros for a read-only list and ? for its local controls.
Ring removal leaves independent named and saved copies available.

C-u F3 executes the current definition once, then records appended input. A
double C-u F3 appends without that preliminary execution. Finish with F4. A failed
preliminary run prevents append capture; canceled capture preserves the old body.
The two [macro preferences](../reference/defaults.md#configuration-value-catalog)
control ring capacity and whether a prefixed append pre-executes.

## Insert a counter or ask before proceeding

While recording, F3 inserts the current integer counter and increments it. It
starts at zero. C-x C-k C-c sets a value, C-x C-k C-a adds to it and C-x C-k C-f
sets a bounded integer format such as `%03d`. Counter changes are session state;
source undo reverses the inserted text. See the [exact counter rules](../reference/commands.md#keyboard-recording-and-replay)
for prefix meanings and format limits.

C-x q adds a query without interrupting recording. Playback asks each time:
Space/y continues, Backspace/n skips this iteration, Enter/q ends the run and
C-r enters manual editing. C-M-c returns to the query. With a supplied prefix,
C-x q enters an unconditional manual pause during capture and replay; edits in
that pause are excluded from recording. C-M-c resumes; C-] or C-g aborts.

## Record in the modal profile

Shared controls also work in modal editing. Begin in Normal; F3, i, type the
literal edit, Escape, then F4 records a body that returns to Normal. Start its
next replay in Normal too. Counts and operators inside the body retain their
native meanings. A recording entered from Insert or Replace requires that same
state on replay; a changed end state prevents another iteration.

To inspect or change a body, use the [definition and step editor](../reference/macro-editing.md).
To keep a reusable snapshot, follow [Save a Keyboard Macro](save-keyboard-macros.md).
