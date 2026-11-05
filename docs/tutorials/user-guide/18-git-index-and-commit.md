# Stage and Commit

Continue with the practice repository from the previous chapter. The saved
`colors.txt` contains `gold`, and Status shows one unstaged change.

## Choose the index content

Select the `colors.txt` path in Status and press s. After the checked operation
finishes, its change appears under Staged. Expand it with Tab to inspect the
index comparison. Press u to unstage it; the saved file still contains `gold`.
Select it again and press s to stage it for this exercise.

A file can appear under both Staged and Unstaged when its saved bytes change
after staging. The two rows describe different comparisons. To stage only an
admitted text hunk, expand an unstaged path, select the hunk and press s.
Unsupported or incomplete comparisons remain inspectable and cannot grant a
hunk edit. [Review and commit changes](../../how-to/review-and-commit.md)
covers compatible marked groups and whole-path discard.

## Compose a message

Press c in Status. The native composer opens immediately. Type:

```text
Use gold in the color list
```

The composer accepts ordinary text; c and q are literal letters here. Plain
Return inserts a newline. Press `C-g` to cancel before execution. Your staged
change remains, and c in Status reopens the session draft.

Submit with `C-c C-c`, or `C-Enter` when your terminal reports that event.
The commit uses the checked index and edited message. It does not stage more
files or save source buffers. A Git hook or signing failure leaves a usable
draft and an actual failure in Processes; inspect it before retrying.

## Verify the result

After success, Status has no changed-path rows in this practice repository.
Press h to open History. The newest commit should have your message. Tab
opens its revision detail, including the full identity and changed path.
Press q to return to History, then q to Status and q to editing.

If a result is stale or unverified, use `$` / `git-processes` to inspect the
actual exit and checked outcome. Refresh and inspect the index and history
before deciding what to do next. Exit zero alone cannot prove every effect.

Next: [Explore Repository History](19-git-history-and-remotes.md).

[Previous](17-git-status.md) · [User Guide](git/index.md)
