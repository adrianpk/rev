# Edit and save files

With [rev installed](install-rev.md), open a file:

```sh
rev notes.txt
```

Type your changes and press `C-x C-s` to save. `Saved` and the clean `--`
marker confirm completion. If the file did not exist, saving creates it;
its parent directory must already exist.

To open another file during the session, press `C-x C-f`, enter its filename
and press Enter. Relative names start beside the current file, or in the
launch directory for an unnamed buffer. The original buffer stays open.

Run `M-x generate-new-buffer` for another unnamed buffer. `M-x open-directory`
chooses an existing directory for a fresh temporary editing context; accepting a
directory through `C-x C-f` follows the same route. Save/discard decisions apply
to modified buffers before replacement, and `C-g` retains the current work.
Ordinary file visits keep the context and explicitly chosen directory root,
including when you visit a file outside that root.

## Save under another name

Press `C-x C-w`, enter `draft.txt` and press Enter. Use an absolute destination
if it should be outside the current directory. After a successful write, the
buffer visits the new path; an existing original file stays on disk.

If the destination is occupied or has changed externally, review the prompt
and answer `y` to replace it. `n` or `C-g` cancels that save.

## Close the session

Use `M-x context-close` to resolve all current buffers and owned jobs without
leaving rev. It returns to an empty standalone buffer with the tree hidden;
command histories, bookmarks and the kill ring remain available. The same
commands are available by name through modal `:`.

Press `C-x C-c`. For each modified buffer, `y` saves, `n` quits without saving
that buffer and `C-g` cancels quitting. If a required save fails, the editor
stays open with your text intact.

Use [Recover edits](recover-edits.md) when a file operation fails. For a guided
exercise, follow [Edit Your First File](../tutorials/user-guide/01-first-file.md).
The [file and prompt reference](../reference/prompts.md) gives the exact path,
completion and confirmation rules.
