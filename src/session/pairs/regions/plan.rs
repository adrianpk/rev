use crate::buffer::edit::{Cursor, Error, Replacement};
use crate::coordinates::Scalar;
use crate::session::Session;

use super::{Plan, ordinary_region};

impl Session {
    pub(super) fn publish_region_plan(&mut self, plan: Plan) -> Result<bool, Error> {
        self.stop_typing()?;
        self.finish_repeat_entry();
        self.cancel_pointer_capture();
        self.yank = None;
        self.kills.break_chain();
        let (edits, cursor) = self.region_replacements(plan)?;
        let id = self.view().point.stamp.buffer;
        let b = self.buffers.get_mut(id)?;
        let points = self.windows.points(b)?;
        let cursor = cursor.map(|position| Cursor {
            index: points.initiator,
            position,
        });
        let mut prepared = b.prepare(&edits, &points.values, cursor)?;
        prepared.retain_initiator(points.initiator)?;
        if ordinary_region(b) {
            prepared.retain_selection();
        }
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(b, prepared, points)?;
        self.published_repeat_boundary();
        self.windows.selected_mut().reset_navigation();
        Ok(true)
    }

    pub(super) fn admit_region_boundaries(&self, plan: &Plan) -> Result<(), Error> {
        let Plan::Unwrap(pair) = plan else {
            return Ok(());
        };
        let coordinates = crate::coordinates::Coordinates::new(self.buffer()?)?;
        for at in [
            pair.opening.range.start,
            pair.opening.range.end,
            pair.closing.range.start,
            pair.closing.range.end,
        ] {
            if !coordinates.is_boundary(Scalar(at))? {
                return Err(crate::coordinates::Error::Cluster(Scalar(at)).into());
            }
        }
        Ok(())
    }

    fn region_replacements(&self, plan: Plan) -> Result<(Vec<Replacement>, Option<Scalar>), Error> {
        let b = self.buffer()?;
        match plan {
            Plan::Wrap(input, closing) => {
                let (open, close) = input.kind.characters();
                let first = input.start.0;
                let last = input.end.0;
                let at = if closing {
                    last.checked_add(input.count * 2)
                } else {
                    first.checked_add(input.count)
                }
                .ok_or(Error::Capacity)?;
                Ok((
                    vec![
                        Replacement {
                            source: b.span(first..first)?,
                            text: open.to_string().repeat(input.count),
                        },
                        Replacement {
                            source: b.span(last..last)?,
                            text: close.to_string().repeat(input.count),
                        },
                    ],
                    Some(Scalar(at)),
                ))
            }
            Plan::Unwrap(pair) => Ok((
                vec![
                    Replacement {
                        source: pair.opening,
                        text: String::new(),
                    },
                    Replacement {
                        source: pair.closing,
                        text: String::new(),
                    },
                ],
                None,
            )),
            Plan::Literal(..) | Plan::Refused => {
                unreachable!("literal entry keeps its native owner")
            }
        }
    }
}
