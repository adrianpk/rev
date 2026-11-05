# Write Org Notes

Org is a plain-text format for notes, outlines and tasks. Here you will create
a small outline, inspect its heading presentation and fold part of it without
changing the document.

Open `trip.org` with `C-x C-f`. Type or paste:

```org
* Trip
Pack light and leave early.
** Equipment
Bring a notebook and a pencil.
** Route
Take the path along the river.
* Notes
Check the forecast before leaving.
```

Save with `C-x C-s`. The status line shows `Org`; the filename enables the
native mode automatically.

## Align heading bodies

With factory settings, the start of your outline appears as:

```text
◉ Trip
  Pack light and leave early.
  ○ Equipment
    Bring a notebook and a pencil.
```

The body follows its heading. Run `M-x org-indent-disable` to see the same
source without the added visual spacing:

```text
◉ Trip
Pack light and leave early.
○ Equipment
Bring a notebook and a pencil.
```

The feedback reads `Org indentation disabled`. Run `M-x org-indent-enable` to
restore the first presentation; the feedback reads `Org indentation enabled`.
`org-indent-mode` toggles between them. Repeating enable or disable keeps that
state. In the modal profile, use `:org-indent-mode`, `:org-indent-enable` or
`:org-indent-disable`. There is no default shortcut.

With indentation disabled, heading markers all start at the left margin. If you
switch to source presentation, the original `*`, `**` and `***` prefixes remain
visible; their characters take space, but no leading spaces are added.

This choice applies to every pane showing this buffer, including read-only Org
results. Other buffers keep their own choices. It survives edits, undo, folds,
resize and reload. Text mode suspends it; returning to Org restores it. Closing
the buffer discards the choice, so reopening uses your startup preference:

```toml
[org]
indent = true
```

Set `indent = false` for the second presentation at startup. Commands do not
write this configuration. Copying and saving still use the original whitespace;
wrapped body lines keep the same outer margin. Pretty headings and
`visual-line-mode` remain independent choices. Narrow panes cap the added
spacing so source text remains reachable. See [Org configuration](../../reference/org/configuration.md)
for the exact margin rules and [indentation commands](../../reference/org/commands.md#visual-indentation)
for their scope.

## Inspect heading stars

With the factory settings, the first two headings appear as `◉ Trip` and
`  ○ Equipment`. The markers follow heading levels; the saved file still
contains `* Trip` and `** Equipment`.

Run `M-x org-pretty-headings-mode`. The feedback reads `Org headings: source`
and the original stars appear. Run it again: `Org headings: pretty` restores
the markers. In the modal profile, `:org-pretty-headings-mode` runs the same
command.

Move through the original star prefix with the arrow keys. Its concealed
cells and marker keep each source star addressable after the visual margin.
Selecting and copying this prefix copies its stars; the added margin is not text.

Run `M-x org-reset-heading-presentation` to return this pane to your startup
choice. With factory settings, the feedback is `Org headings: pretty`.
Each pane has its own choice, so switching presentation does not change the
file or another pane.

The terminal chooses fonts and ligatures. If a marker is unavailable, switch
to source presentation or choose simpler one-cell symbols in the
[heading preferences](../../reference/defaults.md#org-heading-presentation).

## Fold an outline

Move to the `Trip` heading and press Tab. Its body and child headings fold
away. Press Tab again to show its child headings, then again to show the
complete subtree.

After the first Tab, that part of the buffer shows the following outline.
These code examples use source stars; pretty presentation shows the same
headings with markers.

```org
* Trip
* Notes
Check the forecast before leaving.
```

The `Trip` heading remains visible, while its text and children are hidden.

Tab acts on a heading or drawer, so place the cursor on the heading line for
this exercise. To cycle visibility across the whole document, use Shift-Tab.
`M-x org-show-all` restores all source in the current pane.

Folding changes the view, not the saved file. Search can still find folded
text and reveal its containing entry.

## Move between headings

On `* Trip`, press `C-c C-n` to visit the next visible heading. On a child
heading, `C-c C-u` goes to its parent. These controls avoid stepping through
every line of a long section.

You can keep an Org file as ordinary notes or use the task features next.
See [Org support in rev](../../explanation/org-support.md) for its scope and
the [Org reference](../../reference/org-mode.md) for all outline controls.

Next: [Plan Tasks](07-org-tasks-and-agenda.md).

[Org mode guide](org/index.md)
