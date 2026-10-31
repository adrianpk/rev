use super::reorder::Piece;
use super::{Buffer, Change, Cursor, Error, Permutation, Point, Prepared, Replacement, Retained};
use ropey::Rope;

impl Buffer {
    /// Publish retained fragments through the existing candidate/history owner.
    /// Other positions use ordinary edit affinity, including deleted source.
    pub(crate) fn prepare_retained(
        &self,
        edits: &[Replacement],
        points: &[Point],
        cursor: Option<Cursor>,
        retained: &[Retained],
    ) -> Result<Prepared, Error> {
        self.admit(self.stamp())?;
        if retained.is_empty() {
            return self.prepare(edits, points, cursor);
        }
        if retained.len() > 200000 {
            return Err(Error::Capacity);
        }
        let changes = self.changes(edits)?;
        let candidate = self.candidate(&changes)?;
        let order = retained_order(self, &candidate, retained, &changes)?;
        self.prepare_ordered(edits, points, cursor, Some(order))
    }
}
fn retained_order(
    buffer: &Buffer,
    candidate: &Rope,
    retained: &[Retained],
    changes: &[Change],
) -> Result<Permutation, Error> {
    let mut pieces = Vec::with_capacity(retained.len());
    for run in retained {
        pieces.push(admit_run(buffer, candidate, run, changes)?);
    }
    disjoint(pieces.iter().map(|p| p.before.clone()).collect())?;
    disjoint(pieces.iter().map(|p| p.after.clone()).collect())?;
    pieces.sort_by_key(|p| p.before.start);
    Ok(Permutation {
        pieces,
        endpoints: None,
        sparse: true,
    })
}
fn admit_run(
    buffer: &Buffer,
    candidate: &Rope,
    run: &Retained,
    changes: &[Change],
) -> Result<Piece, Error> {
    let source = buffer.slice(&run.source)?;
    let output = candidate
        .get_slice(run.output.clone())
        .ok_or(Error::Order)?;
    if source.len_chars() == 0 || source != output {
        return Err(Error::Order);
    }
    let index = changes.partition_point(|c| c.before.start <= run.source.range.start);
    let owner = changes
        .get(index.checked_sub(1).ok_or(Error::Order)?)
        .ok_or(Error::Order)?;
    if run.source.range.end > owner.before.end
        || run.output.start < owner.after.start
        || run.output.end > owner.after.end
    {
        return Err(Error::Order);
    }
    Ok(Piece {
        retained: true,
        before: run.source.range.clone(),
        after: run.output.clone(),
    })
}
fn disjoint(mut ranges: Vec<std::ops::Range<usize>>) -> Result<(), Error> {
    ranges.sort_by_key(|r| r.start);
    if ranges.windows(2).any(|p| p[0].end > p[1].start) {
        return Err(Error::Order);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{buffer::Buffers, coordinates::Scalar};

    // Retention is checked against both real revisions: stale identities,
    // changed bytes and overlapping/out-of-range claims cannot publish anchors.
    #[test]
    fn retained_admission() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("AAAABBBB").unwrap();
        let foreign = buffers.create("AAAA").unwrap();
        let foreign_span = buffers.get(foreign).unwrap().span(0..4).unwrap();
        let buffer = buffers.get(id).unwrap();
        let edit = Replacement {
            source: buffer.span(0..8).unwrap(),
            text: "BBBBAAAA".into(),
        };
        let run = |source, output| Retained {
            source: buffer.span(source).unwrap(),
            output,
        };
        let valid = vec![run(0..4, 4..8), run(4..8, 0..4)];
        let point = Point {
            stamp: buffer.stamp(),
            position: Scalar(2),
        };
        let prepared = buffer
            .prepare_retained(std::slice::from_ref(&edit), &[point], None, &valid)
            .unwrap();
        drop(prepared);
        let cases = [
            ("changed bytes", vec![run(0..4, 0..4)]),
            ("overlapping source", vec![run(0..2, 4..6), run(0..2, 6..8)]),
            ("overlapping output", vec![run(0..2, 4..6), run(2..4, 4..6)]),
            ("output bounds", vec![run(0..2, 9..11)]),
            ("empty run", vec![run(0..0, 0..0)]),
            (
                "foreign identity",
                vec![Retained {
                    source: foreign_span,
                    output: 4..8,
                }],
            ),
        ];
        for (name, runs) in cases {
            assert!(
                buffer
                    .prepare_retained(std::slice::from_ref(&edit), &[point], None, &runs)
                    .is_err(),
                "{name}"
            );
            assert_eq!(
                buffer.slice(&buffer.span(0..8).unwrap()).unwrap(),
                "AAAABBBB"
            );
            assert!(buffer.history().is_empty());
        }
        let outside_edit = Replacement {
            source: buffer.span(0..4).unwrap(),
            text: "CCCC".into(),
        };
        assert!(
            buffer
                .prepare_retained(&[outside_edit], &[point], None, &[run(4..8, 4..8)])
                .is_err()
        );
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(true);
        assert!(matches!(
            buffer.prepare_retained(&[edit], &[point], None, &valid),
            Err(Error::ReadOnly)
        ));
    }
}
