use std::time::{Duration, Instant};

use rev::buffer::Buffers;
use rev::file::Loaded;
use rev::org::{
    Cancellation, Coverage, Job, JobError, Limit, Limits, Source, StructureKind as Kind,
    StructureScan,
};

use super::structure::{nodes, parse, text};

// Malformed drawers stop at the next real heading. Ordinary drawers after
// paragraphs still own their bodies; a nested drawer spelling is not a drawer.
#[test]
fn drawer_recovery() {
    let cases = [
        ("missing close", "* H\n:LOGBOOK:\nbody\n* Next\n", 0, 1),
        (
            "close past heading",
            "* H\n:LOGBOOK:\n* Next\n:END:\n",
            0,
            1,
        ),
        (
            "after paragraph",
            "* H\nbody\n:LOGBOOK:\ninside\n:END:\n* Next\n",
            1,
            0,
        ),
        (
            "nested spelling",
            "* H\n:LOGBOOK:\n:INNER:\nbody\n:END:\n* Next\n",
            1,
            0,
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, drawers, malformed) in cases {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let structure = parse(buffer);
        let nodes = nodes(buffer, &structure);
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.kind == Kind::Drawer)
                .count(),
            drawers,
            "{name}"
        );
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.kind == Kind::Malformed)
                .count(),
            malformed,
            "{name}"
        );
        assert_eq!(
            nodes
                .iter()
                .filter(|node| matches!(node.kind, Kind::Heading { .. }))
                .count(),
            2,
            "{name}"
        );
        assert!(
            nodes
                .iter()
                .filter(|node| node.kind == Kind::Drawer)
                .all(|node| !text(buffer, node).contains("* Next")),
            "{name}"
        );
    }
}

// Empty property values and additive names retain exact local source. Invalid
// context turns planning/property-looking text into ordinary source observations.
#[test]
fn local_metadata() {
    let source = "* H\nDEADLINE: <2026-10-29 Thu>\n:PROPERTIES:\n:EMPTY:\n:WORDS+: 界 e\u{301}\n:END:\nbody\nSCHEDULED: <2026-10-30 Fri>\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let structure = parse(buffer);
    let nodes = nodes(buffer, &structure);
    let properties: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.kind, Kind::Property { .. }))
        .collect();
    assert_eq!(properties.len(), 2);
    assert!(properties[0].content.span().range.is_empty());
    assert_eq!(properties[1].kind, Kind::Property { additive: true });
    assert_eq!(
        Source::new(buffer)
            .text(&properties[1].content)
            .unwrap()
            .to_string(),
        "界 e\u{301}"
    );
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.kind == Kind::Planning)
            .count(),
        1
    );
}

// Observation preserves BOM, CRLF, unknown syntax and a missing final newline.
// Native edits and undo invalidate old ranges instead of rebasing them silently.
#[test]
fn source_fidelity() {
    let original = "\u{feff}* 界 e\u{301}\r\n#+FUTURE: unchanged\r\n:LOGBOOK:\r\nunknown\r\n:END:";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .load(Loaded::read(original.as_bytes()).unwrap())
        .unwrap();
    let foreign = buffers.create("other").unwrap();
    let buffer = buffers.get(id).unwrap();
    let structure = parse(buffer);
    assert_eq!(super::bytes(buffer), original.as_bytes());
    assert!(structure.node(buffers.get(foreign).unwrap(), 0).is_err());
    let old = structure.node(buffer, 0).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    super::incremental::replace(buffer, 2..5, "New");
    assert!(structure.node(buffer, 0).is_err());
    assert!(Source::new(buffer).text(&old.source).is_err());
    buffer
        .navigate(rev::buffer::edit::History::Undo, &mut [], None)
        .unwrap();
    assert_eq!(super::bytes(buffer), original.as_bytes());
    assert!(structure.node(buffer, 0).is_err());
    assert_eq!(parse(buffer).coverage(), Coverage::Complete);
}

