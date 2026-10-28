use rev::buffer::{Buffer, Buffers};
use rev::coordinates::Byte;
use rev::org::{
    Cancellation, Coverage, InlineContext, InlineKind, InlineObservation, InlineScan, Limits,
    LinkKind, Source,
};

pub(super) fn parse(buffer: &Buffer, context: InlineContext, limits: Limits) -> InlineObservation {
    let range = Source::new(buffer)
        .range(Byte(0)..Byte(buffer.len_bytes()))
        .unwrap();
    InlineScan::new(buffer, &range, context, limits)
        .unwrap()
        .run(&Cancellation::default())
}

pub(super) fn nodes(buffer: &Buffer, observed: &InlineObservation) -> Vec<(InlineKind, String)> {
    (0..observed.len())
        .map(|index| {
            let node = observed.node(buffer, index).unwrap();
            (
                node.kind,
                Source::new(buffer).text(&node.source).unwrap().to_string(),
            )
        })
        .collect()
}

// Each source family retains exact delimiters and a separate content span. Link
// targets, entities and executable-looking text are observations without effects.
#[test]
fn surface_forms() {
    let cases = [
        ("*bold*", InlineKind::Bold, "bold"),
        ("/italic/", InlineKind::Italic, "italic"),
        ("_underline_", InlineKind::Underline, "underline"),
        ("+strike+", InlineKind::Strike, "strike"),
        ("~code~", InlineKind::Code, "code"),
        ("=verbatim=", InlineKind::Verbatim, "verbatim"),
        ("\\alpha{}", InlineKind::Entity, "alpha"),
        (
            "[[file:notes.org][Title]]",
            InlineKind::Link(LinkKind::Bracket),
            "file:notes.org",
        ),
        (
            "<https://example.test/a b>",
            InlineKind::Link(LinkKind::Angle),
            "https://example.test/a b",
        ),
        (
            "https://example.test/a",
            InlineKind::Link(LinkKind::Plain),
            "https://example.test/a",
        ),
        ("<<target>>", InlineKind::Target, "target"),
        ("<<<radio>>>", InlineKind::RadioTarget, "radio"),
        ("[fn:label]", InlineKind::FootnoteReference, "label"),
        ("[fn::note]", InlineKind::FootnoteInline, ""),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (text, kind, body) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let observed = parse(buffer, InlineContext::Standard, Limits::default());
        assert_eq!(observed.coverage(), Coverage::Complete, "{text}");
        assert_eq!(
            nodes(buffer, &observed),
            [(kind, text.to_string())],
            "{text}"
        );
        let node = observed.node(buffer, 0).unwrap();
        assert_eq!(
            Source::new(buffer).text(&node.content).unwrap().to_string(),
            body,
            "{text}"
        );
        assert_eq!(node.source.stamp(), buffer.stamp(), "{text}");
        assert!(buffer.history().is_empty(), "{text}");
    }
}

