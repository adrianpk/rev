# User Guide publication

The Markdown in `docs/tutorials/user-guide/` is the source of the public guide.
Edit those files first. This directory supplies the Spore navigation and a
project-local copy of Spore's Edition theme, following its documented theme
installation procedure.

This publication starts in Night. The reader can choose another appearance;
Edition remembers that choice in the browser.

## Build

With Spore on `PATH`, run from the repository root:

```sh
scripts/build-user-guide.sh
```

The generated HTML is in `target/userguide/`, ready to mount at `/userguide/`.
The build keeps the editor learning path and separate Org guide together, with
supporting how-to, reference and explanation pages. Local documentation links stay in the
Spore publication, including heading anchors and configuration downloads.
Edit the Markdown learning path and `index.sptree` together when adding a chapter.
`cargo run --locked --quiet --example documentation-check` verifies the same
chapter order, previous/next links and live command/configuration inventories.
The language-server chapter and its downloadable TOML example use this same build.
Code blocks remain unchanged; Markdown tables become native Spore tables.
Standalone Markdown images become native Spore figures with their alternative
text and asset URLs beneath `/userguide/`.
Styles and appearance scripts use content-based filenames so updates do not
depend on a browser or CDN cache expiring.

The build uses Rust, Bash, standard Linux utilities and Spore. It does not change
the Markdown, publish the site or modify the editor's configuration.

## Preview and publish

Place the generated directory beside the landing page's `index.html`, named
`userguide`. Preview that complete site with `goservectl preview` so the
`/userguide/` mount and its assets match production.

After reviewing the preview, publish the complete static site through its
existing goserve service. The renderer's source and cache directories stay
outside the served site.
