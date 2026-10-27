# Edit and save files

Build `rev` as described in the [README](../../README.md), then run it in a
terminal that supplies the [required key reports](../reference/terminal-input.md).

1. Run `./target/release/rev notes.txt`. An absent `notes.txt` opens empty without
   creating it. Run without a path for unnamed text.
2. Type directly. Return inserts a line separator; Tab inserts a tab. Move with
   the arrows or `C-f`/`C-b`, and use Backspace or `C-d` to delete whole graphemes.
   Bracketed paste inserts literal text as one undo unit.
3. Press `C-x C-s`. An unnamed buffer asks for a destination; enter one path and
   Return. Use an existing writable parent directory. To choose another path
   later, use `C-x C-w`.
4. Read the echo result. `Saved` and a clean `--` status acknowledge completion.
   A conflict asks for a fresh `y` or `n`; Return has no affirmative default.
   `C-g` cancels. On an error, follow [Recover edits](recover-edits.md).
5. Use `C-x u` to undo, or `M-x undo-redo` to redo. Saving retains history.
   Undoing to a retained saved state restores its clean indication.
6. Press `C-x C-c` to exit. For each modified buffer, `y` saves, `n` permits
   discarding that buffer for this exit, and `C-g` cancels exit. A failed required
   save retains the session. Earlier completed saves remain on disk after cancel.
7. Run `./target/release/rev notes.txt` again to read the saved file. Reopening in
   a new process starts fresh history; file bytes retain their encoding details.

To open another file while editing, use `C-x C-f`. Reopening a canonical path
reuses its current buffer. `C-x b` switches by live buffer name; `C-x C-b` opens
an inventory selected with arrows and Return. `C-x k` closes a buffer after any
required modified-text decision.

For another view of the same text, use `C-x 2` or `C-x 3`, then `C-x o` to cycle.
Each view has independent navigation. `C-x 0` closes the selected view and
`C-x 1` keeps only it; the shared text remains in its buffer.

See [Complete bindings](../reference/bindings.md),
[Commands and M-x](../reference/commands.md) and
[Scope and defaults](../reference/defaults.md).
