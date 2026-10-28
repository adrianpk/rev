use rev::buffer::Buffers;
use rev::coordinates::Byte;
use rev::org::{Cancellation, Coverage, Limit, Limits, Scan, Source};

pub(super) fn complete(mut scan: Scan) -> rev::org::Index {
    let cancellation = Cancellation::default();
    while scan.advance(&cancellation) == Coverage::Pending {}
    scan.into_index()
}

// A skipped heading level still belongs to the nearest preceding lower level;
// line offsets must cover the exact UTF-8/CRLF source without normalizing it.
#[test]
fn outline_rows() {
    let rows = [
        ("preamble\r\n", None, None),
        ("* Root\r\n", Some(1), None),
        ("body\t界\r\n", None, Some(1)),
        ("*** Child\n", Some(3), Some(1)),
        ("note\n", None, Some(3)),
        ("** Peer\n", Some(2), Some(1)),
        ("* Other\n", Some(1), None),
        ("end", None, Some(6)),
    ];
    let text: String = rows.iter().map(|(text, _, _)| *text).collect();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&text).unwrap();
    let buffer = buffers.get(id).unwrap();
    let index = complete(Scan::new(buffer, Limits::default()).unwrap());
    assert_eq!(index.coverage(), Coverage::Complete);
    assert_eq!(index.len(), rows.len());
    assert_eq!(index.covered_bytes(), Byte(text.len()));
    let mut start = 0;
    for (number, (text, level, parent)) in rows.iter().enumerate() {
        let line = index.line(buffer, number).unwrap();
        assert_eq!(line.bytes, Byte(start)..Byte(start + text.len()));
        assert_eq!((line.outline_level, line.parent), (*level, *parent));
        assert_eq!(
            Source::new(buffer).text(&line.source).unwrap().to_string(),
            *text
        );
        start += text.len();
    }
}

// EOF is a real logical line, while an indented or undelimited star prefix is
// ordinary source. Unknown/malformed content remains present in the line index.
#[test]
fn source_edges() {
    let cases = [
        ("empty", "", 1, None),
        ("no separator", "body", 1, None),
        ("final LF", "body\n", 2, None),
        ("CRLF", "\r\n", 2, None),
        ("empty heading", "*", 1, Some(1)),
        ("tab delimiter", "**\tHeading", 1, Some(2)),
        ("no delimiter", "***plain", 1, None),
        ("indented prefix", " * Heading", 1, None),
        ("unknown directive", "#+UNKNOWN: retained", 1, None),
        ("malformed block", "#+BEGIN_SRC rust", 1, None),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, text, count, level) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let index = complete(Scan::new(buffer, Limits::default()).unwrap());
        assert_eq!(index.coverage(), Coverage::Complete, "{name}");
        assert_eq!(index.len(), count, "{name}");
        assert_eq!(
            index.line(buffer, 0).unwrap().outline_level,
            level,
            "{name}"
        );
        assert_eq!(index.covered_bytes(), Byte(text.len()), "{name}");
        assert!(buffer.history().is_empty(), "{name}");
    }
}

// Each ceiling reports its own incomplete state and exposes only complete
// original lines; splitting a multibyte scalar never creates a bogus location.
#[test]
fn coverage_limits() {
    let cases = [
        (
            "byte prefix",
            "* Root\nbody",
            Limits::new(8, 10, 10).unwrap(),
            Coverage::Incomplete(Limit::AnalysisBytes),
            1,
            7,
        ),
        (
            "UTF-8 limit",
            "α\n界",
            Limits::new(5, 10, 10).unwrap(),
            Coverage::Incomplete(Limit::AnalysisBytes),
            1,
            3,
        ),
        (
            "node count",
            "* Root\nbody\n",
            Limits::new(100, 1, 10).unwrap(),
            Coverage::Incomplete(Limit::SourceNodes),
            1,
            7,
        ),
        (
            "outline depth",
            "* Root\n*** Deep\n",
            Limits::new(100, 10, 2).unwrap(),
            Coverage::Incomplete(Limit::SourceDepth),
            1,
            7,
        ),
        (
            "exact bounds",
            "* Root",
            Limits::new(6, 1, 1).unwrap(),
            Coverage::Complete,
            1,
            6,
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, text, limits, state, rows, bytes) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let index = complete(Scan::new(buffer, limits).unwrap());
        assert_eq!(index.coverage(), state, "{name}");
        assert_eq!(index.len(), rows, "{name}");
        assert_eq!(index.covered_bytes(), Byte(bytes), "{name}");
    }
}

// Configuration cannot request an unbounded observation or exceed the selected
// profile maxima; the approved factory values remain independently asserted.
#[test]
fn admitted_limits() {
    assert_eq!(
        Limits::default(),
        Limits::new(8_388_608, 200_000, 256).unwrap()
    );
    assert!(Limits::new(33_554_432, 1_000_000, 1024).is_ok());
    let cases = [
        ("zero bytes", 0, 1, 1, Limit::AnalysisBytes),
        ("too many bytes", 33_554_433, 1, 1, Limit::AnalysisBytes),
        ("zero nodes", 1, 0, 1, Limit::SourceNodes),
        ("too many nodes", 1, 1_000_001, 1, Limit::SourceNodes),
        ("zero depth", 1, 1, 0, Limit::SourceDepth),
        ("too deep", 1, 1, 1025, Limit::SourceDepth),
    ];
    for (name, bytes, nodes, depth, error) in cases {
        assert_eq!(Limits::new(bytes, nodes, depth), Err(error), "{name}");
    }
}

// A single long line cannot bypass step budgets by forcing a whole-line copy.
// UTF-8 at a step edge is deferred intact to a later bounded compute step.
#[test]
fn bounded_steps() {
    let text = format!("{}界{}", "a".repeat(4095), "b".repeat(8192));
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&text).unwrap();
    let mut scan = Scan::new(buffers.get(id).unwrap(), Limits::default()).unwrap();
    assert_eq!(scan.advance(&Cancellation::default()), Coverage::Pending);
    assert!(scan.scanned_bytes().0 <= 4096);
    assert!(text.is_char_boundary(scan.scanned_bytes().0));
    assert_eq!(scan.index().covered_bytes(), Byte(0));
    assert!(scan.index().is_empty());
    let index = complete(scan);
    assert_eq!(index.coverage(), Coverage::Complete);
    assert_eq!(index.covered_bytes(), Byte(text.len()));
    assert_eq!(index.len(), 1);
}
