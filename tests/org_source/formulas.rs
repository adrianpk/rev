use super::*;
use rev::org::{
    Cancellation, StructureKind,
    formula::{ErrorKind, Limits, Program},
};

// Parsing actual canonical formula observations preserves supported, unknown and
// malformed source identically, including BOM/CRLF/EOF and read-only authority.
#[test]
fn formula_source_fidelity() {
    for (name, value, expected) in [
        ("native", "$2=round(@-1$+1,2);%.2f", None),
        ("Lisp", "$2='(identity $1)", Some(ErrorKind::Unsupported)),
        ("Calc", "$2=sin($1);Dp20", Some(ErrorKind::Unsupported)),
        ("malformed", "$2=@+", Some(ErrorKind::Syntax)),
        ("Unicode", "$2=界", Some(ErrorKind::Unsupported)),
    ] {
        let original = format!(
            "\u{feff}Before 界.\r\n#+NAME: stable\r\n|A|B|\r\n|1|2|\r\n#+TBLFM: {value}\r\n\r\nEOF"
        );
        let mut buffers = Buffers::new().unwrap();
        let id = buffers
            .load(Loaded::read(original.as_bytes()).unwrap())
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(true);
        let before = (buffer.stamp(), buffer.state());
        let observed = super::structure::parse(buffer);
        let nodes = super::structure::nodes(buffer, &observed);
        let node = nodes
            .iter()
            .find(|n| n.kind == StructureKind::TableFormula)
            .unwrap();
        let source = Source::new(buffer);
        let text = source.text(&node.content).unwrap().to_string();
        assert_eq!(text, value, "{name}");
        let result = Program::parse(&text, Limits::default(), &Cancellation::default());
        assert_eq!(result.as_ref().err().map(|e| e.kind), expected, "{name}");
        assert_eq!(bytes(buffer), original.as_bytes(), "{name}");
        assert_eq!((buffer.stamp(), buffer.state()), before, "{name}");
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}

fn numerical_source(
    buffer: &Buffer,
    observed: &rev::org::Structure,
) -> (rev::org::formula::eval::Grid, String) {
    let nodes = super::structure::nodes(buffer, observed);
    let source = Source::new(buffer);
    let rows = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == StructureKind::TableRow { separator: false })
        .map(|(id, _)| {
            nodes
                .iter()
                .filter(|n| n.kind == StructureKind::TableCell && n.parent == Some(id))
                .map(|n| source.text(&n.content).unwrap().to_string())
                .collect()
        })
        .collect();
    let formula = nodes
        .iter()
        .find(|n| n.kind == StructureKind::TableFormula)
        .unwrap();
    (
        rev::org::formula::eval::Grid::new(rows, 1).unwrap(),
        source.text(&formula.content).unwrap().to_string(),
    )
}

// Evaluate real source-owned fields and formula text from a read-only canonical
// BOM/CRLF document. Both success and late failure leave bytes and history intact.
#[test]
fn evaluation_source_fidelity() {
    use rev::org::formula::eval::{ErrorKind as NumericError, Policy};
    for (value, expected) in [
        ("$2=$1*2", None),
        ("$2=$1/(3-$1)", Some(NumericError::DivisionByZero)),
    ] {
        let original = format!(
            "\u{feff}Before 界.\r\n#+NAME: stable\r\n|Input|Result|\r\n|-----+------|\r\n|2|old|\r\n|3|old|\r\n#+TBLFM: {value}\r\n\r\nEOF"
        );
        let mut buffers = Buffers::new().unwrap();
        let id = buffers
            .load(Loaded::read(original.as_bytes()).unwrap())
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(true);
        let before = (buffer.stamp(), buffer.state());
        let observed = super::structure::parse(buffer);
        let (data, formula) = numerical_source(buffer, &observed);
        assert_eq!(formula, value);
        let result = Program::parse(&formula, Limits::default(), &Cancellation::default())
            .unwrap()
            .evaluate(&data, Policy::default(), &Cancellation::default());
        assert_eq!(result.as_ref().err().map(|e| e.kind), expected);
        if let Ok(results) = result {
            assert_eq!(
                results.iter().map(|r| r.value).collect::<Vec<_>>(),
                [4.0, 6.0]
            );
        }
        assert_eq!(bytes(buffer), original.as_bytes());
        assert_eq!((buffer.stamp(), buffer.state()), before);
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}
