//! Short-lived logical-line identities, mapped by the canonical edit owner.
//! They are not retained in undo records or given file-reopening authority.
use super::{Buffer, Error, Stamp, Transaction, mapping};
use crate::coordinates::{Coordinates, Scalar};
use std::sync::{Arc, Mutex, Weak};

pub(crate) const LINE_LIMIT: usize = 65_536;
pub(crate) type Shared = Arc<Mutex<MacroLines>>;
pub(crate) type WeakLines = Weak<Mutex<MacroLines>>;

pub(crate) struct MacroLines {
    pub stamp: Stamp,
    pub positions: Vec<Option<Scalar>>,
}
pub(super) struct Mapped {
    owner: Shared,
    positions: Vec<Option<Scalar>>,
}

impl Buffer {
    pub(crate) fn macro_lines(&mut self, positions: Vec<Scalar>) -> Result<Shared, Error> {
        self.macro_lines.retain(|owner| owner.strong_count() > 0);
        // Four suspended manual frames and the current automatic run.
        if self.macro_lines.len() >= 5 || positions.len() > LINE_LIMIT {
            return Err(Error::Capacity);
        }
        let coordinates = Coordinates::new(self)?;
        for &position in &positions {
            if coordinates.line_start(position)? != position {
                return Err(Error::ChangedPositions);
            }
        }
        let owner = Arc::new(Mutex::new(MacroLines {
            stamp: self.stamp(),
            positions: positions.into_iter().map(Some).collect(),
        }));
        self.macro_lines.push(Arc::downgrade(&owner));
        Ok(owner)
    }
}

pub(super) fn prepare(
    buffer: &Buffer,
    record: &Transaction,
    coordinates: &Coordinates<'_>,
) -> Result<Vec<Mapped>, Error> {
    buffer
        .macro_lines
        .iter()
        .filter_map(Weak::upgrade)
        .map(|owner| {
            let current = owner.lock().expect("macro line owner poisoned");
            if current.stamp != record.before {
                return Err(Error::ChangedPositions);
            }
            let positions = current
                .positions
                .iter()
                .map(|position| {
                    position
                        .map(|point| mapped(record, coordinates, point))
                        .transpose()
                        .map(Option::flatten)
                })
                .collect::<Result<_, _>>()?;
            drop(current);
            Ok(Mapped { owner, positions })
        })
        .collect()
}

fn mapped(
    record: &Transaction,
    coordinates: &Coordinates<'_>,
    point: Scalar,
) -> Result<Option<Scalar>, Error> {
    let retained = record
        .permutation
        .as_ref()
        .and_then(|permutation| permutation.bookmark(point));
    if retained.is_none()
        && record
            .changes
            .iter()
            .any(|change| change.before.contains(&point.0) && change.removed.len_lines() > 1)
    {
        return Ok(None);
    }
    let inserted_lines = record.changes.iter().any(|change| {
        change.before.is_empty()
            && change.before.start == point.0
            && change.inserted.len_lines() > 1
    });
    let affinity = if inserted_lines {
        mapping::Affinity::Right
    } else {
        mapping::Affinity::Left
    };
    let position = match retained {
        Some(position) => position,
        None => mapping::mapped(&record.changes, point, affinity, None)?,
    };
    if !coordinates.is_boundary(position)? {
        return Ok(None);
    }
    let start = coordinates.line_start(position)?;
    // At a line-start insertion, follow the original text beyond inserted rows.
    // A joined/deleted original boundary is never snapped to a different line.
    Ok(if inserted_lines {
        Some(start)
    } else {
        (start == position).then_some(position)
    })
}

pub(super) fn admit(buffer: &Buffer, mapped: &[Mapped]) -> Result<(), Error> {
    let owners: Vec<_> = buffer
        .macro_lines
        .iter()
        .filter_map(Weak::upgrade)
        .collect();
    if owners.len() != mapped.len()
        || owners.iter().zip(mapped).any(|(owner, map)| {
            !Arc::ptr_eq(owner, &map.owner)
                || owner.lock().expect("macro line owner poisoned").stamp != buffer.stamp()
        })
    {
        return Err(Error::ChangedPositions);
    }
    Ok(())
}
pub(super) fn publish(mapped: Vec<Mapped>, stamp: Stamp) {
    for map in mapped {
        let mut owner = map.owner.lock().expect("macro line owner poisoned");
        owner.stamp = stamp;
        owner.positions = map.positions;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffers, edit::Replacement};

    // A newly registered target set invalidates an already prepared edit.
    // Accepted edits map live targets, but undo history cannot keep them alive.
    #[test]
    fn regions_admission() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("a\nb").unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let edits = [Replacement {
            source: buffer.span(2..2).unwrap(),
            text: "new\n".into(),
        }];
        let prepared = buffer.prepare(&edits, &[], None).unwrap();
        let before = buffer.stamp();
        let owner = buffer.macro_lines(vec![Scalar(0), Scalar(2)]).unwrap();
        assert_eq!(
            buffer.publish(prepared, &mut []),
            Err(Error::ChangedPositions)
        );
        assert_eq!(buffer.stamp(), before);
        assert!(buffer.history().is_empty());
        assert_eq!(buffer.slice(&buffer.span(0..3).unwrap()).unwrap(), "a\nb");
        let prepared = buffer.prepare(&edits, &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
        {
            let mapped = owner.lock().unwrap();
            assert_eq!(mapped.stamp, buffer.stamp());
            assert_eq!(mapped.positions, [Some(Scalar(0)), Some(Scalar(6))]);
        }
        let weak = Arc::downgrade(&owner);
        drop(owner);
        assert_eq!(buffer.history().len(), 1);
        assert!(weak.upgrade().is_none());
    }
}
