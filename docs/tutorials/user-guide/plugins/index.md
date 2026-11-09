# Experimental plugins

Plugins are an optional way to run trusted local tools through named editor
commands. Everyday editing needs no plugin configuration. The current API is
experimental; keep each tool matched to the editor version it was tested with.

## Get started

1. [Try an Experimental Plugin](../22-experimental-plugins.md): admit one local package, run its qualified command and inspect or stop it.

Use a package you have reviewed, or build the small example in
[Write Your First Experimental Plugin](../../write-plugin.md). That authoring
tutorial requires Rust; ordinary editor use does not.

## Specific tasks

- [Configure and troubleshoot plugins](../../../how-to/use-plugins.md): choose startup activation, override shortcuts and recover from failures.

## Understand and look up

[Plugin ownership](../../../explanation/plugin-ownership.md) explains native
text, undo, processes and shared state. The [behavior reference](../../../reference/plugins.md)
covers configuration and editor commands; the [protocol reference](../../../reference/plugin-protocol.md)
is for package authors.

[User Guide](../index.md) · [All documentation](../../../index.md)
