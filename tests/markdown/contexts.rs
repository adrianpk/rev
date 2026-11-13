use super::*;

// Exercise native captures from CommonMark source context, including literal
// ownership and reference links whose destinations are outside the visible row.
#[test]
fn source_contexts() {
    let cases = [
        (
            "emphasis",
            "*word*",
            0,
            "word",
            Some(MarkdownRole::Emphasis),
        ),
        (
            "strong",
            "**word**",
            0,
            "word",
            Some(MarkdownRole::Emphasis),
        ),
        ("escaped", "\\*word\\*", 0, "word", None),
        ("intraword", "one_word_two", 0, "word", None),
        (
            "heading inline",
            "# Title *word*",
            0,
            "word",
            Some(MarkdownRole::Emphasis),
        ),
        ("seven hashes", "####### word", 0, "word", None),
        ("Setext", "word\n====", 0, "word", None),
        (
            "link",
            "[word](https://example.com)",
            0,
            "word",
            Some(MarkdownRole::Link),
        ),
        (
            "reference",
            "[word][id]\n\n[id]: https://example.com",
            0,
            "word",
            Some(MarkdownRole::Link),
        ),
        (
            "autolink",
            "<https://example.com>",
            0,
            "example",
            Some(MarkdownRole::Link),
        ),
        (
            "image",
            "![word](image.png)",
            0,
            "word",
            Some(MarkdownRole::Link),
        ),
        (
            "inline literal",
            "`*word*`",
            0,
            "word",
            Some(MarkdownRole::Code),
        ),
        (
            "fence",
            "```md\n*word*\n```",
            1,
            "word",
            Some(MarkdownRole::Code),
        ),
        (
            "tilde fence",
            "~~~\n# word\n~~~",
            1,
            "word",
            Some(MarkdownRole::Code),
        ),
        (
            "unclosed fence",
            "```\n*word*",
            1,
            "word",
            Some(MarkdownRole::Code),
        ),
        (
            "indented code",
            "    *word*",
            0,
            "word",
            Some(MarkdownRole::Code),
        ),
        ("quote", "> word", 0, "word", Some(MarkdownRole::Quote)),
        ("unordered", "- word", 0, "word", Some(MarkdownRole::List)),
        ("ordered", "3. word", 0, "word", Some(MarkdownRole::List)),
        (
            "nested",
            "- Parent\n  + word",
            1,
            "word",
            Some(MarkdownRole::List),
        ),
        (
            "table",
            "| word | value |\n| --- | --- |",
            0,
            "word",
            Some(MarkdownRole::Table),
        ),
        (
            "outer pipes optional",
            "word | value\n--- | ---",
            0,
            "word",
            Some(MarkdownRole::Table),
        ),
        ("invalid table", "word | value\n---", 0, "word", None),
        ("HTML literal", "<div>\n*word*\n</div>", 1, "word", None),
        ("extension off", "~~word~~", 0, "word", None),
    ];
    for (name, source, row, token, expected) in cases {
        let (_fixture, mut session) = editor("", "notes.md", source);
        settle(&mut session);
        assert_eq!(
            style_at(&session.frame().unwrap(), row, token),
            expected.map(Style::Markdown),
            "{name}"
        );
        assert_eq!(text(session.buffer().unwrap()), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Enclosing quote/list/table spans cannot override inline semantics. A complete
// literal block owns Markdown-looking headings, lists and tables in its body.
#[test]
fn nested_precedence() {
    for (name, source, row) in [
        ("quote", "> *word* and `code`", 0),
        ("list", "- *word* and `code`", 0),
        ("table", "| *word* | `code` |\n| --- | --- |", 0),
    ] {
        let (_fixture, mut session) = editor("", "notes.md", source);
        settle(&mut session);
        let frame = session.frame().unwrap();
        assert_eq!(
            style_at(&frame, row, "word"),
            Some(Style::Markdown(MarkdownRole::Emphasis)),
            "{name}"
        );
        assert_eq!(
            style_at(&frame, row, "code"),
            Some(Style::Markdown(MarkdownRole::Code)),
            "{name}"
        );
    }
    let source = "```md\n# heading\n- item\n| head | value |\n| --- | --- |\n```";
    let (_fixture, mut session) = editor("", "notes.md", source);
    settle(&mut session);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .flat_map(|row| row.syntax())
            .all(|paint| paint.style == Style::Markdown(MarkdownRole::Code))
    );
}

// Factory recognition remains subject to explicit association precedence and
// markdown.enabled; optional external language services are not prerequisites.
#[test]
fn offline_recognition() {
    for (name, config, expected) in [
        ("notes.md", "lsp.enabled=false", "Markdown"),
        ("notes.markdown", "", "Markdown"),
        ("notes.MD", "", "Text"),
        ("notes.md", "files.associations.'*.md'='plain'", "Text"),
        (
            "notes.txt",
            "files.associations.'*.txt'='markdown'",
            "Markdown",
        ),
        ("notes.md", "markdown.enabled=false", "Text"),
        ("notes.rs", "", "Rust"),
        ("notes.org", "", "Org"),
    ] {
        let (_fixture, mut session) = editor(config, name, "*word*\n");
        settle(&mut session);
        assert_eq!(
            session.buffer().unwrap().mode(),
            expected,
            "{name}, {config}"
        );
        assert_eq!(text(session.buffer().unwrap()), "*word*\n");
    }
}