// Clipped UTF-8 lines, exhausted nodes and deep owners report incomplete
// observations. None of their ranges certify a complete semantic edit target.
#[test]
fn observation_limits() {
    let cases = [
        (
            "bytes",
            "* H\n界 body\n* Next\n",
            Limits::new(6, 100, 20).unwrap(),
            Limit::AnalysisBytes,
        ),
        (
            "nodes",
            "* H\n| a | b |\n",
            Limits::new(100, 3, 20).unwrap(),
            Limit::SourceNodes,
        ),
        (
            "depth",
            "* H\n** Child\nbody\n",
            Limits::new(100, 100, 1).unwrap(),
            Limit::SourceDepth,
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, limits, limit) in cases {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let structure = StructureScan::new(buffer, limits)
            .unwrap()
            .run(&Default::default());
        assert_eq!(structure.coverage(), Coverage::Incomplete(limit), "{name}");
        for node in nodes(buffer, &structure) {
            assert!(!node.complete, "{name}");
            assert!(Source::new(buffer).text(&node.source).is_ok(), "{name}");
        }
    }
}

// Structural snapshots use the same single owned worker and stale-result check
// as inline analysis. A cancelled large scan cannot publish or mutate source.
#[test]
fn owned_observations() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .create(&"* H\n#+begin_src text\n,* body\n#+end_src\n".repeat(10000))
        .unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut job = Job::default();
    let scan = StructureScan::new(buffer, Limits::default()).unwrap();
    job.start(buffer.stamp(), move |cancel| scan.run(cancel))
        .unwrap();
    super::incremental::replace(buffer, 0..0, "changed\n");
    let deadline = Instant::now() + Duration::from_secs(2);
    let outcome = loop {
        if let Some(outcome) = job.poll(buffer) {
            break outcome;
        }
        assert!(Instant::now() < deadline, "structural worker completed");
        std::thread::yield_now();
    };
    assert!(matches!(outcome, Err(JobError::Source(_))));
    let cancellation = Cancellation::default();
    cancellation.cancel();
    let scan = StructureScan::new(buffer, Limits::default()).unwrap();
    assert_eq!(scan.run(&cancellation).coverage(), Coverage::Cancelled);
    assert!(super::bytes(buffer).starts_with(b"changed\n* H\n"));
}

// Literal bodies do not spend the cell budget on apparent tables. Outside the
// block, dense cells reach the shared node cap without unbounded allocation.
#[test]
fn long_lines() {
    let cases = [
        (
            "literal cells",
            format!("#+begin_example\n{}\n#+end_example\n", "|".repeat(50000)),
            Coverage::Complete,
        ),
        (
            "real cells",
            format!("{}\n", "|".repeat(50000)),
            Coverage::Incomplete(Limit::SourceNodes),
        ),
        (
            "star text",
            format!("{}\n", "*".repeat(50000)),
            Coverage::Complete,
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, expected) in cases {
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let structure = StructureScan::new(buffer, Limits::new(100000, 32, 20).unwrap())
            .unwrap()
            .run(&Default::default());
        assert_eq!(structure.coverage(), expected, "{name}");
        assert_eq!(super::bytes(buffer), source.as_bytes(), "{name}");
        assert!(
            !nodes(buffer, &structure)
                .iter()
                .any(|node| matches!(node.kind, Kind::List { .. })),
            "{name}"
        );
    }
}

// A caption can belong to a paragraph as well as a table. Its own ranges are
// separate from inline content, and blank cells remain valid empty locations.
#[test]
fn affiliation_ranges() {
    let source = "#+CAPTION: text\nparagraph\n\n| | value |\n|  \n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let structure = parse(buffer);
    let nodes = nodes(buffer, &structure);
    let paragraph = nodes
        .iter()
        .find(|node| node.kind == Kind::Paragraph)
        .unwrap();
    assert!(paragraph.affiliated.is_some());
    assert_eq!(
        Source::new(buffer)
            .text(&paragraph.content)
            .unwrap()
            .to_string(),
        "paragraph\n"
    );
    let cells: Vec<_> = nodes
        .iter()
        .filter(|node| node.kind == Kind::TableCell)
        .map(|node| text(buffer, node))
        .collect();
    assert_eq!(cells, ["", "value", ""]);
}
