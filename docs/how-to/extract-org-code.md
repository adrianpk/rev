# Extract code from an Org document

Tangling writes source blocks from an Org document into ordinary code files.
Use it when you keep an explanation and its source together but need a separate
file for a compiler or another tool.

Open `example.org` and add:

```org
* Greeting
This block supplies the program in hello.rs.

#+BEGIN_SRC rust :tangle hello.rs
fn main() {
    println!("Hello");
}
#+END_SRC
```

Save the document, then run `M-x org-babel-tangle`. The command creates
`hello.rs` beside `example.org`. No output directory needs to be prepared.
If the file already exists, rev asks `yes/no` before replacing it.

Open `hello.rs` with `C-x C-f` and check that it contains the Rust code without
the Org heading or block delimiters. Tangling writes the file; it does not
compile or execute the program.

Return to `example.org` with `C-x b`. To edit the block in a separate buffer,
put the cursor inside it and press `C-c '` — Control-c, then the apostrophe key.
`C-c C-c` returns the edited body to the Org document; `C-c C-k` discards that draft.
Save and tangle again when you want to update `hello.rs`.

See [Org source blocks](../reference/org-source-blocks.md) for multiple blocks,
shared fragments, output paths and separately configured execution.
