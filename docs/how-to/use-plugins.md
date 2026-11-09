# Configure and troubleshoot plugins

Use these procedures for a trusted local package already built and reviewed.
For a first run, follow [Try an Experimental Plugin](../tutorials/user-guide/22-experimental-plugins.md).
For executable code, use the [authoring tutorial](../tutorials/write-plugin.md).

## Choose activation

Set `plugins.enabled = true` and list explicit local manifest paths in your
startup configuration, then restart rev. Relative manifest paths follow the
configuration file that last assigned the array, including an included file.
Program paths follow their manifest directory. Arguments remain literal;
variables and PATH are not expanded. Review the complete manifest before use.

Leave `startup = []` for lazy command activation. To activate the example at
startup, change that array to `startup = ["field.hello"]`. It must name an
admitted package; there are at most eight distinct startup IDs. Manifest
filetype and opened/changed/closed triggers are another explicit package choice.
An event trigger starts a package but does not replay events or subscribe it.

Startup settings are not live toggles. `plugin-activate`, `plugin-deactivate`
and `plugin-restart` control the current admitted package for this session.
Each editor owns its own process; opening a second editor does not reuse the
first editor's process or buffer handles.

## Choose a shortcut

Add this user choice after the `[plugins]` settings, removing a literal
`keybindings = []` assignment before using array-of-table entries:

```toml
[[plugins.keybindings]]
command = "field.hello.ping"
keys = "F8"
context = "editor"
scope = "global"
```

For modal Normal use `context = "vim_normal"`. The
[complete configuration example](../reference/plugin-config-example.toml)
contains both choices. Restart to apply it. `F8` now invokes the same qualified
command as the selector. These are native contextual keys; a prompt or another
transient owner keeps control.

For a Rust-only choice, replace scope with `"buffer"` and add
`filetypes = ["rust"]`. For package-created results use
`buffer_owner = "field.hello"`. When both selectors appear they intersect.
A global entry cannot have selectors. Buffer-local choices resolve before global
ones; within a scope, explicit user choices precede package defaults. A local
package default can therefore precede a global user key. Supply a matching local
user choice to change that policy. Conflicting package defaults do not select a
winner by activation order. Inspect their status in `plugin-list`.

## Use prefixes and recording

Check the command's declared policy before supplying an integer prefix. A
`prefix = "integer"` handler receives one captured value once; the host does not
call it N times. The hello example declares `none`, so a prefix refuses it.

Only commands declaring `recording = "semantic"` can become qualified calls in
keyboard macros. Recording stores the command, literal validated arguments and
prefix. Replay checks the current declaration and waits for completion. `C-g`
stops replay while keeping committed edits. Generated RPC edits are not recorded
again. Saving a macro containing plugin calls explicitly writes library schema 2;
an older editor may not read it. The hello example deliberately rejects recording.

## Resolve failures

| Observation | Next action |
| --- | --- |
| No qualified command appears | Check enabled, manifest path, startup diagnostic and complete declaration admission; restart after configuration edits. |
| Command appears but activation fails | Inspect `plugin-list` for program/permission/handshake diagnostics. Discovery does not probe executables. Match package, version, API and handler identities. |
| A declared command is unavailable | The accepted handshake did not install that handler. Correct the package and explicitly restart. |
| Shortcut is shadowed or conflicting | Inspect its context, scope, user/default provenance and selectors in `plugin-list`. Protected recovery keys and native prefix subtrees cannot be overridden. |
| Command is pending | Continue ordinary editing, or use `C-g` to cancel and retire owned work. Macro replay retains its own input rules. |
| A buffer edit is stale | Keep current text. Have the package re-read and recalculate before proposing a new edit; do not force an old result. |
| Save or close needs confirmation | Use native save/write/close controls and their current prompts. The protocol cannot grant overwrite or discard consent. |
| Shared package state conflicts | Re-read the record and use its current version for a new explicit CAS. An uncertain committed write requires observation before another mutation. |
| Cleanup is uncertain | Keep the reported evidence; rev refuses a new generation. Supervision cannot prove escaped descendants have stopped. |

Use bounded diagnostics as text. Stdout must contain only protocol frames;
logging belongs on stderr. For exact errors, capabilities and resource limits,
see the [protocol](../reference/plugin-protocol.md) and
[behavior](../reference/plugins.md) references. Return to [How-to guides](index.md).
