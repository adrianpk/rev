# Write Org Notes

Org is a plain-text format for notes, outlines and tasks. Here you will create
a small outline and fold part of it without changing the document.

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

## Fold an outline

Move to the `* Trip` heading and press Tab. Its body and child headings fold
away. Press Tab again to show its child headings, then again to show the
complete subtree.

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
