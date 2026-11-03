# Follow External File Changes

Watch a clean file refresh after another program replaces it, then reload a
file with unsaved edits by explicit choice. This chapter describes the
Unreleased development version. Use `M-x` in non-modal editing or `:` in modal
Normal for the named commands below.

## Open a practice file

In a terminal, create a file and open it:

```sh
mkdir -p reload-practice
printf 'first version\n' > reload-practice/note.txt
rev reload-practice/note.txt
```

Keep rev open. From another terminal, replace the file atomically:

```sh
printf 'second version\n' > reload-practice/replacement
mv reload-practice/replacement reload-practice/note.txt
```

The buffer now shows `second version` without a keystroke. Its clean state is
retained. Observation covers visiting files, including hidden buffers and shared
panes; you do not need an AI backend or language server. The notification does
not accept or cancel another prompt you are answering.

## Keep or discard an unsaved edit

Add a word in rev without saving. In modal editing, enter Insert first and
return to Normal afterward. Replace the disk file again from the other terminal:

```sh
printf 'third version\n' > reload-practice/replacement
mv reload-practice/replacement reload-practice/note.txt
```

Your unsaved text stays visible. The editor reports the external change and
keeps the save overwrite guard. To take the new disk text, run
`M-x revert-buffer`. Confirm the discard only when you intend to replace those
unsaved edits. Cancel with `C-g` or Escape to keep them.

After a successful reload, undo once (`C-/` in non-modal editing, `u` in modal
Normal). Your prior buffer text returns and becomes unsaved again; the disk
still contains `third version`. Reloading and undoing do not save the file.

## Choose notification instead of automatic reload

To keep even clean buffers unchanged until you choose, set this in your existing
configuration and restart rev:

```toml
[files]
external_changes = "ask"
```

Merge this field into an existing `[files]` table if one is present. An external
change now produces a passive notice and `disk: revert-buffer` status. Run
`revert-buffer` to read the current disk image. The default is `"reload"`.
Neither policy discards unsaved text automatically.

If the source is missing, invalid UTF-8 or unstable while read, rev retains the
buffer and reports the failure. Restore the file, then retry `revert-buffer`.
See [Recover edits](../../how-to/recover-edits.md) and the
[reload reference](../../reference/commands.md#reload-a-visiting-file) for exact
consent, race and cleanup behavior.

Next: [Edit with Pi](14-ai-editing.md) to inspect a proposal before applying it to native text.

[Previous](12-language-servers.md) · [User Guide](index.md)
