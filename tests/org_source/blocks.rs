use rev::buffer::Buffers;
use rev::org::{
    BlockKind, Coverage, InlineContext, InlineScan, Limits, Source, StructureKind as Kind,
};

use super::structure::{nodes, parse, text};

// Comma-quoted apparent headings/delimiters and markup inside literal blocks
// remain original body text. Their structural owner supplies a literal context.
#[test]
fn literal_regions() {
    let cases = [
        (
            "source",
            "SrC rust :results output",
            "sRc",
            BlockKind::Source,
        ),
        ("example", "example", "example", BlockKind::Example),
        ("export", "export html", "export", BlockKind::Export),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, opening, closing, kind) in cases {
        let body =
            ",* apparent\n,#+end_src\n*bold* [[https://example.test]]\n:PROPERTIES:\n| a |\n";
        let source =
            format!("* H\n#+NAME: block\n  #+BEGIN_{opening}\n{body}  #+END_{closing}\n** Child\n");
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let structure = parse(buffer);
        let nodes = nodes(buffer, &structure);
        assert_eq!(structure.coverage(), Coverage::Complete, "{name}");
        let block = nodes
            .iter()
            .find(|node| node.kind == Kind::Block(kind))
            .unwrap();
        assert_eq!(block.context, Some(InlineContext::Literal), "{name}");
        assert_eq!(
            Source::new(buffer)
                .text(&block.content)
                .unwrap()
                .to_string(),
            body,
            "{name}"
        );
        assert!(
            block.opening.is_some() && block.closing.is_some() && block.affiliated.is_some(),
            "{name}"
        );
        let inline = InlineScan::new(
            buffer,
            &block.content,
            block.context.unwrap(),
            Limits::default(),
        )
        .unwrap()
        .run(&Default::default());
        assert!(inline.is_empty(), "{name}");
        assert_eq!(
            nodes
                .iter()
                .filter(|node| matches!(node.kind, Kind::Heading { .. }))
                .count(),
            2,
            "{name}"
        );
        assert!(!nodes.iter().any(|node| node.kind == Kind::Table), "{name}");
        assert_eq!(super::bytes(buffer), source.as_bytes(), "{name}");
    }
}

// Raw headings are context-free, and a mismatched or suffixed closing line
// cannot validate a block. Recovery must still find the following real subtree.
#[test]
fn delimiter_recovery() {
    let cases = [
        ("raw heading", "#+begin_src rust\n* Real\n#+end_src\n"),
        ("mismatch", "#+begin_example\nbody\n#+end_src\n* Real\n"),
        (
            "trailing text",
            "#+begin_export html\nbody\n#+end_export extra\n* Real\n",
        ),
        ("missing close", "#+begin_src rust\nbody\n* Real\n"),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source) in cases {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let structure = parse(buffer);
        let nodes = nodes(buffer, &structure);
        assert!(
            !nodes.iter().any(|node| matches!(node.kind, Kind::Block(_))),
            "{name}"
        );
        let malformed = nodes
            .iter()
            .find(|node| node.kind == Kind::Malformed)
            .unwrap();
        assert!(!malformed.complete, "{name}");
        let real = nodes
            .iter()
            .find(|node| matches!(node.kind, Kind::Heading { .. }))
            .unwrap();
        assert!(text(buffer, real).starts_with("* Real\n"), "{name}");
    }
}

// Literal element bodies can be unindented inside an indented list item.
// Greater blocks parse structural children; verse retains its inline context.
#[test]
fn enclosing_contexts() {
    let source = "- first\n  #+begin_src text\nunindented\n,* apparent\n  #+end_src\n- second\n\n#+begin_quote\n| a |\n#+end_quote\n#+begin_verse\n*bold*\n#+end_verse\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let structure = parse(buffer);
    let nodes = nodes(buffer, &structure);
    let first = nodes
        .iter()
        .find(|node| matches!(node.kind, Kind::Item { .. }))
        .unwrap();
    assert!(text(buffer, first).contains("unindented\n,* apparent\n  #+end_src\n"));
    let (quote, _) = nodes
        .iter()
        .enumerate()
        .find(|(_, node)| node.kind == Kind::Block(BlockKind::Greater))
        .unwrap();
    assert!(
        nodes
            .iter()
            .any(|node| node.kind == Kind::Table && node.parent == Some(quote))
    );
    let verse = nodes
        .iter()
        .find(|node| node.kind == Kind::Block(BlockKind::Verse))
        .unwrap();
    assert_eq!(verse.context, Some(InlineContext::Standard));
}
