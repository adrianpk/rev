# Changelog

All notable changes to this project are documented here.

This project uses Semantic Versioning.

## Unreleased

### Added

- Configure existing editing, display, palette, history, file and context-keybinding preferences through startup TOML under the XDG configuration directory. Explicit includes and language overrides compose before startup; invalid sources show a diagnostic and retain complete defaults. Configuration changes take effect after restarting, and startup creates no settings files or empty XDG directories.
- Edit Unicode text directly in the terminal, with whole-grapheme movement and deletion, literal tabs, visible controls and scrolling that retains the intended display column. The status shows position, modification, read-only state and language mode.
- Open UTF-8 files at startup or with `C-x C-f`, preserving BOM, LF/CRLF and final-newline presence. Canonical paths reuse live buffers; absent paths start empty without creating files. Invalid UTF-8 and NUL-containing binary input reject without replacing live text.
- Complete find-file paths in a vertical list from the visiting file's directory or cwd. Tab fills selected or unique names, extends shared prefixes and descends into directories; Up/Down or C-p/C-n selects candidates. Relative, absolute, home, space and Unicode paths retain their identities, incomplete lists show a warning, and resize or cancellation preserves the editing workspace.
- Save with `C-x C-s` or write another path with `C-x C-w`. Occupied destinations and external changes require fresh consent; symlink saves preserve the link and write its resolved target. Saves preserve access permissions and distinguish rejected writes from published files whose completion remains unresolved.
- Switch, list and close buffers with `C-x b`, `C-x C-b` and `C-x k`. Modified close asks for save/discard/cancel; a failed required save keeps the buffer open. Closing the final buffer leaves a fresh editing buffer.
- Create nested shared-text windows with `C-x 2` and `C-x 3`; cycle with `C-x o`, close a view with `C-x 0` or keep only it with `C-x 1`. Each view retains its own cursor and scrolling. Tiny/resized terminals retain text, layout and history while presentation adapts or suspends.
- Undo with `C-x u` or reported `C-/`/`C-_`, and redo with `M-x undo-redo`. Adjacent typing shares one undo step until another command or save; paste and compound edits undo whole. History remains per buffer across saves, restores retained clean states and maps shared views; a new edit discards redo.
- Select a linear region with `C-SPC`, extend it with motion, exchange point and mark with `C-x C-x`, or clear selection with `C-g`. Copy with `M-w`, kill with `C-w` and yank with `C-y`. Immediately after yank, `M-y` rotates entries within that exact insertion. The session retains 32 linear entries; directional line/word kills coalesce in document order.
- Select display-column rectangles with `C-x SPC`, copy with `C-x r M-w`, kill with `C-x r k` and yank with `C-x r y`. Whole graphemes and partial-tab spaces preserve adjacent text. Short source rows gain no absent trailing text; insertion adds only required padding and lines. Each block mutation undoes as one group, separate from the linear ring.
- Find commands with `M-x`, a vertical candidate list, configurable prefix/substring matching and row limits, Up/Down or C-p/C-n candidate selection, Tab completion and separate M-p/M-n session history. Return invokes the selected or exact typed command once. The highlighted candidate stays visible through navigation and resize without scrolling the document underneath. Named commands use the same handlers and focused arguments as bindings; `C-g` cancels without inserting prompt text.
- Search literally with `C-s`/`C-r`, using Unicode smart case and visible matches across lines. Repeat either chord to continue without implicit wrap; Return accepts and `C-g` restores the starting view. `M-%` confirms each query-replace occurrence with y/n/q, retaining one undo step per accepted change.
- Paste bracketed multiline Unicode literally as one undo step, preserving LF/CRLF and keeping command-looking text out of key decoding. Single-line argument paste stays in its prompt; decision prompts require explicit keys.
- Show/focus the left file tree with `C-x t t`/`C-x t f`. Navigate with arrows, expand lazily and refresh without restarting. Directory symlinks remain leaves; narrow terminals hide the pane and release its focus. Return opens files through normal buffer reuse.
- Create tree files/directories with `C-x t n`/`C-x t d` and rename with `C-x t R`, retaining visited text and reconciling descendant paths. Delete with `C-x t D`, confirming again for a nonempty directory. Open text survives deletion and requires explicit saving to recreate files; symlink removal retains referents.
- Highlight `.rs` files with bundled Rust syntax and Tokyo Night RGB syntax colors, without installing a grammar. Unicode edits, paste, rectangles and undo/redo keep colors aligned across shared views. Unsupported files and syntax failures stay editable as plain text.
- Exit with `C-x C-c` or `M-x save-buffers-kill-terminal`, deciding save/discard/cancel for each modified buffer, including hidden and unnamed text. Cancel retains the live session and earlier completed saves. Failed or unresolved required saves stop exit.
- Use reported Alt/Meta or a compatible Escape prefix with a 100-ms deadline. Standalone Escape clears pending key prefixes while preserving focused prompts; `C-g` cancels the active interaction. The [terminal input reference](docs/reference/terminal-input.md) documents supported reports and physical-terminal limits.

### Changed

- The terminal UI uses Tokyo Night RGB colors for document text, status, tree, separators, selections, search and messages. Supported terminals apply cursor color and restore their default colors on exit or failure.

- Editing panes and the file tree share continuous Unicode separators and connected junctions. Stacked panes reserve a separator row in addition to their status rows.

- Tab inserts spaces to the next four-column indentation stop by default. Pasted tab characters remain literal.

### Fixed

- Rectangle copy expands selected tabs into retained spaces without changing source text; rectangle kill preserves complete tabs. Yanking aligns row starts at the destination column, retaining whole destination graphemes and using the destination tab stops.
- Visiting a file through an alias retains its visible basename for the buffer label and language selection, while saves and repeated visits share the canonical destination.
- Terminal cleanup attempts every restoration step after an output error, reports failed steps together and still releases raw input.
- Saving rechecks the destination after staging temporary output, preserving changed, removed or replaced external data observed before replacement and requiring a new decision.
- Tree creation rejects a directory replaced while its name prompt was open, keeping entries out of the replacement directory.
- Canceling `M-x` retains the intended vertical column and recenter cycle. Argument edits clear outdated feedback; Tab in write/save prompts retains the draft and displays path-entry guidance.
