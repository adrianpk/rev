use std::time::{Duration, Instant};

use rev::buffer::edit::History;
use rev::buffer::{Buffers, Error};
use rev::coordinates::{Byte, Coordinates, Scalar};
use rev::file::{self, Loaded};
use rev::org::{
    Cancellation, Coverage, InlineContext, InlineKind, InlineScan, Job, JobError, Limit, Limits,
    Source,
};

use super::inline::{nodes, parse};

// Original byte offsets must remain distinct from scalar and display positions
// across combining sequences, wide characters, emoji and CRLF source boundaries.
#[test]
fn unicode_ranges() {
    let text = "e\u{301} 界 *λbold* [[file:界.org][👩🏽‍💻]]\r\n\\alpha{}";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(text).unwrap();
    let buffer = buffers.get(id).unwrap();
    let observed = parse(buffer, InlineContext::Standard, Limits::default());
    let expected = ["*λbold*", "[[file:界.org][👩🏽‍💻]]", "\\alpha{}"];
    assert_eq!(observed.len(), expected.len());
    let coordinates = Coordinates::new(buffer).unwrap();
    for (index, raw) in expected.iter().enumerate() {
        let node = observed.node(buffer, index).unwrap();
        let byte = text.find(raw).unwrap();
        let scalar = Scalar(text[..byte].chars().count());
        assert_eq!(
            coordinates
                .to_byte(Scalar(node.source.span().range.start))
                .unwrap(),
            Byte(byte)
        );
        assert_eq!(coordinates.from_byte(Byte(byte)).unwrap(), scalar);
        assert_eq!(
            Source::new(buffer).text(&node.source).unwrap().to_string(),
            *raw
        );
        for range in [Some(node.content), node.detail].into_iter().flatten() {
            let start = coordinates
                .to_byte(Scalar(range.span().range.start))
                .unwrap();
            let end = coordinates.to_byte(Scalar(range.span().range.end)).unwrap();
            assert!(text.is_char_boundary(start.0) && text.is_char_boundary(end.0));
        }
    }
}

// Every resource stop is explicit and retains only admitted complete nodes.
// A clipped UTF-8 scalar or unfinished token cannot be published as a valid form.
#[test]
fn observation_limits() {
    let cases = [
        (
            "bytes",
            "*a* /b/",
            Limits::new(4, 10, 10).unwrap(),
            Coverage::Incomplete(Limit::AnalysisBytes),
            1,
        ),
        (
            "UTF-8 clip",
            "~界~",
            Limits::new(3, 10, 10).unwrap(),
            Coverage::Incomplete(Limit::AnalysisBytes),
            0,
        ),
        (
            "nodes",
            "*a* /b/",
            Limits::new(100, 1, 10).unwrap(),
            Coverage::Incomplete(Limit::SourceNodes),
            1,
        ),
        (
            "nested depth",
            "*outer /inner/*",
            Limits::new(100, 10, 1).unwrap(),
            Coverage::Incomplete(Limit::SourceDepth),
            1,
        ),
        (
            "balanced depth",
            "[fn::x [y]]",
            Limits::new(100, 10, 1).unwrap(),
            Coverage::Incomplete(Limit::SourceDepth),
            0,
        ),
        (
            "exact depth",
            "*plain*",
            Limits::new(100, 1, 1).unwrap(),
            Coverage::Complete,
            1,
        ),
        (
            "exact bytes",
            "~界~",
            Limits::new(5, 1, 1).unwrap(),
            Coverage::Complete,
            1,
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, text, limits, coverage, count) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let observed = parse(buffer, InlineContext::Standard, limits);
        assert_eq!(observed.coverage(), coverage, "{name}");
        assert_eq!(observed.len(), count, "{name}");
        assert!(
            nodes(buffer, &observed)
                .iter()
                .all(|(_, raw)| text.contains(raw)),
            "{name}"
        );
    }
}

