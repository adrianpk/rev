use rev::org::{
    Cancellation,
    formula::{Coordinate as C, Expression as E, Function as F, Limits, Operator as O, Program},
};
fn parsed(text: &str) -> Program {
    Program::parse(text, Limits::default(), &Cancellation::default()).unwrap()
}
// The public syntax owner produces real typed field/column assignments and
// source spans, without evaluating a value or needing editor/runtime services.
#[test]
fn formula_assignments() {
    let text = "$4 = $2*$3 :: @2$3 = round(abs(@-1$+2), 2);f4";
    let p = parsed(text);
    assert_eq!(p.assignments().len(), 2);
    let a = &p.assignments()[0];
    assert_eq!(a.target.column.as_ref().unwrap().coordinate, C::Absolute(4));
    assert_eq!(&text[a.source.clone()], "$4 = $2*$3");
    assert!(matches!(
        p.nodes()[a.expression].expression,
        E::Binary(O::Multiply, _, _)
    ));
    let a = &p.assignments()[1];
    assert_eq!(a.target.row.as_ref().unwrap().coordinate, C::Absolute(2));
    assert_eq!(a.decimals, Some(4));
    assert!(matches!(
        p.nodes()[a.expression].expression,
        E::Call(F::Round, _)
    ));
    let relative = p
        .nodes()
        .iter()
        .find_map(|n| match &n.expression {
            E::Field(r) if r.row.is_some() => Some(r),
            _ => None,
        })
        .unwrap();
    assert_eq!(relative.row.as_ref().unwrap().coordinate, C::Relative(-1));
    assert_eq!(relative.column.as_ref().unwrap().coordinate, C::Relative(2));
}
// Omitted and explicit current coordinates stay distinct from signed offsets.
// Inclusive ranges retain both original endpoint forms for later resolution.
#[test]
fn formula_coordinates() {
    for (name, text, row, column) in [
        ("current column", "$2=$0", None, Some(C::Current)),
        ("current row", "$2=@0", Some(C::Current), None),
        (
            "relative zero",
            "$2=@+0$-0",
            Some(C::Relative(0)),
            Some(C::Relative(0)),
        ),
        (
            "absolute",
            "$2=@03$01",
            Some(C::Absolute(3)),
            Some(C::Absolute(1)),
        ),
    ] {
        let p = parsed(text);
        let E::Field(r) = &p.nodes()[0].expression else {
            panic!("{name}")
        };
        assert_eq!(r.row.as_ref().map(|c| c.coordinate), row, "{name}");
        assert_eq!(r.column.as_ref().map(|c| c.coordinate), column, "{name}");
        assert_eq!(&text[r.source.clone()], &text[3..]);
    }
    let text = "$3=vsum(@-1$-2..@-1)";
    let p = parsed(text);
    let E::Range(a, b) = &p.nodes()[0].expression else {
        panic!("range")
    };
    assert_eq!(&text[a.source.clone()], "@-1$-2");
    assert_eq!(&text[b.source.clone()], "@-1");
}
// All declared aggregate aliases and scalar functions have explicit typed
// identities; unsupported names never become a callback or an expression string.
#[test]
fn formula_functions() {
    for (name, function) in [
        ("vsum", F::Sum),
        ("sum", F::Sum),
        ("vmean", F::Mean),
        ("mean", F::Mean),
        ("vmin", F::Minimum),
        ("min", F::Minimum),
        ("vmax", F::Maximum),
        ("max", F::Maximum),
        ("round", F::Round),
        ("abs", F::Abs),
    ] {
        let p = parsed(&format!("$2={name}($1)"));
        let E::Call(actual, arguments) = &p.nodes()[p.assignments()[0].expression].expression
        else {
            panic!("{name}")
        };
        assert_eq!(*actual, function);
        assert_eq!(arguments.len(), 1);
    }
}
