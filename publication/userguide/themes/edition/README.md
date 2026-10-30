# Edition

## Purpose

Provide an optional editorial presentation for an HTML publication.

## Includes

- An HTML template with navigation, landmarks, and a stylesheet.
- A warm palette, reading and interface font stacks, responsive layout, and print rules.
- Styles for the semantic HTML blocks emitted by Spore.

## Apply to a project

Copy this directory to `themes/edition/` in the publication and select it in `spore.toml`:

```toml
[book]
theme = "edition"
```

## Customize

Edit `styles/book.css` for presentation changes and `templates/book.html.spt` for page structure. Keep the `book.css` asset name and its template reference aligned.
