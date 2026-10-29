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
fn shape(program: &Program, id: usize) -> String {
    match &program.nodes()[id].expression {
        E::Number(number) => number.to_string(),
        E::Unary(operator, child) => format!("({operator:?} {})", shape(program, *child)),
        E::Binary(operator, left, right) => format!(
            "({operator:?} {} {})",
            shape(program, *left),
            shape(program, *right)
        ),
        E::Group(child) => format!("(Group {})", shape(program, *child)),
        _ => panic!("arithmetic fixture"),
    }
}
// Independent expression trees establish right-associative powers, unary signs
// below powers, left-associative subtraction/division and Org's * above / rule.
#[test]
fn formula_precedence() {
    for (name, expression, expected) in [
        ("product", "2+3*4", "(Add 2 (Multiply 3 4))"),
        ("Org quotient", "8/2*2", "(Divide 8 (Multiply 2 2))"),
        (
            "product then quotient",
            "8*2/2",
            "(Divide (Multiply 8 2) 2)",
        ),
        ("quotient chain", "8/2/2", "(Divide (Divide 8 2) 2)"),
        ("difference chain", "8-2-1", "(Subtract (Subtract 8 2) 1)"),
        ("power chain", "2^3^2", "(Power 2 (Power 3 2))"),
        ("negative power", "-2^2", "(Subtract (Power 2 2))"),
        ("negative base", "(-2)^2", "(Power (Group (Subtract 2)) 2)"),
        ("negative exponent", "2^-3", "(Power 2 (Subtract 3))"),
        ("unary chain", "-+2", "(Subtract (Add 2))"),
        ("decimal exponent", ".5+1.2e-3", "(Add 0.5 0.0012)"),
    ] {
        let p = parsed(&format!("$1={expression}"));
        assert_eq!(shape(&p, p.assignments()[0].expression), expected, "{name}");
    }
}
// Fixed formatting is data in either compatible spelling, with exact source
// retained. Bounds reject the complete parse before any evaluation can occur.
#[test]
fn formula_formats() {
    for (name, spelling, places) in [
        ("Calc", "f4", 4),
        ("printf", "%.2f", 2),
        ("zero", "%.0f", 0),
        ("maximum", "f12", 12),
    ] {
        let text = format!("$2 = $1 ;{spelling} :: $3=abs(-1)");
        let p = parsed(&text);
        assert_eq!(p.assignments()[0].decimals, Some(places), "{name}");
        assert_eq!(p.assignments()[1].decimals, None);
        assert_eq!(&text[p.assignments()[0].target.source.clone()], "$2");
    }
    let e = Program::parse("$2=$1;f13", Limits::default(), &Cancellation::default()).unwrap_err();
    assert_eq!(e.kind, rev::org::formula::ErrorKind::Limit);
    let p = Program::parse(
        "$2=$1;%.15f",
        Limits {
            decimal_places: 15,
            ..Limits::default()
        },
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(p.assignments()[0].decimals, Some(15));
}
// Real parsing respects depth, input/work budgets and cancellation/deadline
// tokens, including pathological unary/power chains without recursive AST drop.
#[test]
fn formula_bounds() {
    use rev::org::formula::ErrorKind as K;
    let token = Cancellation::default();
    let text = format!("$2={}1{}", "(".repeat(64), ")".repeat(64));
    assert!(Program::parse(&text, Limits::default(), &token).is_ok());
    let cases = [
        (
            "parentheses",
            format!("$2={}1{}", "(".repeat(65), ")".repeat(65)),
            Limits::default(),
        ),
        (
            "powers",
            format!("$2={}1", "1^".repeat(65)),
            Limits::default(),
        ),
        (
            "unary",
            format!("$2={}1", "-".repeat(65)),
            Limits::default(),
        ),
        (
            "tokens",
            format!("$2={}1", "1+".repeat(50000)),
            Limits::default(),
        ),
        (
            "bytes",
            "$2=$1".into(),
            Limits {
                bytes: 4,
                ..Limits::default()
            },
        ),
        (
            "steps",
            "$2=$1".into(),
            Limits {
                steps: 3,
                ..Limits::default()
            },
        ),
    ];
    for (name, text, limits) in cases {
        let e = Program::parse(&text, limits, &token).unwrap_err();
        assert_eq!(e.kind, K::Limit, "{name}: {e}");
        assert!(text.is_char_boundary(e.source.start) && text.is_char_boundary(e.source.end));
    }
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        Program::parse("$2=$1", Limits::default(), &cancelled)
            .unwrap_err()
            .kind,
        K::Cancelled
    );
}
