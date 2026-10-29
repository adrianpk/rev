use rev::org::{
    Cancellation,
    formula::{
        Limits, Program,
        eval::{Grid, Policy, ResultCell},
    },
};
pub(super) fn grid(rows: &[&[&str]], headers: usize) -> Grid {
    Grid::new(
        rows.iter()
            .map(|r| r.iter().map(|s| (*s).into()).collect())
            .collect(),
        headers,
    )
    .unwrap()
}
pub(super) fn evaluate(data: &Grid, formula: &str) -> Vec<ResultCell> {
    Program::parse(formula, Limits::default(), &Cancellation::default())
        .unwrap()
        .evaluate(data, Policy::default(), &Cancellation::default())
        .unwrap()
}
// Literal arithmetic is checked against independently specified numeric answers,
// including Org precedence, rightward powers and negative half-away rounding.
#[test]
fn evaluated_arithmetic() {
    let data = grid(&[&["old"]], 0);
    for (name, expression, expected) in [
        ("addition", "2+3*4", 14.0),
        ("Org division", "8/2*2", 2.0),
        ("power", "2^3^2", 512.0),
        ("negative power", "-2^2", -4.0),
        ("negative exponent", "2^-3", 0.125),
        ("absolute", "abs(-4)", 4.0),
        ("negative half", "round(-1.5)", -2.0),
        ("decimal half", "round(-1.25,1)", -1.3),
        ("coarse half", "round(-125,-1)", -130.0),
    ] {
        let results = evaluate(&data, &format!("$1={expression}"));
        assert_eq!(results.len(), 1, "{name}");
        assert_eq!(results[0].value, expected, "{name}");
        assert_eq!(data.cell(results[0].cell), Some("old"));
    }
}
// Effective field formulas override column formulas, headers remain unchanged,
// and later-defined dependencies are evaluated before their consumers.
#[test]
fn evaluated_dependencies() {
    let data = grid(
        &[
            &["Input", "Double", "Next"],
            &["2", "old", "old"],
            &["3", "old", "old"],
        ],
        1,
    );
    let results = evaluate(&data, "$3=$2+1::$2=$1*2::@3$2=20");
    let actual: Vec<_> = results
        .iter()
        .map(|r| (r.cell.row, r.cell.column, r.text.as_str()))
        .collect();
    assert_eq!(
        actual,
        vec![(2, 2, "4"), (2, 3, "5"), (3, 2, "20"), (3, 3, "21")]
    );
    assert_eq!(
        data.cell(rev::org::formula::eval::Cell { row: 1, column: 2 }),
        Some("Double")
    );
}
// Empty cells count as zero; text cells are omitted only inside aggregates.
// Every endpoint belongs to the inclusive rectangular range, including empties.
#[test]
fn evaluated_aggregates() {
    let data = grid(&[&["1", "", "old"], &["text", "3", "old"]], 0);
    for (function, expected, text) in [
        ("vsum", 4.0, "4"),
        ("vmean", 4.0 / 3.0, "1.33333333333"),
        ("vmin", 0.0, "0"),
        ("vmax", 3.0, "3"),
    ] {
        let results = evaluate(&data, &format!("$3={function}(@1$1..@2$2)"));
        assert_eq!(results.len(), 2);
        assert!((results[0].value - expected).abs() < 1e-15, "{function}");
        assert_eq!(results[0].text, text, "{function}");
    }
}
// Fixed decimal text uses the selected half-away rule. Default significant
// formatting preserves small finite values and produces bounded numeric text.
#[test]
fn evaluated_formatting() {
    let data = grid(&[&["old"]], 0);
    for (name, formula, expected) in [
        ("negative fixed half", "$1=-1.25;%.1f", "-1.3"),
        ("positive fixed half", "$1=1.25;f1", "1.3"),
        ("fixed zero", "$1=-0.001;f2", "0.00"),
        ("significant", "$1=1234567890123", "1.23456789012e12"),
        ("small", "$1=0.0000012345", "1.2345e-6"),
        ("subnormal", "$1=5e-324", "5e-324"),
        ("maximum", "$1=1.7976931348623157e308", "1.79769313486e308"),
    ] {
        assert_eq!(evaluate(&data, formula)[0].text, expected, "{name}");
    }
}
