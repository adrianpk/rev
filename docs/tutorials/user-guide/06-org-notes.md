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

## Inspect heading stars

With the factory settings, the first two headings appear as `◉ Trip` and
` ○ Equipment`. The markers follow heading levels; the saved file still
contains `* Trip` and `** Equipment`.

Run `M-x org-pretty-headings-mode`. The feedback reads `Org headings: source`
and the original stars appear. Run it again: `Org headings: pretty` restores
the markers. In the modal profile, `:org-pretty-headings-mode` runs the same
command.

Move through the prefix with the arrow keys. Each blank leading cell and the
marker keep the original star's position. Selecting and copying this prefix
copies its stars.

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
