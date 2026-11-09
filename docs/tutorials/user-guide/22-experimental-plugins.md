# Try an Experimental Plugin

Run one trusted local command and inspect its process without changing your
editing profile. This optional lesson uses the `field.hello` example from
[the authoring tutorial](../write-plugin.md). Build that example first, or use a
reviewed package with its own declared command and substitute its package ID.
There is no automatic plugin discovery, download or installation.

## Admit one package

Add this table to your [startup configuration](../../reference/defaults.md#startup-configuration),
replacing the manifest path with the actual absolute path. Merge with an existing
`[plugins]` table rather than duplicating it:

```toml
[plugins]
enabled = true
manifests = ["/absolute/path/to/plugin-example.toml"]
startup = []
keybindings = []
```

Save the configuration and restart rev with a small practice text file. These
settings apply at startup. Invalid declarations report a diagnostic and retain
the complete built-in configuration; they do not admit a partial package set.
When plugins are disabled, rev does not open their manifests.

## Find and run the command

Run `M-x plugin-list`. In modal Normal, use `:plugin-list` instead. The read-only
inventory shows `field.hello`, its declared `field.hello.ping` command and process
state. Opening this inventory does not start the example program. Return to your
practice file through ordinary buffer navigation, `C-x b`, or the modal
`switch-to-buffer` command.

Run `M-x field.hello.ping` (modal Normal: `:field.hello.ping`) and accept with
Return. Command use starts one owned process lazily. A completion notice names
the qualified command and its request identity. This example changes no text;
typing, moving and saving in your practice file still use native editor controls.
Run `plugin-list` again to inspect its active generation and diagnostics.

Other packages can create ordinary unsaved result buffers. Use the normal
buffer/pane controls to visit them. Generated text is not saved automatically;
it remains available after deactivation. Undo belongs to each affected buffer,
even when a package changed several buffers together.

## Stop or retry the package

Run `plugin-deactivate` and choose `field.hello`. This stops its owned generation
and revokes pending work, subscriptions and dynamic shortcuts. It keeps native
text and package state. `plugin-activate` starts it explicitly; `plugin-restart`
retires the current generation before starting another.

If a command is pending, `C-g` cancels plugin work and begins owned cleanup.
Cancellation does not undo edits already committed. A failure does not discard
your text or retry indefinitely. Inspect `plugin-list`, correct the cause and
use an explicit activation, command or restart to try again. Packages share your
account's permissions; process supervision is not a security sandbox.

To stop admission on the next launch, set `enabled = false` and restart.
For startup activation and shortcuts, use [Configure and troubleshoot plugins](../../how-to/use-plugins.md).
The [plugin reference](../../reference/plugins.md) lists exact commands and
limits. Return to [Experimental plugins](plugins/index.md) or the [User Guide](index.md).