// Parent spans contain nested objects, but literal/code contents and link
// targets remain opaque. Radio targets and descriptions use minimal contexts.
#[test]
fn nested_contexts() {
    let cases = [
        (
            "*bold /italic/ and ~code *literal*~*",
            &[
                (
                    InlineKind::Bold,
                    "*bold /italic/ and ~code *literal*~*",
                    None,
                ),
                (InlineKind::Italic, "/italic/", Some(0)),
                (InlineKind::Code, "~code *literal*~", Some(0)),
            ][..],
        ),
        (
            "[[file:x][*bold* \\alpha]]",
            &[
                (
                    InlineKind::Link(LinkKind::Bracket),
                    "[[file:x][*bold* \\alpha]]",
                    None,
                ),
                (InlineKind::Bold, "*bold*", Some(0)),
                (InlineKind::Entity, "\\alpha", Some(0)),
            ][..],
        ),
        (
            "<<<*radio*>>>",
            &[
                (InlineKind::RadioTarget, "<<<*radio*>>>", None),
                (InlineKind::Bold, "*radio*", Some(0)),
            ][..],
        ),
        (
            "Text [fn:note:see /italics/ and [[x]]]",
            &[
                (
                    InlineKind::FootnoteInline,
                    "[fn:note:see /italics/ and [[x]]]",
                    None,
                ),
                (InlineKind::Italic, "/italics/", Some(0)),
                (InlineKind::Link(LinkKind::Bracket), "[[x]]", Some(0)),
            ][..],
        ),
        (
            "=x *literal* \\alpha=",
            &[(InlineKind::Verbatim, "=x *literal* \\alpha=", None)][..],
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (text, expected) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let observed = parse(buffer, InlineContext::Standard, Limits::default());
        let actual: Vec<_> = (0..observed.len())
            .map(|index| {
                let node = observed.node(buffer, index).unwrap();
                (
                    node.kind,
                    Source::new(buffer).text(&node.source).unwrap().to_string(),
                    node.parent,
                )
            })
            .collect();
        let expected: Vec<_> = expected
            .iter()
            .map(|(kind, raw, parent)| (*kind, raw.to_string(), *parent))
            .collect();
        assert_eq!(actual, expected, "{text}");
        assert_eq!(observed.coverage(), Coverage::Complete, "{text}");
    }
}

// Escaped target brackets retain their spelling, unknown entities stay source,
// and a malformed outer delimiter cannot swallow a later valid inner form.
#[test]
fn delimiter_recovery() {
    let cases = [
        (r"Text \*no* *yes*", &[(InlineKind::Bold, "*yes*")][..]),
        (
            r"[[x\]y][*bold*]]",
            &[
                (InlineKind::Link(LinkKind::Bracket), r"[[x\]y][*bold*]]"),
                (InlineKind::Bold, "*bold*"),
            ][..],
        ),
        (
            "[[bad [[good]]",
            &[(InlineKind::Link(LinkKind::Bracket), "[[good]]")][..],
        ),
        ("[[bad *good*", &[(InlineKind::Bold, "*good*")][..]),
        (
            r"\alpha2 \frac12 \alphaβ \unknown{}",
            &[
                (InlineKind::Entity, r"\alpha"),
                (InlineKind::Entity, r"\frac12"),
            ][..],
        ),
        (r"\_   ", &[(InlineKind::Entity, "\\_   ")][..]),
        (
            "[[x][a ] b]]",
            &[(InlineKind::Link(LinkKind::Bracket), "[[x][a ] b]]")][..],
        ),
        (
            "https://example.test/a(b(c)).",
            &[(
                InlineKind::Link(LinkKind::Plain),
                "https://example.test/a(b(c))",
            )][..],
        ),
        (
            "https://example.test/(unclosed",
            &[(InlineKind::Link(LinkKind::Plain), "https://example.test/")][..],
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (text, expected) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let observed = parse(buffer, InlineContext::Standard, Limits::default());
        let expected: Vec<_> = expected
            .iter()
            .map(|(kind, raw)| (*kind, raw.to_string()))
            .collect();
        assert_eq!(nodes(buffer, &observed), expected, "{text}");
    }
}

// Structural callers can select literal or minimal regions without changing
// their bytes. Malformed syntax is not repaired or represented as valid markup.
#[test]
fn excluded_forms() {
    let mut buffers = Buffers::new().unwrap();
    for text in [
        "* untrimmed*",
        "*trailing *",
        "*unclosed",
        "word*bold*",
        "<< space>>",
        "<<bad\nline>>",
        "[[x]",
        "[fn:bad label]",
        "<unknown:thing>",
        r"\alphabeta",
    ] {
        let id = buffers.create(text).unwrap();
        let observed = parse(
            buffers.get(id).unwrap(),
            InlineContext::Standard,
            Limits::default(),
        );
        assert!(observed.is_empty(), "{text}");
    }
    let id = buffers
        .create("*bold* [[x]] [fn:a] <<target>> \\alpha")
        .unwrap();
    let buffer = buffers.get(id).unwrap();
    assert!(parse(buffer, InlineContext::Literal, Limits::default()).is_empty());
    assert_eq!(
        nodes(
            buffer,
            &parse(buffer, InlineContext::Minimal, Limits::default())
        ),
        [
            (InlineKind::Bold, "*bold*".into()),
            (InlineKind::Entity, "\\alpha".into())
        ]
    );
}
