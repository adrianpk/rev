# Changelog

All notable changes to this project are documented here.

This project uses Semantic Versioning.

## Unreleased

### Added

- Open UTF-8 files at startup or with `C-x C-f`, preserving BOM, line endings and final-newline absence. Reopening a canonical file reuses its current buffer; absent paths open empty without creating files. Invalid UTF-8 and NUL-containing binary input produce an error without installing damaged text.
- Save buffers with `C-x C-s`, supplying a destination for unnamed text, or use `C-x C-w` to write another file. Occupied destinations and external changes require explicit confirmation; symlink saves preserve the link and write its resolved target. Saves preserve text format and existing access permissions. Failed saves retain edits and distinguish rejected writes from files already replaced with completion still unresolved.
- Switch between live buffers with `C-x b` and Tab completion, or select one from `C-x C-b`. Close a buffer with `C-x k`, choosing save, discard or cancel for modified text; failed required saves keep it open. Closing the final buffer leaves a fresh empty editing buffer.
- Edit Unicode text directly in the terminal with Enter and Tab. Backspace and `C-d` remove complete graphemes, including CRLF line breaks.
- Move by graphemes, Unicode words, logical line ends and buffer boundaries with Emacs bindings, keeping point visible as the viewport scrolls.
- Keep the intended display column when moving through short lines, move by pages, and cycle the current line through the center, top and bottom of the viewport with `C-l`.
- Undo edits with `C-x u` or a reported `C-/` or `C-_`, restoring text, cursor and clean state. Adjacent typing forms one undo step until another command or save; compound edits remain whole. The `undo-redo` command reapplies undone edits; making a new edit discards the redo future.
- See cursor position, modification and read-only state in the status line while errors remain in a separate echo area.
- Protect unsaved text: `C-x C-c` refuses to exit while any buffer has changes or an unresolved save.
