# Inspect Saved Changes

See a saved edit in native Git Status, expand its diff and return to the same
source buffer. Use a new practice directory so that every change is yours.

## Prepare a local repository

Run these commands in a terminal with Git installed. Choose an unused directory:

```sh
mkdir rev-git-practice
cd rev-git-practice
git init --initial-branch=main
git config user.name 'Practice User'
git config user.email 'practice@example.invalid'
printf 'red\ngreen\nblue\n' > colors.txt
git add -- colors.txt
git -c commit.gpgSign=false commit -m 'Add colors'
rev colors.txt
```

The identity settings apply only to this practice repository. The initial
terminal command disables signing for that one practice commit; rev retains
Git's configured hooks and signing policy for its own operations.

## Compare saved text

Change `green` to `gold`. Save with `C-x C-s`, or `:save-buffer` in modal Normal.
Press `C-x g`. Status occupies the editor canvas and shows `colors.txt` in
Unstaged. Only nonempty groups appear. A clean repository has no change rows.

Use Up/Down to select the path, then Tab to expand its comparison and hunk.
You should see the removed `green` and added `gold` lines. Invoke
`M-x git-diff` or `:git-diff` for read-only patch detail. Press q to return
one level. The diff is an observation of disk and index bytes, not a second
editable copy of your file.

## Return to your source

From the selected Status path, Return visits the canonical source buffer.
`C-x g` returns to Git. From Status, q restores the surviving editor layout.
Your file, cursor and other panes remain available; hidden terminals keep
running.

Try changing `gold` to `amber` without saving. Reopen Status and press g to
refresh. Git still observes saved `gold`; the source buffer keeps unsaved
`amber`. Undo that unsaved edit to restore `gold` before continuing. Git does
not save a source buffer when you open or refresh Status.

An unavailable or truncated observation is labelled. Missing Git, a timeout
or an incomplete read never means that a repository is clean. Refresh after
correcting the cause; the [ownership explanation](../../explanation/git-ownership.md)
describes how old confirmed observations remain distinguishable.

Next: [Stage and Commit](18-git-index-and-commit.md).

[User Guide](git/index.md)
