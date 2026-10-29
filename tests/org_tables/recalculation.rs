use super::*;
fn readonly_editor(source: &str) -> (Fixture, Session) {
    let f = Fixture::new();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&f.write("notes.org", source)).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 18,
        },
    )
    .unwrap();
    (f, s)
}
// Explicit whole-table recalculation changes result text only, uses a single
// canonical history unit, and repeated identical results do not add history.
#[test]
fn recalculation_transaction() {
    let source =
        "|Item|Units|Price|Total|\n|-+-+-+-|\n|A|2|3|old|\n|B|4|1.25|old|\n#+TBLFM: $4=$2*$3;f2\n";
    let expected = source
        .replacen("|old|", "|6.00|", 1)
        .replacen("|old|", "|5.00|", 1);
    let (_f, mut s) = editor("", source);
    action(&mut s, C::TableRecalculate);
    assert_eq!(text(&s), expected, "{}", s.echo());
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    action(&mut s, C::TableRecalculate);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    super::structure::roundtrip(&mut s, source, &expected);
}
// Failure in a later field, unsupported source, invalid destinations and
// read-only authority cannot commit earlier calculated results or move point.
#[test]
fn recalculation_preservation() {
    for (name, source, readonly, diagnostic) in [
        (
            "late numeric failure",
            "|2|old|\n|bad|old|\n#+TBLFM: $2=$1*2\n",
            false,
            "Numeric",
        ),
        (
            "unsupported",
            "|2|old|\n#+TBLFM: $2='(identity $1)\n",
            false,
            "unsupported",
        ),
        (
            "missing source field",
            "|2|\n|3|old|\n#+TBLFM: $2=$1*2\n",
            false,
            "no existing source field",
        ),
        (
            "read-only",
            "|2|old|\n#+TBLFM: $2=$1*2\n",
            true,
            "read-only",
        ),
    ] {
        let (_f, mut s) = if readonly {
            readonly_editor(source)
        } else {
            editor("", source)
        };
        let view = s.view().clone();
        action(&mut s, C::TableRecalculate);
        assert!(
            s.echo().to_lowercase().contains(&diagnostic.to_lowercase()),
            "{name}: {}",
            s.echo()
        );
        assert_eq!(text(&s), source, "{name}");
        assert_eq!(s.view(), &view, "{name}");
        assert!(s.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Selected output computes its assigned dependencies privately. Unselected
// results and an unrelated failing formula do not enter the selected update.
#[test]
fn recalculation_selection() {
    let source =
        "| 2 | old | old | text |\n| 4 | old | old | text |\n#+TBLFM: $2=$1*2::$3=$2+1::$4=1/0\n";
    for (name, command, expected) in [
        (
            "cell",
            C::TableRecalculateCell,
            source.replacen("| old | text", "| 5 | text", 1),
        ),
        (
            "column",
            C::TableRecalculateColumn,
            source.replacen("| old | text", "| 5 | text", 1).replacen(
                "| old | text",
                "| 9 | text",
                1,
            ),
        ),
    ] {
        let (_f, mut s) = editor("", source);
        at(&mut s, source.find("| old | text").unwrap() + 2);
        action(&mut s, command);
        assert_eq!(text(&s), expected, "{name}: {}", s.echo());
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        super::structure::roundtrip(&mut s, source, &expected);
    }
}

// A table command uses the first formula line. Precise C-c C-c on a later
// formula applies only that program; C-c C-c in the body still only aligns.
#[test]
fn recalculation_formula_context() {
    let source = "| 2 | old |\n#+TBLFM: $2=$1*2\n#+TbLfM: $2=$1*3\n";
    let (_f, mut s) = editor("", source);
    action(&mut s, C::CtrlCCtrlC);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    action(&mut s, C::TableRecalculate);
    assert_eq!(text(&s), source.replacen("old", "4", 1), "{}", s.echo());
    s.execute(Command::History(History::Undo));
    at(&mut s, source.find("#+TbLfM").unwrap());
    action(&mut s, C::CtrlCCtrlC);
    assert_eq!(text(&s), source.replacen("old", "6", 1), "{}", s.echo());
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
}

// Each admitted numeric preference changes live recalculation or constrains it
// before publication; invalid configuration is rejected by native startup.
#[test]
fn recalculation_configuration() {
    let source = "|1|old|\n#+TBLFM: $2=1/3;f2\n";
    for (name, config, expected) in [
        (
            "digits",
            "org.tables.significant_digits=3",
            "|1|0.333|\n#+TBLFM: $2=1/3\n",
        ),
        ("cells", "org.limits.formula_cells=1", source),
        ("steps", "org.limits.formula_steps=1", source),
        (
            "depth",
            "org.limits.formula_depth=1",
            "|1|old|\n#+TBLFM: $2=((1))\n",
        ),
        ("decimals", "org.limits.formula_decimal_places=1", source),
    ] {
        let input = if name == "digits" {
            "|1|old|\n#+TBLFM: $2=1/3\n"
        } else {
            expected
        };
        let (_f, mut s) = editor(config, input);
        action(&mut s, C::TableRecalculate);
        assert_eq!(text(&s), expected, "{name}: {}", s.echo());
        assert_eq!(
            s.buffer().unwrap().history().len(),
            usize::from(name == "digits"),
            "{name}"
        );
    }
    let f = Fixture::new();
    for (name, config) in [
        ("negative", "org.tables.significant_digits=-1"),
        ("digits", "org.tables.significant_digits=16"),
        ("cells", "org.limits.formula_cells=1000001"),
        ("steps", "org.limits.formula_steps=10000001"),
        ("depth", "org.limits.formula_depth=257"),
        ("places", "org.limits.formula_decimal_places=16"),
    ] {
        assert!(
            prepare(&f.write("invalid.toml", config))
                .diagnostic
                .is_some(),
            "{name}"
        );
    }
}
