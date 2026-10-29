use super::evaluation::grid;
use rev::org::{
    Cancellation,
    formula::{
        Limits, Program,
        eval::{Cell, Error, ErrorKind, Grid, Policy},
    },
};
fn failure(data: &Grid, formula: &str, policy: Policy) -> Error {
    Program::parse(formula, Limits::default(), &Cancellation::default())
        .unwrap()
        .evaluate(data, policy, &Cancellation::default())
        .unwrap_err()
}
// Invalid dependencies and numeric operations reject the entire result set,
// even after another target could have produced a valid value.
#[test]
fn evaluation_rejections() {
    let data = grid(&[&["text", "old", "old"]], 0);
    for (name, formula, expected) in [
        ("self cycle", "$2=$2", ErrorKind::Cycle),
        ("mutual cycle", "$2=$3::$3=$2", ErrorKind::Cycle),
        ("range cycle", "$2=vsum($1..$2)", ErrorKind::Cycle),
        ("outside field", "$2=$4", ErrorKind::Reference),
        ("negative row", "$2=@-1$1", ErrorKind::Reference),
        ("positive row", "$2=@+1$1", ErrorKind::Reference),
        (
            "relative overflow",
            "$2=$+9223372036854775807",
            ErrorKind::Reference,
        ),
        ("outside target", "@2$2=1", ErrorKind::Reference),
        ("text scalar", "$2=$1+1", ErrorKind::Numeric),
        ("range scalar", "$2=($1..$1)+1", ErrorKind::Numeric),
        ("text abs", "$2=abs($1)", ErrorKind::Numeric),
        ("zero division", "$2=1/0", ErrorKind::DivisionByZero),
        ("negative zero", "$2=1/(-0)", ErrorKind::DivisionByZero),
        ("overflow", "$2=1e308*2", ErrorKind::NonFinite),
        ("invalid power", "$2=(-1)^0.5", ErrorKind::NonFinite),
        (
            "fractional precision",
            "$2=round(2,0.5)",
            ErrorKind::Numeric,
        ),
        ("precision ceiling", "$2=round(2,13)", ErrorKind::Numeric),
        ("empty mean", "$2=vmean($1)", ErrorKind::EmptyAggregate),
        ("empty minimum", "$2=vmin($1)", ErrorKind::EmptyAggregate),
        ("empty maximum", "$2=vmax($1)", ErrorKind::EmptyAggregate),
        ("late failure", "$2=2::$3=1/0", ErrorKind::DivisionByZero),
    ] {
        let error = failure(&data, formula, Policy::default());
        assert_eq!(error.kind, expected, "{name}: {error}");
        assert_eq!(data.cell(Cell { row: 1, column: 2 }), Some("old"));
        assert_eq!(data.cell(Cell { row: 1, column: 3 }), Some("old"));
    }
}
// Non-finite input is never mistaken for omittable text, and an overflowing
// aggregate or final formatted value cannot leak earlier private results.
#[test]
fn evaluation_nonfinite() {
    for input in ["NaN", "inf", "-inf", "1e999"] {
        let data = grid(&[&[input, "old"]], 0);
        for formula in ["$2=$1", "$2=vsum($1)", "$2=vmax($1)"] {
            assert_eq!(
                failure(&data, formula, Policy::default()).kind,
                ErrorKind::NonFinite
            );
        }
    }
    let data = grid(&[&["1e308", "1e308", "old"]], 0);
    assert_eq!(
        failure(&data, "$3=vsum($1..$2)", Policy::default()).kind,
        ErrorKind::NonFinite
    );
    let policy = Policy {
        significant_digits: 1,
        ..Policy::default()
    };
    assert_eq!(
        failure(&data, "$2=2::$3=1.7976931348623157e308", policy).kind,
        ErrorKind::NonFinite
    );
}
// Public policy bounds, actual traversed work and full untrimmed input bytes
// constrain numerical work; cancellation is checked before engine allocation.
#[test]
fn evaluation_limits() {
    let data = grid(&[&["1", "old"]], 0);
    for policy in [
        Policy {
            cells: 1,
            ..Policy::default()
        },
        Policy {
            cells: 1000001,
            ..Policy::default()
        },
        Policy {
            steps: 1,
            ..Policy::default()
        },
        Policy {
            steps: 10000001,
            ..Policy::default()
        },
        Policy {
            significant_digits: 0,
            ..Policy::default()
        },
        Policy {
            significant_digits: 16,
            ..Policy::default()
        },
        Policy {
            decimal_places: 16,
            ..Policy::default()
        },
    ] {
        assert_eq!(failure(&data, "$2=$1+1", policy).kind, ErrorKind::Limit);
    }
    let policy = Policy {
        steps: 100,
        ..Policy::default()
    };
    let padded = Grid::new(vec![vec![" ".repeat(1000), "old".into()]], 0).unwrap();
    assert_eq!(failure(&padded, "$2=$1", policy).kind, ErrorKind::Limit);
    let policy = Policy {
        decimal_places: 0,
        ..Policy::default()
    };
    assert_eq!(failure(&data, "$2=2;f1", policy).kind, ErrorKind::Limit);
    let program = Program::parse("$2=$1", Limits::default(), &Cancellation::default()).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        program
            .evaluate(&data, Policy::default(), &cancel)
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
}
// Formula-relative UTF-8 spans identify the failing operand or operation; a bad
// assignment target cannot claim a valid output cell that does not exist.
#[test]
fn evaluation_locations() {
    let data = grid(&[&["text", "old"]], 0);
    for (formula, span, cell) in [
        ("$2=1+$3", 5..7, Some(Cell { row: 1, column: 2 })),
        ("$2=1+(1/0)", 6..9, Some(Cell { row: 1, column: 2 })),
        ("@9$2=1", 0..4, None),
    ] {
        let error = failure(&data, formula, Policy::default());
        assert_eq!(error.source, span, "{formula}: {error}");
        assert_eq!(error.cell, cell, "{formula}: {error}");
    }
}
