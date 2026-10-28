use rev::buffer::{Buffer, Buffers};
use rev::org::{
    Cancellation, Coverage, Limits, Source, Structure, StructureKind as Kind, StructureNode,
    StructureScan,
};

pub(super) fn parse(buffer: &Buffer) -> Structure {
    StructureScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default())
}

pub(super) fn nodes(buffer: &Buffer, structure: &Structure) -> Vec<StructureNode> {
    (0..structure.len())
        .map(|index| structure.node(buffer, index).unwrap())
        .collect()
}

pub(super) fn text(buffer: &Buffer, node: &StructureNode) -> String {
    Source::new(buffer).text(&node.source).unwrap().to_string()
}

// A heading owns its complete descendants, but its section stops before its
// first child. Blank lines before a section and lexical star candidates differ.
#[test]
fn subtree_ranges() {
    let source = "intro\n* Parent\n\nbody\n** Child\nchild body\n* Peer\n*\ttext\n*\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let structure = parse(buffer);
    assert_eq!(structure.coverage(), Coverage::Complete);
    let nodes = nodes(buffer, &structure);
    let headings: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| matches!(node.kind, Kind::Heading { .. }))
        .collect();
    assert_eq!(headings.len(), 3);
    let cases = [
        (
            "parent",
            0,
            "* Parent\n\nbody\n** Child\nchild body\n",
            None,
        ),
        ("child", 1, "** Child\nchild body\n", Some(headings[0].0)),
        ("peer", 2, "* Peer\n*\ttext\n*\n", None),
    ];
    for (name, index, expected, parent) in cases {
        assert_eq!(text(buffer, headings[index].1), expected, "{name}");
        assert_eq!(headings[index].1.parent, parent, "{name}");
    }
    let sections: Vec<_> = nodes
        .iter()
        .filter(|node| node.kind == Kind::Section)
        .map(|node| text(buffer, node))
        .collect();
    assert_eq!(
        sections,
        ["intro\n", "body\n", "child body\n", "*\ttext\n*\n"]
    );
}

// Property classification depends on heading/planning adjacency or the zeroth
// section. The same spelling later in a section is an ordinary drawer.
#[test]
fn metadata_contexts() {
    let cases = [
        (
            "heading",
            "* H\n:PROPERTIES:\n:ID: x\n:X+: more\n:END:\n",
            true,
            2,
        ),
        (
            "planning",
            "* H\nSCHEDULED: <2026-10-28 Wed>\n:PROPERTIES:\n:X: v\n:END:\n",
            true,
            1,
        ),
        (
            "zeroth",
            "\n# comment\n:PROPERTIES:\n:X: v\n:END:\n",
            true,
            1,
        ),
        ("blank", "* H\n\n:PROPERTIES:\n:X: v\n:END:\n", false, 0),
        (
            "ordinary",
            "* H\nbody\n:PROPERTIES:\n:X: v\n:END:\n",
            false,
            0,
        ),
        (
            "invalid body",
            "* H\n:PROPERTIES:\nnot a property\n:END:\n",
            false,
            0,
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, properties, count) in cases {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let structure = parse(buffer);
        let nodes = nodes(buffer, &structure);
        assert_eq!(
            nodes.iter().any(|node| node.kind == Kind::PropertyDrawer),
            properties,
            "{name}"
        );
        assert_eq!(
            nodes
                .iter()
                .filter(|node| matches!(node.kind, Kind::Property { .. }))
                .count(),
            count,
            "{name}"
        );
    }
}

// A list retains continuation paragraphs and nested items. A following
// unindented paragraph is outside it; tabs use Org's eight-column indentation.
#[test]
fn list_ownership() {
    let source =
        "- [ ] first\n  continuation\n\n  paragraph\n  + child\n\t- deep\n- second\n\n\noutside\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let structure = parse(buffer);
    let nodes = nodes(buffer, &structure);
    let lists: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.kind, Kind::List { .. }))
        .collect();
    assert_eq!(lists.len(), 3);
    assert_eq!(
        text(buffer, lists[0]),
        source.strip_suffix("outside\n").unwrap()
    );
    let items: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.kind, Kind::Item { .. }))
        .collect();
    assert_eq!(items.len(), 4);
    assert!(text(buffer, items[0]).contains("paragraph\n  + child\n\t- deep\n"));
    assert!(matches!(items[2].kind, Kind::Item { indent: 8 }));
    let starts: Vec<_> = nodes
        .iter()
        .map(|node| node.source.span().range.start)
        .collect();
    assert!(starts.windows(2).all(|pair| pair[0] <= pair[1]));
}

// Table ownership includes directly affiliated metadata and formulas. Cell
// ranges retain Unicode spelling; a blank line breaks caption affiliation.
#[test]
fn table_metadata() {
    let source = "#+CAPTION: cap\n#+NAME: grid\n  | 界 | *bold* |\n  |---+---|\n  | a | b |\n#+TBLFM: $1=$2\n\n#+CAPTION: detached\n\ntext\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let structure = parse(buffer);
    let nodes = nodes(buffer, &structure);
    let table = nodes.iter().find(|node| node.kind == Kind::Table).unwrap();
    assert_eq!(
        text(buffer, table),
        source.split("#+CAPTION: detached").next().unwrap()
    );
    let source = Source::new(buffer);
    assert_eq!(
        source
            .text(table.affiliated.as_ref().unwrap())
            .unwrap()
            .to_string(),
        "#+CAPTION: cap\n#+NAME: grid\n"
    );
    let cells: Vec<_> = nodes
        .iter()
        .filter(|node| node.kind == Kind::TableCell)
        .map(|node| text(buffer, node))
        .collect();
    assert_eq!(cells, ["界", "*bold*", "a", "b"]);
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.kind == Kind::TableFormula)
            .count(),
        1
    );
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.kind == Kind::AffiliatedKeyword)
            .count(),
        2
    );
    assert!(
        nodes
            .iter()
            .any(|node| node.kind == Kind::Keyword && text(buffer, node).contains("detached"))
    );
}