// Analysis leaves unknown directives, malformed links, BOM and separator state
// unchanged. An unrelated edit and undo retain exact file serialization.
#[test]
fn unknown_fidelity() {
    let original = "\u{feff}Text \\unknown{} [[broken and *valid*\r\n#+FUTURE: retain\t \r\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .load(Loaded::read(original.as_bytes()).unwrap())
        .unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let observed = parse(buffer, InlineContext::Standard, Limits::default());
    assert_eq!(
        nodes(buffer, &observed),
        [(InlineKind::Bold, "*valid*".into())]
    );
    let mut bytes = Vec::new();
    file::write(buffer, &mut bytes).unwrap();
    assert_eq!(bytes, original.as_bytes());
    super::incremental::replace(buffer, 0..4, "Edited");
    bytes.clear();
    file::write(buffer, &mut bytes).unwrap();
    assert_eq!(bytes, original.replacen("Text", "Edited", 1).as_bytes());
    assert!(observed.node(buffer, 0).is_err());
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    bytes.clear();
    file::write(buffer, &mut bytes).unwrap();
    assert_eq!(bytes, original.as_bytes());
    assert!(observed.node(buffer, 0).is_err());
}

// Unknown or unmatched openers must recover across rope chunks. A missing
// delimiter is remembered per context instead of rescanning the entire suffix.
#[test]
fn malformed_chunks() {
    let mut buffers = Buffers::new().unwrap();
    for prefix in ["*unclosed ", "<bad ", "[[bad ", "<<bad "] {
        let text = format!("{}\n\n*valid*", prefix.repeat(3000));
        let id = buffers.create(&text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let observed = parse(buffer, InlineContext::Standard, Limits::default());
        assert_eq!(
            nodes(buffer, &observed),
            [(InlineKind::Bold, "*valid*".into())],
            "{prefix}"
        );
        assert_eq!(observed.coverage(), Coverage::Complete);
    }
}

// Inline snapshots share the ordinary job owner. Live edits must reject an old
// completion before it can replace observations for the new canonical revision.
#[test]
fn qualified_worker() {
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.create("*before*").unwrap();
    let other = buffers.create("*before*").unwrap();
    let buffer = buffers.get(first).unwrap();
    let range = Source::new(buffer)
        .range(Byte(0)..Byte(buffer.len_bytes()))
        .unwrap();
    assert!(
        InlineScan::new(
            buffers.get(other).unwrap(),
            &range,
            InlineContext::Standard,
            Limits::default()
        )
        .is_err()
    );
    let scan = InlineScan::new(buffer, &range, InlineContext::Standard, Limits::default()).unwrap();
    let mut job = Job::default();
    job.start(buffer.stamp(), move |cancel| scan.run(cancel))
        .unwrap();
    let buffer = buffers.get_mut(first).unwrap();
    super::incremental::replace(buffer, 1..7, "after");
    let started = Instant::now();
    let result = loop {
        if let Some(result) = job.poll(buffer) {
            break result;
        }
        assert!(started.elapsed() < Duration::from_secs(2));
        std::thread::yield_now();
    };
    assert!(matches!(result, Err(JobError::Source(Error::Stale { .. }))));
    assert!(!job.is_active());
}

// Cancellation also applies before the first byte and in a literal context;
// a cancelled observation never claims complete empty coverage.
#[test]
fn cancelled_observation() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("*bold* [[x]]").unwrap();
    let buffer = buffers.get(id).unwrap();
    let range = Source::new(buffer)
        .range(Byte(0)..Byte(buffer.len_bytes()))
        .unwrap();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    for context in [
        InlineContext::Standard,
        InlineContext::Minimal,
        InlineContext::Literal,
    ] {
        let result = InlineScan::new(buffer, &range, context, Limits::default())
            .unwrap()
            .run(&cancelled);
        assert_eq!(result.coverage(), Coverage::Cancelled);
        assert!(result.is_empty());
    }
}
