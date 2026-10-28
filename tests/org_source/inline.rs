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
