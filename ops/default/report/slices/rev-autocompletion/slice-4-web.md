# Slice 4: HTML, CSS, JavaScript and TypeScript

Status: delivered
Delivery set: rev-autocompletion
Plan: [Native Autocompletion Delivery](../../../plan/rev-autocompletion.md)
Tracker: [Native Autocompletion Tracker](../../../tracker/rev-autocompletion.md)
Branch: `feat/rev-autocompletion-04-web`
PR: [#198](https://forge.adrianpk.com/adrianpk/rev/pulls/198)

## Purpose

Deliver the web-language subset of CMP03 and CMP05-CMP07 through the existing
syntax owner, bounded open-buffer observations and checked completion interaction.

## Delivered Behavior

HTML, CSS, JavaScript, TypeScript and TSX have bundled parsers, highlight queries
and explicit extraction queries. Script candidates include functions, classes,
types, import bindings/aliases, members and local/parameter names. Local bindings
respect their block/function scopes; JavaScript var bindings retain function
scope. TSX uses its own parser and JSX colors. First visiting `.tsx` names retain
the `typescript` settings identity, including explicit plain/grammar overrides.
TS and TSX file buffers reuse compatible same-language observations.

HTML combines observed tags/attributes with finite common standard vocabulary
in their structural contexts. It excludes prose, attribute values, comments and
raw script/style bodies. CSS offers observed selectors and property names, a
finite property-name catalog, and observed custom properties inside var().
Ordinary CSS values/strings do not receive a value catalog. The compiled subsets
contain 72 HTML tags, 48 attributes and 84 CSS properties. Their standard sources
and actual context limits are linked in the completion reference.

Automatic requests admit HTML tag delimiters/attribute spacing and CSS selector/
declaration delimiters. Script comments, strings, ordinary templates and JSX text
retain suppression; template substitutions retain code admission. Hyphenated
HTML/CSS names and dollar/private script identifiers use whole checked literal
ranges. Shared selection, cancellation, literal edit and one-step undo remain
unchanged. Rendering projects every admitted grammar through the existing
semantic palette and grapheme/cell rules.

## Implementation Notes

Pinned grammar additions are HTML 0.23.2, CSS 0.25.0, JavaScript 0.25.0 and
TypeScript/TSX 0.23.2, with the single existing Tree-sitter 0.27.1 engine. TS/TSX
highlight queries compose the applicable JavaScript/type/JSX payloads; ordinary
TypeScript does not receive the JSX-only query. Each buffer still owns one
retained parser, tree and compiled queries.

The existing temporary worker derives current context, queries declarations,
adds the applicable finite catalog, filters/ranks and returns proposals. Catalogs
rank after observed sources within a matching class and count against aggregate
symbol/payload admission. The 2 MiB per-buffer, 8 MiB total snapshot, 32-buffer,
4096 per-buffer/16384 aggregate symbol, 8 MiB payload, 512-byte label and 64 KiB
insertion bounds remain enforced. One active worker, one newest pending request,
cancellation and the 50 ms job deadline remain unchanged.

The HTML closing-tag regression exposed an error tree containing no tag-name
node. Recovery examines only the current bounded identifier and its literal
opening/closing delimiter at the error node's start. It supplies context only;
observed names still come from explicit queries. Literal/value suppression,
source/history assertions and work limits remain intact. Current source pins
still retire proposals after revision, path, language, grammar or buffer changes.

## Contracts Added or Changed

Grammar choices now include html, css, javascript, typescript and tsx alongside
the existing choices. HTML/HTM and CSS associations join the built-in table.
Existing `.tsx` language preference resolution is preserved while its parser and
mode label are distinct. The complete sample/default catalog, syntax/completion
references and the existing Unreleased completion entry describe these outcomes.
No new preference leaves or commands are introduced.

## Files of Interest

- `src/syntax/catalog.rs`, `script.scm`, `javascript.scm`, `typescript.scm`: parser/query selection.
- `src/syntax/html.scm`, `css.scm`, `web.rs`: structural observations and context admission.
- `src/completion/context.rs`, `vocabulary.rs`, `target.rs`: bounded catalogs and identifier ranges.
- `src/buffer/syntax.rs`, `src/session/completion/sources.rs`: TSX identity and source compatibility.
- `tests/completion/web.rs`: real web workflows, scopes, contexts, colors and edit/undo.
- `tests/files/associations.rs`, `tests/syntax/preferences.rs`: alias and grammar-policy contracts.

## Validation

Development `cargo check --offline` passed and updated the lockfile with only
the four approved grammar packages. The real five-parser frame/manual edit/undo
case passed. The six web integration cases passed after the closing-tag context
correction, including declarations/imports, lexical scopes, structural/catalog
filtering, automatic suppression/triggers, TS/TSX reuse and literal ranges.
A multi-target development invocation used the web filter: six completion cases
ran; configuration, file and syntax cases were filtered out. Their targeted and
configured checks are recorded separately rather than claimed from that filter.

The initial recovered-end-tag unit check confirmed the parser's actual error
shape and failed its unchanged Context::Tag assertion. The bounded context
correction retains that assertion for fresh recorded validation.

T4.1 is commit `7adec3f3b4beceb3789c49e4b136fe3b0d4afb03`.
T4.2 is commit `e5d4eeba950799674a6e018183c4504da2917fc9`.
Focused controller receipt `THY6F2VMXIYQZAM2QBJCC3MJDM` passed at clean
candidate `0c0277a12e4500fe51a75071ca151ca9bbaf4afa`:

1. `cargo test --locked --lib completion`: 11 tests passed.
2. `cargo test --locked --lib syntax::web`: one parser recovery test passed.
3. `cargo test --locked --test completion web`: six web tests passed.
4. `cargo test --locked --test files typescript_aliases`: one alias test passed.
5. `cargo test --locked --test syntax preferences::grammar_policy`: one grammar policy test passed.
6. `cargo clippy --locked --lib -- -D warnings`: passed.
7. `cargo run --locked --quiet --example source-check -- --file src/buffer/policy.rs --file src/buffer/syntax.rs --file src/completion.rs --file src/completion/job.rs --file src/completion/context.rs --file src/completion/vocabulary.rs --file src/completion/target.rs --file src/config.rs --file src/config/resolve.rs --file src/config/associations.rs --file src/session/completion/sources.rs --file src/syntax.rs --file src/syntax/catalog.rs --file src/syntax/completion.rs --file src/syntax/scope.rs --file src/syntax/style.rs --file src/syntax/web.rs`: passed.
8. `cargo run --locked --quiet --example documentation-check`: passed.
9. `git diff --check`: passed.

Documentation admitted 326 commands, 198 configuration entries, 60 reachable
pages and 347 local links/anchors. Canonical PR #198 links the full immutable
report-introduction commit. Final controller receipt
`5GMST5QOZS5KBPNTHKKH4PAPQP` passed all three configured checks and 270 tests
at clean candidate `b93665235c4b29954d2c53f824904dd232e6e1a3` in this order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test completion --test syntax --test configuration --test files`
3. `git diff --check`

Canonical PR #198 merged that exact candidate into dev through Rebase +
Fast-forward. Slice 4 of 8 is delivered.

## Risks and Follow-ups

Names are syntactic observations, not semantic imports, receiver types or schema
knowledge. Finite HTML/CSS subsets do not model element-specific validity or
injected languages. Integrated footprint/PTY/resource acceptance and the remaining
sources retain their approved slices; the full delivery-set gate remains pending.
