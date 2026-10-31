use std::collections::{BTreeMap, VecDeque};

use crate::buffer::Stamp;

use super::{Cell, Scalar};

/// Disposable coordinate observations, shared by a buffer's views. The cap is
/// independent of source size; eviction only causes a checked recomputation.
#[derive(Default)]
pub(crate) struct Checkpoints {
    identity: Option<(Stamp, usize)>,
    points: BTreeMap<Scalar, Cell>,
    order: VecDeque<Scalar>,
    changes: VecDeque<(Stamp, Stamp, Scalar)>,
}

pub(crate) const LIMIT: usize = 1024;
pub(crate) const INTERVAL: usize = 256;

impl Checkpoints {
    pub(crate) fn observe(&mut self, stamp: Stamp, tab_width: usize) {
        if self.identity != Some((stamp, tab_width)) {
            self.points.clear();
            self.order.clear();
            self.identity = Some((stamp, tab_width));
        }
    }

    pub(crate) fn before(&self, point: Scalar, row: usize) -> Option<(Scalar, usize)> {
        self.points
            .range(..=point)
            .next_back()
            .filter(|(_, cell)| cell.row == row)
            .map(|(source, cell)| (*source, cell.column))
    }

    pub(crate) fn column(
        &self,
        cell: Cell,
        source: std::ops::RangeInclusive<Scalar>,
    ) -> Option<(Scalar, usize)> {
        self.points
            .range(source)
            .rfind(|(_, at)| at.row == cell.row && at.column <= cell.column)
            .map(|(source, at)| (*source, at.column))
    }

    pub(crate) fn retain_prefix(&mut self, before: Stamp, after: Stamp, start: Scalar) {
        self.changes.push_back((before, after, start));
        if self.changes.len() > 64 {
            self.changes.pop_front();
        }
        if self.identity.is_some_and(|(stamp, _)| stamp == before) {
            self.points.retain(|point, _| *point < start);
            self.order.retain(|point| *point < start);
            if let Some((_, tab)) = self.identity {
                self.identity = Some((after, tab));
            }
        } else {
            self.identity = None;
            self.points.clear();
            self.order.clear();
        }
    }

    pub(crate) fn unchanged_prefix(&self, before: Stamp, after: Stamp) -> Option<Scalar> {
        let mut expected = before;
        let mut prefix = Scalar(usize::MAX);
        for (old, new, start) in &self.changes {
            if *old == expected {
                expected = *new;
                prefix = prefix.min(*start);
                if expected == after {
                    return Some(prefix);
                }
            }
        }
        None
    }

    pub(crate) fn remember(&mut self, point: Scalar, cell: Cell) {
        if self.points.insert(point, cell).is_none() {
            self.order.push_back(point);
        }
        while self.order.len() > LIMIT {
            let oldest = self
                .order
                .pop_front()
                .expect("checkpoint retention exceeds its cap");
            self.points.remove(&oldest);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffers, edit::Replacement};
    use crate::coordinates::Coordinates;

    // Checkpoint restarts must match an uncached full-line scan across rope
    // chunks, RI/ZWJ/combining clusters, tabs, CRLF and source-line changes.
    #[test]
    fn coordinate_restarts() {
        let source = format!(
            "{}\r\n{}",
            "a\t界e\u{301}🇦🇧👩‍💻 ".repeat(400),
            "tail ".repeat(100)
        );
        let text = ropey::Rope::from_str(&source);
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&source).unwrap();
        let b = buffers.get(id).unwrap();
        let reference = Coordinates::for_text(text.slice(..), b.stamp(), b.policy().tab_width);
        let cached = Coordinates::new(b).unwrap();
        for at in [source.chars().count(), 3500, 12, 2000, 4000, 1, 499, 0] {
            let point = Scalar(at);
            assert_eq!(cached.to_cell(point), reference.to_cell(point), "{at}");
        }
        for cell in [
            Cell {
                row: 0,
                column: 3000,
            },
            Cell { row: 0, column: 1 },
            Cell {
                row: 1,
                column: 400,
            },
            Cell {
                row: 1,
                column: usize::MAX,
            },
        ] {
            assert_eq!(
                cached.from_cell(cell),
                reference.from_cell(cell),
                "{cell:?}"
            );
        }
    }

    // A changed preceding glyph (emoji variation/combining insertion) and an
    // earlier newline deletion invalidate suffix columns under a fresh stamp.
    #[test]
    fn published_prefix() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers
            .create(&format!("{}❤Z\r\nlast", "x".repeat(600)))
            .unwrap();
        let b = buffers.get_mut(id).unwrap();
        for (start, end, inserted) in [(601, 601, "\u{fe0f}"), (603, 605, ""), (0, 0, "a\t")] {
            let coordinates = Coordinates::new(b).unwrap();
            coordinates.to_cell(Scalar(b.len_chars())).unwrap();
            let edit = Replacement {
                source: b.span(start..end).unwrap(),
                text: inserted.into(),
            };
            let prepared = b.prepare(&[edit], &[], None).unwrap();
            b.publish(prepared, &mut []).unwrap();
            let source = b
                .slice(&b.span(0..b.len_chars()).unwrap())
                .unwrap()
                .to_string();
            let text = ropey::Rope::from_str(&source);
            let reference = Coordinates::for_text(text.slice(..), b.stamp(), b.policy().tab_width);
            let cached = Coordinates::new(b).unwrap();
            for at in 598..=b.len_chars() {
                assert_eq!(
                    cached.to_cell(Scalar(at)),
                    reference.to_cell(Scalar(at)),
                    "{start}/{at}"
                );
            }
        }
    }

    // Bounds are hard caps under duplicate observations and more publications
    // than the finite prefix-proof journal can retain.
    #[test]
    fn retention_bounds() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("").unwrap();
        let stamp = buffers.get(id).unwrap().stamp();
        let mut cache = Checkpoints::default();
        cache.observe(stamp, 4);
        for at in 0..LIMIT * 3 {
            cache.remember(Scalar(at), Cell { row: 0, column: at });
        }
        assert_eq!(cache.points.len(), LIMIT);
        assert_eq!(cache.order.len(), LIMIT);
        let mut before = stamp;
        for n in 1..=65 {
            let after = Stamp {
                revision: stamp.revision + n,
                ..stamp
            };
            cache.retain_prefix(before, after, Scalar(3));
            before = after;
        }
        assert_eq!(cache.changes.len(), 64);
        assert_eq!(cache.unchanged_prefix(stamp, before), None);
    }
}
