use super::structure::roundtrip;
use super::*;

// Absolute targets and operands follow a real column permutation. Only changed
// coordinate digits are replaced; alternate formula lines and formatting remain.
#[test]
fn column_references() {
    let source = "#+NAME: stable\r\n#+CAPTION: Keep\r\n|a|b|c|d|\r\n|-+-+-+-|\r\n|1|2|3|4|\r\n#+TBLFM: $04 = $01 + 2*$03 :: @02$02 = round(@02$01 + 1.5,1)\r\n#+TBLFM: @2$3=vsum($1..$2)";
    let (_f, mut s) = editor("", source);
    at(&mut s, source.find("c|d").unwrap());
    action(&mut s, C::TableMoveColumnRight);
    let after = text(&s);
    assert!(after.starts_with("#+NAME: stable\r\n#+CAPTION: Keep\r\n"));
    assert!(after.ends_with("#+TBLFM: $3 = $01 + 2*$4 :: @02$02 = round(@02$01 + 1.5,1)\r\n#+TBLFM: @2$4=vsum($1..$2)"), "{}", s.echo());
    assert_eq!(
        super::metadata::values(&s),
        vec!["a", "b", "d", "c", "1", "2", "4", "3"]
    );
    roundtrip(&mut s, source, &after);
}
// Data row coordinates ignore hlines. Row movement and row insertion retarget
// both assignment fields and row-only operands in one ordinary source operation.
#[test]
fn row_references() {
    let source = "|A|1|\n|-+-|\n|B|2|\n|C|3|\n#+TBLFM: @2$2=@3$1+@1$2::$2=@2+1\n";
    for (name, command, formula) in [
        ("move", C::TableMoveRowDown, "@3$2=@2$1+@1$2::$2=@3+1"),
        ("insert", C::TableInsertRow, "@3$2=@4$1+@1$2::$2=@3+1"),
        ("hline", C::TableInsertHline, "@2$2=@3$1+@1$2::$2=@2+1"),
    ] {
        let (_f, mut s) = editor("", source);
        at(&mut s, source.find("B|2").unwrap());
        action(&mut s, command);
        let after = text(&s);
        assert!(
            after.ends_with(&format!("#+TBLFM: {formula}\n")),
            "{name}: {}",
            s.echo()
        );
        roundtrip(&mut s, source, &after);
    }
}
// Inserting/deleting unreferenced axes shifts actual old targets. A target that
// would disappear rejects instead of dropping a formula or inventing a value.
#[test]
fn axis_changes() {
    let source = "|A|B|C|D|\n|1|2|3|4|\n#+TBLFM: $4=$1+$3\n";
    for (command, formula) in [
        (C::TableInsertColumn, "$5=$1+$4"),
        (C::TableDeleteColumn, "$3=$1+$2"),
    ] {
        let (_f, mut s) = editor("", source);
        at(&mut s, source.find("B|C").unwrap());
        action(&mut s, command);
        let after = text(&s);
        assert!(
            after.ends_with(&format!("#+TBLFM: {formula}\n")),
            "{}",
            s.echo()
        );
        roundtrip(&mut s, source, &after);
    }
    let (_f, mut s) = editor("", source);
    at(&mut s, source.find("C|D").unwrap());
    let view = s.view().clone();
    action(&mut s, C::TableDeleteColumn);
    assert!(s.echo().contains("removed or outside"), "{}", s.echo());
    assert_eq!(text(&s), source);
    assert_eq!(s.view(), &view);
    assert!(s.buffer().unwrap().history().is_empty());
}
// Uniform shifts preserve every cell of a rectangular range, including its
// original row/column context. The rewritten form remains compatible Org source.
#[test]
fn rectangular_range() {
    let source = "|A|B|C|D|\n|1|2|3|4|\n|5|6|7|8|\n#+TBLFM: @3$4=mean(@1$1..@2$2)\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 1);
    action(&mut s, C::TableInsertColumn);
    let after = text(&s);
    assert!(
        after.ends_with("#+TBLFM: @3$5=mean(@1$2..@2$3)\n"),
        "{}",
        s.echo()
    );
    roundtrip(&mut s, source, &after);
}
// Endpoint provenance is insufficient: a swap inside a range or an insertion
// through it changes its interior target sequence and must reject atomically.
#[test]
fn range_target_proof() {
    let source = "|A|B|C|D|E|\n|1|2|3|4|5|\n#+TBLFM: $5=vsum($1..$3)\n";
    for (name, command) in [
        ("move", C::TableMoveColumnRight),
        ("insert", C::TableInsertColumn),
    ] {
        let (_f, mut s) = editor("", source);
        at(&mut s, source.find("B|C").unwrap());
        let view = s.view().clone();
        action(&mut s, command);
        assert!(s.echo().contains("Range no longer"), "{name}: {}", s.echo());
        assert_eq!(text(&s), source);
        assert_eq!(s.view(), &view);
        assert!(s.buffer().unwrap().history().is_empty());
    }
}
// Unknown Calc/Lisp forms, relative/named/hlines/current references, malformed
// expressions and out-of-table targets are never rewritten speculatively.
#[test]
fn unsupported_references() {
    for (name, formula) in [
        ("lisp", "$2='(evil $1)"),
        ("remote", "$2=remote(name,$1)"),
        ("relative", "$2=$+1"),
        ("hline", "$2=@I$1"),
        ("named", "$2=$name"),
        ("letter", "$2=A1"),
        ("malformed", "$2=($1+"),
        ("current", "$2=$0"),
        ("column bounds", "$2=$9"),
        ("row bounds", "$2=@4$1"),
    ] {
        let source = format!("| A | B | C |\n| 1 | 2 | 3 |\n#+TBLFM: {formula}\n");
        let (_f, mut s) = editor("", &source);
        at(&mut s, 10);
        let view = s.view().clone();
        action(&mut s, C::TableMoveColumnLeft);
        assert!(
            s.echo().contains("reference rewriting"),
            "{name}: {}",
            s.echo()
        );
        assert_eq!(text(&s), source, "{name}");
        assert_eq!(s.view(), &view, "{name}");
        assert!(s.buffer().unwrap().history().is_empty());
    }
}
// Identical cell bytes do not make logical column movement a no-op. The actual
// selected field and references move together and undo restores both identities.
#[test]
fn identical_fields() {
    let source = "| a | a | a |\n#+TBLFM: $3=$1+$2\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 2);
    action(&mut s, C::TableMoveColumnRight);
    assert_eq!(
        text(&s),
        "| a | a | a |\n#+TBLFM: $3=$2+$1\n",
        "{}",
        s.echo()
    );
    assert_eq!(s.view().point.position.0, 6);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    assert_eq!(s.view().point.position.0, 2);
}
