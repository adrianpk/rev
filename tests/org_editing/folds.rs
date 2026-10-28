use std::ops::Range;

use rev::buffer::{Buffer, Mode};
use rev::coordinates::{Byte, Coordinates, LineByte};
use rev::org::{Source, SourceRange};
use rev::view::{Error, FoldError};

use super::*;

pub(super) fn range(buffer: &Buffer, rows: Range<usize>) -> SourceRange {
    let at = |row| {
        if row == buffer.len_lines() {
            return buffer.len_bytes();
        }
        let coordinates = Coordinates::new(buffer).unwrap();
        coordinates
            .to_byte(coordinates.from_line(LineByte { row, column: 0 }).unwrap())
            .unwrap()
            .0
    };
    Source::new(buffer)
        .range(Byte(at(rows.start))..Byte(at(rows.end)))
        .unwrap()
}

pub(super) fn buffer(source: &str) -> (Buffers, rev::buffer::BufferId) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    buffers.get_mut(id).unwrap().select_mode(Mode::Org).unwrap();
    (buffers, id)
}

// The row map is a bijection over visible canonical lines. Nested/overlapping
// ranges merge without duplicating hidden counts; cloned views remain separate.
#[test]
fn independent_map() {
    let (buffers, id) = buffer("* A\nbody\n** B\nchild\n* C\nlast");
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.set_folds(buffer, &[range(buffer, 2..4), range(buffer, 1..3)])
        .unwrap();
    assert_eq!(view.visible_lines(buffer), 3);
    for (local, source) in [0, 4, 5].into_iter().enumerate() {
        assert_eq!(view.source_row(buffer, local), Some(source));
        assert_eq!(view.screen_row(buffer, source), Some(local));
    }
    for source in 1..4 {
        assert_eq!(view.screen_row(buffer, source), None);
    }
    assert_eq!(view.source_row(buffer, 3), None);
    assert_eq!(view.source_row(buffer, usize::MAX), None);
    let mut peer = view.clone();
    peer.clear_folds();
    assert_eq!(peer.visible_lines(buffer), 6);
    assert_eq!(view.visible_lines(buffer), 3);
    assert_eq!(buffer.history().len(), 0);
    assert!(!buffer.is_modified());
}

// Invalid authority or partial line/CRLF boundaries cannot replace a valid map.
// Folding the point is explicitly rejected instead of moving it to another row.
#[test]
fn fold_admission() {
    let (mut buffers, id) = buffer("* A\r\nbody\r\n* B\r\n");
    let other = buffers.create("* A\r\nbody\r\n* B\r\n").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    let valid = range(buffer, 1..2);
    view.set_folds(buffer, std::slice::from_ref(&valid))
        .unwrap();
    let before = view.clone();
    let partial = Source::new(buffer).range(Byte(1)..Byte(5)).unwrap();
    assert_eq!(
        view.set_folds(buffer, &[partial]),
        Err(Error::Fold(FoldError::Boundary(rev::coordinates::Scalar(
            1
        ))))
    );
    let foreign = range(buffers.get(other).unwrap(), 1..2);
    assert!(view.set_folds(buffer, &[foreign]).is_err());
    assert_eq!(view.source_row(buffers.get(other).unwrap(), 0), None);
    assert_eq!(view.screen_row(buffers.get(other).unwrap(), 0), None);
    let crlf = Source::new(buffer).range(Byte(4)..Byte(5)).unwrap();
    assert_eq!(
        view.set_folds(buffer, &[crlf]),
        Err(Error::Fold(FoldError::Boundary(rev::coordinates::Scalar(
            4
        ))))
    );
    assert_eq!(
        view.set_folds(buffer, &vec![valid.clone(); 4097]),
        Err(Error::Fold(FoldError::Limit))
    );
    assert!(view.set_folds(buffer, &[range(buffer, 0..1)]).is_err());
    assert_eq!(view, before);
    buffers
        .get_mut(id)
        .unwrap()
        .select_mode(Mode::Text)
        .unwrap();
    assert_eq!(
        view.set_folds(buffers.get(id).unwrap(), &[valid]),
        Err(Error::Fold(FoldError::Mode))
    );
    assert_eq!(view.visible_lines(buffers.get(id).unwrap()), 4);
}

// Unterminated EOF rows are valid complete source ranges. Empty final LF rows
// remain distinct visible insertion positions, including an otherwise empty doc.
#[test]
fn eof_projection() {
    for (name, source, hidden, visible) in [
        ("unterminated", "* A\nlast", 1..2, vec![0]),
        ("LF", "* A\nlast\n", 1..2, vec![0, 2]),
        ("empty", "", 0..0, vec![0]),
    ] {
        let (buffers, id) = buffer(source);
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.set_folds(buffer, &[range(buffer, hidden)]).unwrap();
        assert_eq!(
            (0..view.visible_lines(buffer))
                .map(|row| view.source_row(buffer, row).unwrap())
                .collect::<Vec<_>>(),
            visible,
            "{name}"
        );
    }
}
