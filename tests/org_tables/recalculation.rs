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
