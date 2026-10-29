use super::*;
use rev::org::{Cancellation, Coverage, Limits, Source, StructureKind, StructureScan};
fn values(s: &Session) -> Vec<String> {
    let b = s.buffer().unwrap();
    let observed = StructureScan::new(b, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    assert_eq!(observed.coverage(), Coverage::Complete);
    (0..observed.len())
        .filter_map(|i| {
            let n = observed.node(b, i).unwrap();
            (n.kind == StructureKind::TableCell)
                .then(|| Source::new(b).text(&n.content).unwrap().to_string())
        })
        .collect()
}
// Names, captions, attributes, formula spelling and surrounding paragraphs are
// outside the alignment replacements. Formula-changing insertions reject as a unit.
#[test]
fn affiliated_formula_source() {
    let source = "Before.\r\n#+NAME: sample\r\n#+CAPTION: Keep *markup*\r\n#+ATTR_HTML: :class keep\r\n  |item|qty|\r\n  |--+--|\r\n  |very long|02|\r\n  #+TBLFM: $2=$1+1::weird=unknown()\r\n\r\nAfter.\r\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, source.find("|item").unwrap());
    let before = values(&s);
    action(&mut s, C::TableAlign);
    let expected = source.replace(
        "  |item|qty|\r\n  |--+--|\r\n  |very long|02|",
        "  | item      | qty |\r\n  |-----------+-----|\r\n  | very long |  02 |",
    );
    assert_eq!(text(&s), expected, "{}", s.echo());
    assert_eq!(values(&s), before);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    let view = s.view().clone();
    for command in [C::TableInsertRow, C::TableInsertHline, C::MetaReturn] {
        action(&mut s, command);
        assert!(s.echo().contains("reference rewriting"), "{}", s.echo());
        assert_eq!(text(&s), expected);
        assert_eq!(s.view(), &view);
        assert_eq!(s.buffer().unwrap().history().len(), 1);
    }
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
}
// Ragged rows gain explicit empty fields while original values and literal
// markup/entity spellings remain unchanged; hlines expand to the actual columns.
#[test]
fn ragged_cell_source() {
    let source = "|A||C\n|-+-|\n|B|\n";
    let (_f, mut s) = editor("", source);
    action(&mut s, C::TableAlign);
    assert_eq!(
        text(&s),
        "| A |   | C |\n|---+---+---|\n| B |   |   |\n",
        "{}",
        s.echo()
    );
    assert_eq!(values(&s), vec!["A", "", "C", "B", "", ""]);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    let source = "|*bold*|\\vert{}|{{{unknown(x)}}}|\n";
    let (_f, mut s) = editor("", source);
    let before = values(&s);
    action(&mut s, C::TableAlign);
    assert_eq!(values(&s), before, "{}", s.echo());
}
// Alignment controls stay literal cell source and use actual source width;
// numeric alignment is unaffected by a width-only formatting row.
#[test]
fn alignment_cookies() {
    let source = "|<l>|<r>|<c>|\n|A|2|xy|\n|BBB|10|Q|\n";
    let (_f, mut s) = editor("", source);
    let before = values(&s);
    action(&mut s, C::TableAlign);
    assert_eq!(
        text(&s),
        "| <l> | <r> | <c> |\n| A   |   2 | xy  |\n| BBB |  10 |  Q  |\n",
        "{}",
        s.echo()
    );
    assert_eq!(values(&s), before);
    let (_f, mut s) = editor("", "|Count|\n|<10>|\n|2|\n");
    action(&mut s, C::TableAlign);
    assert_eq!(
        text(&s),
        "| Count |\n|  <10> |\n|     2 |\n",
        "{}",
        s.echo()
    );
}
// Creating on a blank line between existing tables retains separate table
// identities and all surrounding source instead of joining their row streams.
#[test]
fn creation_table_boundaries() {
    let source = "| old |\n\n| next |\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 8);
    action(&mut s, C::TableCreate);
    argument(&mut s, "1x1");
    assert_eq!(
        text(&s),
        "| old |\n\n|   |\n|---|\n\n| next |\n",
        "{}",
        s.echo()
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
}
// Literal and unsupported block bodies do not become editable outer tables;
// quotes retain native table interpretation and unknown cell text stays literal.
#[test]
fn table_contexts() {
    for block in ["src text", "example", "comment", "mystery"] {
        let closing = block.split_whitespace().next().unwrap();
        let source = format!("#+begin_{block}\n|A|B|\n#+end_{closing}\n");
        let (_f, mut s) = editor("", &source);
        at(&mut s, source.find("|A").unwrap());
        let before = s.view().clone();
        action(&mut s, C::TableAlign);
        assert_eq!(text(&s), source, "{block}: {}", s.echo());
        assert_eq!(s.view(), &before);
        assert!(s.buffer().unwrap().history().is_empty());
    }
    let source = "#+begin_quote\n|A|B|\n#+end_quote\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, source.find("|A").unwrap());
    action(&mut s, C::TableAlign);
    assert_eq!(
        text(&s),
        source.replace("|A|B|", "| A | B |"),
        "{}",
        s.echo()
    );
}
