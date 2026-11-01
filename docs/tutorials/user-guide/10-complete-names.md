# Complete Names

Use a name already present in a Rust file, then undo its insertion. This chapter
uses the default completion settings and a small file you create yourself.

## Ask for a name

Open `completion.rs` and enter these lines:

```rust
fn configuration() {}
fn config_value() {}
con
```

Leave the cursor after `con`. After a short delay, a single-border list appears
near the cursor. Its first row is selected using the selection colors:

```text
con
┌─────────────────────────┐
│ƒ configuration  function│
│ƒ config_value   function│
└─────────────────────────┘
```

The exact width and order follow the available space and nearby declarations.
The indicator and type label use syntax colors; only the name is inserted.
The list can appear above the cursor when there is little room below it.
The source text stays unchanged while you choose.

## Choose and undo

Press Down or `C-n` to select another row; Up or `C-p` selects the previous row.
Select `configuration`, then press Tab or Return. The final line now contains
`configuration`. Press `C-/` to undo: the line returns to `con` in one step.

Run `M-x completion-at-point` to request the list immediately. `M-TAB` invokes
the same command when your terminal and desktop deliver that chord. Press Escape
or `C-g` to cancel this request. Cancellation changes no text and stays quiet
until you edit an eligible name again or request completion explicitly.

These suggestions come from syntax and observed names, including unsaved text
in admitted same-language open buffers. They do not infer a receiver's type or
resolve imports. You can keep editing and saving normally without accepting a
suggestion.

See [Use completion](../../how-to/use-completion.md) to change automatic requests
or timing, and [Native completion](../../reference/completion.md) for language
contexts and work limits. Org has additional structural contexts described in
that reference; the separate [Org guide](org/index.md) remains its learning path.

## Add optional semantic suggestions

The steps above work with LSP disabled, using names already observed in your
buffers. With an installed `rust-analyzer`, run `M-x lsp-mode` in a Rust project
to request semantic suggestions in the same popup. They follow unsaved text and
can provide checked same-document imports with one undo. Empty or unavailable
server results keep native fallback available. `M-x lsp-disconnect` excludes
this buffer; `M-x lsp-connect` restores it. See
[Use a language server](../../how-to/use-language-server.md) for setup and recovery.

You have completed the User Guide. Continue with a
[task guide](../../how-to/index.md) or look up a
[command](../../reference/commands.md).

[Previous](09-preferences.md) · [User Guide](index.md)
