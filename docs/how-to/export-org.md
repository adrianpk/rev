# Export an Org document

To create an HTML file from the current Org document:

1. Run `M-x org-export-to-html`.
2. At `Export scope`, press Enter to export the whole buffer.
3. At `Export OPTIONS`, press Enter to use the document's options.
4. At `Export to local file`, enter `notes.html` and press Enter.
5. If the destination exists, enter `yes` and press Enter to replace it,
   or `no` to leave it unchanged.

The completion message confirms the output. For a file-backed document,
`notes.html` is written beside the Org file.
For an unnamed buffer, it is relative to the current directory. Open the
result in your browser to check the rendered document.

Export uses the current buffer, including unsaved edits, but does not save or
modify the Org source. HTML export needs no external program.

For Markdown instead, use `M-x org-export-to-markdown` and a `.md` destination.
The same scope and options prompts apply.

See [Org export and publishing](../reference/org-export.md) for the other
formats, document options, subtree/region export, optional PDF tools and
publishing projects.
