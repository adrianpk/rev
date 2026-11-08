# Pair Preferences and Native State

Status: Draft
Date: 2026-11-08
Specification: [Delimiter Pairs](editor-delimiter-pairs.md)
Review scope: canonical draft; no runtime or stored configuration is changed

## Configuration proposal

Use the existing startup settings loader, origins, validation and per-language
merge owners. No new state/data file, settings directory or startup discovery.
Missing values use compiled defaults. Unknown/invalid leaves follow the normal
whole-candidate diagnostic/fallback contract; runtime commands never save settings.

Proposed global leaves:

    [pairs]
    enabled = true
    wrap_selection = true
    delete_empty = true
    skip_closing = true
    quotes = true
    highlight = "depth"
    pulse_ms = 250

highlight admits depth, match, pulse and off. The owner selected depth as the
default and match as the restrained alternative. pulse_ms is an integer in 50..1000.
Quotes remain subject to the language profile; true does not force apostrophe,
lifetime, raw-string or template transformations. These are draft settings,
not a claim that the current binary accepts them.

Per-language overrides belong to the existing language preference model. Editing
leaves may override enabled/wrapping/deletion/skipping/quotes. The document must
not invent project-local config discovery or a directory per language. Exact
integration with existing language-table admission is verified before approval.

Proposed visual leaves:

    [palette.pairs]
    match_background = "#283457"
    match_text = "#c0caf5"
    mismatch = "#e0af68"
    depth_colors = ["#7aa2f7", "#e0af68", "#bb9af7", "#7dcfff", "#9ece6a", "#ff9e64"]

Colors use existing opaque #RRGGBB parsing; depth_colors has 1..8 entries. Existing
syntax roles remain authoritative outside eligible delimiters. Monochrome uses
supported attributes without changing source bytes, cursor position or glyphs.
Underline is the proposed active-pair attribute in match/depth/pulse presentation.
Resolve final overlay order with current selection/search/diagnostic ownership.

## Runtime model

| Value | Qualified fields and lifetime |
| --- | --- |
| Pair profile | Recognized language, admitted delimiters/quotes, lexical conditions; derived from current settings/syntax |
| Buffer policy | Initial merged editing preferences plus optional buffer-local runtime override |
| Pair result | Buffer id, revision, opening/closing source spans, pair kind, nesting, provenance and confirmed/unmatched/uncertain state |
| Match request | Source/view/point revision and finite scan/work budget |
| View presentation | Selected style, last non-off style, caret/viewport identity and source-qualified visible spans |
| Pulse | One generation, qualified spans and monotonic expiry; at most one pending mark per view |

Coordinates use native source units; terminal cells are derived only during
projection. Quotes may have identical visible endpoints but retain explicit
opening/closing roles. No raw tree nodes/pointers survive a syntax revision.
Malformed or limited matching never publishes a confirmed pair.

A buffer editing override affects every view of that buffer. Visual override and
pulse state belong to one view. Closing/reloading/replacing the source cancels
stale requests and overlays. Resizing reprojects the same request and expiry;
it cannot restart a pulse. Offscreen feedback is derived text, not a navigation
or history transaction. No runtime preference is persisted automatically.

## Edit ownership

The pair owner proposes one native source transaction with source-precondition,
selection/mark mapping and cursor placement. The ordinary edit owner admits it
and creates its undo unit. Overtyping a closer is a navigation operation; deleting
or wrapping publishes source edits. No unchecked byte-index insertion path.

Input provenance distinguishes physical typed characters, paste, completion and
other accepted literal edits. Generated mates are part of the same source effect,
not new synthetic keyboard events. Replay observes the same raw-versus-literal
contract to avoid second-generation mates. Pairing does not own the kill ring,
macro library, terminal prompt, search query or provider output.

## Proposed bounded retention

- Fallback scan: at most 256 KiB per request, nesting at most 256.
- Presentation: at most 8,192 eligible delimiter spans per view.
- Pair metadata: at most 2 MiB per buffer.
- Cooperative slice: at most 256 tokens and a four-millisecond scheduling target.
- Pulse: one finite expiry per view; no deadline after it clears.

These bounds require native measurement before becoming an approved contract.
Shared/cached allocations count once under an explicit owner. No whole-file
recoloring or reparsing is justified solely by a caret move. No background service,
permanent timer, external dependency or separate persistent pairing database.
