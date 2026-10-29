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
