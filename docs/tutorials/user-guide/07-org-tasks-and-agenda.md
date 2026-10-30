# Plan Tasks

Open a new `tasks.org` file with `C-x C-f`. Add:

```org
* TODO Pack the bag
* TODO Check the route
```

`TODO` is a task state. An ordinary heading without a recognized state is
still a heading, but it is not an unfinished task.

## Finish a task

Place the cursor on `Pack the bag` and press `C-c C-t`. With the default
workflow, `TODO` changes to `DONE`. Save with `C-x C-s`.

The heading now reads:

```org
* DONE Pack the bag
```

## Schedule the remaining task

Move to the `Check the route` heading and press `C-c C-s`. Press Enter to
accept today's date. A `SCHEDULED:` line appears below the heading.

Save again. Scheduling changes the Org source, so you can read the date in
the file itself.

For example, with October 30, 2026 as today, the scheduled entry reads:

```org
* TODO Check the route
SCHEDULED: <2026-10-30 Fri>
```

## Find it in the agenda

Press `C-c a`. With the default settings, rev asks for an agenda source.
Type `current`:

```text
Agenda source [current/buffers/path]: current
```

Press Enter to use this file.

The agenda shows the scheduled task. Its row includes the state, title and
source filename:

```text
Scheduled: TODO Check the route (tasks.org)
```

`g` refreshes the agenda and `q` closes it.
Move to an entry with Up/Down and press Enter to visit its source heading.

Within an agenda, `s` opens a text search of its source entries. This differs
from `C-s` in an editing buffer: the agenda does not provide incremental
search over its displayed rows.

If you already configured agenda sources, rev uses those instead of asking
for `current`. The [agenda reference](../../reference/org-agenda.md) describes
source selection, TODO views and filters.

For a quick task entry without leaving your current document, see
[Capture a task](../../how-to/capture-task.md). Checklists, tags, priorities,
deadlines and clocks are described in the [Org reference](../../reference/org-mode.md).

Next: [Calculate a Table](08-org-tables.md).

[Previous](06-org-notes.md) · [Org mode guide](org/index.md)
