use super::evaluation::{evaluate, grid};
use rev::org::{
    Cancellation,
    formula::{
        Limits, Program,
        eval::{Cell, Grid, Policy},
    },
};

// Relative signs mean motion from the output cell, while omitted/zero axes
// retain its current coordinate. Literal answers catch reversed directions.
#[test]
fn evaluation_directions() {
    let data = grid(
        &[
            &["10", "20", "30"],
            &["40", "old", "60"],
            &["70", "80", "90"],
        ],
        0,
    );
    for (reference, expected) in [
        ("@-1$-1", 10.0),
        ("@+1$+1", 90.0),
        ("$-1", 40.0),
        ("$+1", 60.0),
        ("@-1", 20.0),
        ("@+1", 80.0),
        ("@0$-1", 40.0),
        ("@-1$0", 20.0),
        ("vsum(@-1$-1..@+1$-1)", 120.0),
        ("vsum(@+1$+1..@-1$+1)", 180.0),
    ] {
        assert_eq!(
            evaluate(&data, &format!("@2$2={reference}"))[0].value,
            expected,
            "{reference}"
        );
    }
    for range in ["@1$1..@2$2", "@2$2..@1$1", "@1$2..@2$1"] {
        assert_eq!(
            evaluate(&data, &format!("@3$3=vsum({range})"))[0].value,
            70.0,
            "{range}"
        );
    }
}

// Last same-target definitions win, explicit fields override columns regardless
// of source order, and missing ragged fields behave as real empty values.
#[test]
fn evaluation_assignments() {
    let data = grid(
        &[&["A", "B", "C"], &["1", "old", "old"], &["2", "old", "old"]],
        1,
    );
    let results = evaluate(&data, "@2$2=7::$2=$1*2::$2=$1*3::@2$2=9::$3=$2+1");
    assert_eq!(
        results.iter().map(|r| r.value).collect::<Vec<_>>(),
        [9.0, 10.0, 6.0, 7.0]
    );
    assert_eq!(evaluate(&data, "@1$2=99::$2=$1")[0].value, 99.0);
    assert!(evaluate(&grid(&[&["Header"]], 1), "$1=1").is_empty());
    let ragged = grid(&[&["2"], &["3", "old"]], 0);
    assert_eq!(
        (ragged.rows(), ragged.columns(), ragged.header_rows()),
        (2, 2, 0)
    );
    assert_eq!(ragged.cell(Cell { row: 1, column: 2 }), Some(""));
    assert_eq!(
        evaluate(&ragged, "$2=$1+1")
            .iter()
            .map(|r| r.value)
            .collect::<Vec<_>>(),
        [3.0, 4.0]
    );
}

// Ranges depend on every assigned field, not just endpoints. Text-only sum and
// genuinely empty numeric cells retain their distinct aggregate meanings.
#[test]
fn evaluation_range_values() {
    let data = grid(&[&["old", "old", "old", "old", "old", "old"]], 0);
    assert_eq!(
        evaluate(
            &data,
            "$4=vsum($1..$3)::$1=2::$2=$5+3::$3=10::$5=$6+1::$6=1"
        )
        .iter()
        .map(|r| r.value)
        .collect::<Vec<_>>(),
        [2.0, 5.0, 10.0, 17.0, 2.0, 1.0]
    );
    let text = grid(&[&["界", "word", "old"]], 0);
    assert_eq!(evaluate(&text, "$3=vsum($1..$2)")[0].value, 0.0);
    let empty = grid(&[&["", " ", "old"]], 0);
    for function in ["vsum", "vmean", "vmin", "vmax"] {
        assert_eq!(
            evaluate(&empty, &format!("$3={function}($1..$2)"))[0].value,
            0.0,
            "{function}"
        );
    }
    assert_eq!(evaluate(&empty, "$3=$1+$2+1")[0].value, 1.0);
}

// Presentation precision cannot alter dependent arithmetic. Mean avoids an
// unnecessary overflowing sum when the mathematical binary64 result is finite.
#[test]
fn evaluation_precision() {
    let data = grid(&[&["old", "old", "old"]], 0);
    let results = evaluate(&data, "$1=1/3;f1::$2=$1*3::$3=round($1,1)*3;f1");
    assert_eq!(results[0].text, "0.3");
    assert_eq!(results[1].value, 1.0);
    assert!((results[2].value - 0.9).abs() < 1e-15);
    assert_eq!(results[2].text, "0.9");
    for (left, right, expected) in [("1e308", "1e308", 1e308), ("1e308", "-1e308", 0.0)] {
        assert_eq!(
            evaluate(&grid(&[&[left, right, "old"]], 0), "$3=vmean($1..$2)")[0].value,
            expected
        );
    }
    let policy = Policy {
        significant_digits: 1,
        ..Policy::default()
    };
    for (formula, expected) in [
        ("$1=9.5", "1e1"),
        ("$1=-9.5", "-1e1"),
        ("$1=0.000095", "0.0001"),
    ] {
        let parsed = Program::parse(formula, Limits::default(), &Cancellation::default()).unwrap();
        assert_eq!(
            parsed
                .evaluate(&data, policy, &Cancellation::default())
                .unwrap()[0]
                .text,
            expected
        );
    }
}

// A 10000-cell chain reaches the native row ceiling without recursive graph or
// expression stack frames. The first field overrides an invalid predecessor.
#[test]
fn evaluation_long_chain() {
    let data = Grid::new(vec![vec!["old".into()]; 10000], 0).unwrap();
    let results = evaluate(&data, "$1=@-1+1::@1$1=1");
    assert_eq!(results.len(), 10000);
    assert_eq!(results.first().unwrap().value, 1.0);
    assert_eq!(results.last().unwrap().value, 10000.0);
    assert_eq!(
        results.last().unwrap().cell,
        Cell {
            row: 10000,
            column: 1
        }
    );
    assert_eq!(data.cell(results.last().unwrap().cell), Some("old"));
}

// Invalid grid dimensions and header bounds reject before coordinate resolution
// or allocation of dependency state over an unqualified shape.
#[test]
fn evaluation_grid_limits() {
    use rev::org::formula::eval::ErrorKind;
    for (name, rows, headers, expected) in [
        ("no rows", Vec::new(), 0, ErrorKind::Limit),
        ("no columns", vec![Vec::new()], 0, ErrorKind::Limit),
        (
            "too many rows",
            vec![vec![String::new()]; 10001],
            0,
            ErrorKind::Limit,
        ),
        (
            "too many columns",
            vec![vec![String::new(); 257]],
            0,
            ErrorKind::Limit,
        ),
        (
            "outside header",
            vec![vec![String::new()]],
            2,
            ErrorKind::Reference,
        ),
    ] {
        assert_eq!(
            Grid::new(rows, headers).unwrap_err().kind,
            expected,
            "{name}"
        );
    }
}
