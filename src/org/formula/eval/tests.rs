use super::*;
// The numerical worker honors an already expired deadline through the same
// private token as syntax and IO, without publishing or exposing a new timer API.
#[test]
fn evaluation_deadline() {
    let program = Program::parse(
        "$1=2+2",
        super::super::Limits::default(),
        &Cancellation::default(),
    )
    .unwrap();
    let grid = Grid::new(vec![vec!["old".into()]], 0).unwrap();
    let expired = Cancellation::default().bounded(std::time::Instant::now());
    let error = program
        .evaluate(&grid, Policy::default(), &expired)
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Cancelled);
    assert_eq!(grid.cell(Cell { row: 1, column: 1 }), Some("old"));
}
