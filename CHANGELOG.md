# Changelog

All notable changes to this project are documented here.

This project uses Semantic Versioning.

## Unreleased

### Added

- Open UTF-8 files at startup or with `C-x C-f`, preserving BOM, line endings and final-newline absence. Reopening a canonical file reuses its current buffer; absent paths open empty without creating files. Invalid UTF-8 and NUL-containing binary input produce an error without installing damaged text.
- Save buffers with `C-x C-s`, supplying a destination for unnamed text, or use `C-x C-w` to write another file. Occupied destinations and external changes require explicit confirmation; symlink saves preserve the link and write its resolved target. Saves preserve text format and existing access permissions. Failed saves retain edits and distinguish rejected writes from files already replaced with completion still unresolved.
- Split editing windows with `C-x 2` for upper/lower panes and `C-x 3` for left/right panes. Splits can nest, share the same text and retain the original focus; each view keeps its own cursor and scrolling. Splits that do not fit leave the layout unchanged. Resizing uses the minimum space required by each nested branch and safely suspends presentation when the layout cannot fit. Cycle editing panes with `C-x o`, close the selected pane with `C-x 0`, or keep only it with `C-x 1`; closing panes retains their buffers and unsaved text. Edits and undo/redo update shared cursors and visible line anchors independently; history restores its original cursor while that view remains live.
- Switch between live buffers with `C-x b` and Tab completion, or select one from `C-x C-b`. Close a buffer with `C-x k`, choosing save, discard or cancel for modified text; failed required saves keep it open. Closing the final buffer leaves a fresh empty editing buffer.
- Edit Unicode text directly in the terminal with Enter and Tab. Backspace and `C-d` remove complete graphemes, including CRLF line breaks. Kill to logical line end with `C-k`, or kill words forward with `M-d` and backward with `M-DEL`. Consecutive kills retain text in direction order, and each kill can be undone independently.
- Move by graphemes, Unicode words, logical line ends and buffer boundaries with Emacs bindings, keeping point visible as the viewport scrolls.
- Keep the intended display column when moving through short lines, move by pages, and cycle the current line through the center, top and bottom of the viewport with `C-l`.
- Undo edits with `C-x u` or a reported `C-/` or `C-_`, restoring text, cursor and clean state. Adjacent typing forms one undo step until another command or save; compound edits remain whole. The `undo-redo` command reapplies undone edits; making a new edit discards the redo future.
- Select linear text with `C-SPC`, extend it with motion, and exchange point and mark with `C-x C-x`. The selected pane highlights the region across rows and scrolling; `C-g` clears the selection while retaining its mark. Ordinary edits deactivate the region.
- See cursor position, modification and read-only state in the status line while errors remain in a separate echo area.
- Protect unsaved text: `C-x C-c` refuses to exit while any buffer has changes or an unresolved save.
