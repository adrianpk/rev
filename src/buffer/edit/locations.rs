use super::{Error, Transaction, mapping};
use crate::{coordinates::Coordinates, location::Snapshot};

pub(super) fn map(
    record: &Transaction,
    coordinates: &Coordinates<'_>,
) -> Result<Vec<Snapshot>, Error> {
    record
        .before_locations
        .iter()
        .map(|anchor| {
            let position = anchor
                .position
                .map(|point| mapping::anchor(record, coordinates, point))
                .transpose()?
                .flatten();
            Ok(Snapshot {
                position,
                ..*anchor
            })
        })
        .collect()
}

pub(super) fn restore(
    current: &mut [Snapshot],
    saved: &[Snapshot],
    coordinates: &Coordinates<'_>,
) -> Result<(), Error> {
    for anchor in current {
        if let Ok(index) = saved.binary_search_by_key(&anchor.id, |saved| saved.id) {
            let position = saved[index].position;
            if let Some(position) = position {
                mapping::boundary(coordinates, position)?;
            }
            anchor.position = position;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        buffer::{
            Buffers,
            edit::{Error, Point, Replacement},
        },
        coordinates::Scalar,
    };

    // A new live location between prepare and publish invalidates the entire
    // transaction before any source, point, history or anchor can be published.
    #[test]
    fn location_admission() {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("abcdef").unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let prepared = buffer
            .prepare(
                &[Replacement {
                    source: buffer.span(0..0).unwrap(),
                    text: "X".into(),
                }],
                &[],
                None,
            )
            .unwrap();
        let _location = buffer
            .locations
            .lock()
            .unwrap()
            .capture(
                buffer,
                Point {
                    stamp: buffer.stamp(),
                    position: Scalar(3),
                },
            )
            .unwrap();
        assert_eq!(
            buffer.publish(prepared, &mut []),
            Err(Error::ChangedPositions)
        );
        assert_eq!(buffer.slice(&buffer.span(0..6).unwrap()).unwrap(), "abcdef");
        assert!(buffer.history().is_empty());
    }
}
