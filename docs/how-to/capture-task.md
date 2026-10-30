# Capture a task

With the default capture settings, you can add a task to an inbox while
editing another file.

1. Press `C-c c` to start capture.
2. Type `t`, then press Enter to select the built-in Task template.
3. At the destination prompt, enter `inbox.org` and press Enter.
4. In the capture draft, type the task title where the cursor is placed.
5. Press `C-c C-c` to finish. rev inserts the task and saves the destination.

The entered relative path is beside the file you were editing, or in the
current directory for an unnamed buffer. On successful completion, `inbox.org`
contains the task under `Tasks`; missing target headings are created as needed.
The file is created by finishing capture, not by selecting its name.

The template also adds an inactive timestamp and, for a file-backed origin,
a link back to where capture started. That link identifies the origin, not
the inbox destination.

To discard the draft, press `C-c C-k`. Selecting `t` is a separate prompt:
press Enter before writing the task text.

If you have configured capture templates or a default destination, those
replace the corresponding defaults above. Configured relative destinations
resolve against the configuration file that declares them. See the
[capture reference](../reference/org-capture.md) for persistent destinations,
custom templates and conflict handling.
