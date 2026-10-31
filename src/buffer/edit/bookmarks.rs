use crate::bookmark::Anchor;
use crate::coordinates::Coordinates;

use super::{Error, Transaction, mapping};

pub(super) fn map(
    record: &Transaction,
    coordinates: &Coordinates<'_>,
) -> Result<Vec<Anchor>, Error> {
    record
        .before_bookmarks
        .iter()
        .map(|anchor| {
            let Some(point) = anchor.position else {
                return Ok(*anchor);
            };
            let retained = record.permutation.as_ref().and_then(|p| p.bookmark(point));
            let removed =
                retained.is_none() && record.changes.iter().any(|c| c.before.contains(&point.0));
            let position = if removed {
                None
            } else {
                let mapped = match retained {
                    Some(position) => position,
                    None => mapping::mapped(&record.changes, point, mapping::Affinity::Left, None)?,
                };
                // A grapheme join can erase the original boundary; it must not be
                // replaced with a neighbouring cursor merely to keep it in range.
                coordinates.is_boundary(mapped)?.then_some(mapped)
            };
            Ok(Anchor {
                position,
                ..*anchor
            })
        })
        .collect()
}

pub(super) fn restore(
    current: &mut [Anchor],
    saved: &[Anchor],
    coordinates: &Coordinates<'_>,
) -> Result<(), Error> {
    for anchor in current {
        let Ok(index) = saved.binary_search_by_key(&anchor.id, |a| a.id) else {
            continue;
        };
        let prior = saved[index];
        if prior.generation == anchor.generation {
            if let Some(position) = prior.position {
                mapping::boundary(coordinates, position)?;
            }
            anchor.position = prior.position;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bookmark::Resolution,
        buffer::{
            Buffers,
            edit::{History, Point, Replacement},
        },
        coordinates::Scalar,
        fixture::Fixture,
    };

    fn capture(buffer: &crate::buffer::Buffer, name: &str, at: usize) {
        buffer
            .bookmarks
            .lock()
            .unwrap()
            .set(
                name,
                buffer.file().unwrap().path().into(),
                Point {
                    stamp: buffer.stamp(),
                    position: Scalar(at),
                },
                true,
                crate::bookmark::Context::capture(buffer, Scalar(at)).unwrap(),
            )
            .unwrap();
    }

    // Bookmarks join the prepared publication admission. A set/delete/new anchor
    // between prepare and publish invalidates the candidate before source changes.
    #[test]
    fn changed_admission() {
        for mutation in ["replace", "delete", "late"] {
            let f = Fixture::new();
            let mut buffers = Buffers::new().unwrap();
            let id = buffers.visit(&f.write("source", "abcdef")).unwrap();
            let buffer = buffers.get_mut(id).unwrap();
            capture(buffer, "here", 3);
            let edit = Replacement {
                source: buffer.span(0..0).unwrap(),
                text: "XX".into(),
            };
            let prepared = buffer.prepare(&[edit], &[], None).unwrap();
            match mutation {
                "replace" => capture(buffer, "here", 4),
                "late" => capture(buffer, "new", 4),
                _ => {
                    buffer.bookmarks.lock().unwrap().delete("here").unwrap();
                }
            }
            assert_eq!(
                buffer.publish(prepared, &mut []),
                Err(Error::ChangedPositions),
                "{mutation}"
            );
            assert_eq!(buffer.slice(&buffer.span(0..6).unwrap()).unwrap(), "abcdef");
            assert!(buffer.history().is_empty());
        }
    }

    // All base-space replacements contribute to one atomic anchor outcome;
    // later removals cannot be mistaken for earlier inserted coordinates.
    #[test]
    fn compound_outcomes() {
        let f = Fixture::new();
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&f.write("source", "0123456789")).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        capture(buffer, "kept", 3);
        capture(buffer, "removed", 7);
        let edits = [
            Replacement {
                source: buffer.span(1..2).unwrap(),
                text: "XX".into(),
            },
            Replacement {
                source: buffer.span(6..8).unwrap(),
                text: "".into(),
            },
        ];
        let prepared = buffer.prepare(&edits, &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
        assert_eq!(
            buffer
                .bookmarks
                .lock()
                .unwrap()
                .get("kept")
                .unwrap()
                .point()
                .position
                .0,
            4
        );
        assert_eq!(
            buffer
                .bookmarks
                .lock()
                .unwrap()
                .get("removed")
                .unwrap()
                .resolution(),
            Resolution::Stale
        );
        buffer.navigate(History::Undo, &mut [], None).unwrap();
        assert_eq!(
            buffer
                .bookmarks
                .lock()
                .unwrap()
                .get("removed")
                .unwrap()
                .point()
                .position
                .0,
            7
        );
        assert_eq!(
            buffer
                .bookmarks
                .lock()
                .unwrap()
                .get("removed")
                .unwrap()
                .resolution(),
            Resolution::Resolved
        );
        buffer.navigate(History::Redo, &mut [], None).unwrap();
        assert_eq!(
            buffer
                .bookmarks
                .lock()
                .unwrap()
                .get("removed")
                .unwrap()
                .resolution(),
            Resolution::Stale
        );
    }
}
