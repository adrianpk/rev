use super::*;
use rev::org::{
    Cancellation,
    formula::{ErrorKind as K, Limits, Program},
};

// Diagnostics identify exact input bytes, including EOF, multibyte unsupported
// tokens, incomplete numbers and whole bad coordinates rather than guessed cells.
#[test]
fn formula_diagnostics() {
    for (name, text, kind, source) in [
        ("empty", "", K::Syntax, 0..0),
        ("missing value", "$2=", K::Syntax, 3..3),
        ("missing coordinate", "$2=@+", K::Syntax, 5..5),
        ("missing operand", "$2=1+", K::Syntax, 5..5),
        ("missing close", "$2=($1", K::Syntax, 6..6),
        ("arity", "$2=abs(1,2)", K::Syntax, 3..11),
        ("remote", "$2=remote(name,$1)", K::Unsupported, 3..9),
        ("advanced function", "$2=sin($1)", K::Unsupported, 3..6),
        ("named", "$2=$name", K::Unsupported, 4..5),
        ("hline", "$2=@I$1", K::Unsupported, 4..5),
        ("Lisp", "$2='(identity $1)", K::Unsupported, 3..4),
        ("Unicode", "$2=界", K::Unsupported, 3..6),
        ("Calc flags", "$2=1;EN", K::Unsupported, 5..7),
        ("decimal", "$2=1.2.3", K::Syntax, 6..8),
        ("exponent", "$2=1e+", K::Syntax, 3..6),
        ("nonfinite", "$2=1e999", K::Syntax, 3..8),
        ("empty assignment", "$2=1::", K::Syntax, 6..6),
        ("format sign", "$2=1;f+2", K::Syntax, 5..8),
    ] {
        let error = Program::parse(text, Limits::default(), &Cancellation::default()).unwrap_err();
        assert_eq!(error.kind, kind, "{name}: {error}");
        assert_eq!(error.source, source, "{name}: {error}");
        assert!(text.is_char_boundary(source.start) && text.is_char_boundary(source.end));
    }
    let text = format!("$2=${}0", usize::MAX);
    let error = Program::parse(&text, Limits::default(), &Cancellation::default()).unwrap_err();
    assert_eq!(error.kind, K::Syntax);
    assert_eq!(error.source, 4..text.len());
}
// Caller-supplied syntax limits cannot remove the compiled stack/work ceilings.
// The maximum admitted recursion depth remains executable without recursive drop.
#[test]
fn formula_limit_admission() {
    for limits in [
        Limits {
            bytes: 0,
            ..Limits::default()
        },
        Limits {
            bytes: 33554433,
            ..Limits::default()
        },
        Limits {
            steps: 0,
            ..Limits::default()
        },
        Limits {
            steps: 10000001,
            ..Limits::default()
        },
        Limits {
            depth: 0,
            ..Limits::default()
        },
        Limits {
            depth: 257,
            ..Limits::default()
        },
        Limits {
            depth: usize::MAX,
            ..Limits::default()
        },
        Limits {
            decimal_places: 16,
            ..Limits::default()
        },
    ] {
        let error = Program::parse("$2=1", limits, &Cancellation::default()).unwrap_err();
        assert_eq!(error.kind, K::Limit);
    }
    let text = format!("$2={}1{}", "(".repeat(256), ")".repeat(256));
    assert!(
        Program::parse(
            &text,
            Limits {
                depth: 256,
                ..Limits::default()
            },
            &Cancellation::default()
        )
        .is_ok()
    );
}
// Shared syntax drives actual reference publication. Formatting, constants and
// affiliated bytes remain unchanged while only real coordinate digits move.
#[test]
fn formatted_reference_edit() {
    let source = "#+NAME: stable\r\n|a|b|c|\r\n|1|2|3|\r\n#+TBLFM: $3=abs($1)+round($2,1);%.2f";
    let (_f, mut s) = editor("", source);
    at(&mut s, source.find("a|b").unwrap());
    action(&mut s, C::TableMoveColumnRight);
    let after = text(&s);
    assert!(
        after.ends_with("#+TBLFM: $3=abs($2)+round($1,1);%.2f"),
        "{}",
        s.echo()
    );
    assert_eq!(
        super::metadata::values(&s),
        vec!["b", "a", "c", "2", "1", "3"]
    );
    super::structure::roundtrip(&mut s, source, &after);
}
