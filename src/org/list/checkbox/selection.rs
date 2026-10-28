use super::super::{Lists, progress::State};
use crate::buffer::{Buffer, edit::Point};
use crate::command::OrgCommand;
use crate::coordinates::{Coordinates, Scalar};

impl Lists {
    pub(super) fn checkbox_selection(
        &self,
        buffer: &Buffer,
        command: OrgCommand,
        point: Scalar,
    ) -> Result<Vec<usize>, String> {
        if buffer.rectangle_active() {
            return Err("Org checkbox operations require a linear region".into());
        }
        if let Some(selected) = self.region_selection(buffer, point)? {
            return Ok(selected);
        }
        let (byte, eof_line) = super::super::operation::location(buffer, point)?;
        if let Some(index) = self.at(byte, eof_line) {
            return Ok(vec![index]);
        }
        let heading = self
            .heading_at(byte)
            .filter(|index| byte <= self.headings[*index].title.end)
            .ok_or("No Org checkbox or statistics heading here")?;
        if command == OrgCommand::UpdateStatisticsCookies {
            return Ok(Vec::new());
        }
        Ok(self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                self.heading_at(item.source.start) == Some(heading) && item.checkbox.is_some()
            })
            .map(|(index, _)| index)
            .collect())
    }

    fn region_selection(
        &self,
        buffer: &Buffer,
        point: Scalar,
    ) -> Result<Option<Vec<usize>>, String> {
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let Some(region) = buffer
            .region(Point {
                stamp: buffer.stamp(),
                position: point,
            })
            .map_err(|e| e.to_string())?
        else {
            return Ok(None);
        };
        let start = coordinates
            .to_byte(Scalar(region.range.start))
            .map_err(|e| e.to_string())?
            .0;
        let end = coordinates
            .to_byte(Scalar(region.range.end))
            .map_err(|e| e.to_string())?
            .0;
        Ok(Some(
            self.items
                .iter()
                .enumerate()
                .filter(|(_, item)| start <= item.source.start && item.source.start < end)
                .map(|(index, _)| index)
                .collect(),
        ))
    }

    pub(super) fn admit_ordered(&self, state: &State) -> Result<(), String> {
        for list in &self.lists {
            let heading = list
                .items
                .first()
                .and_then(|index| self.heading_at(self.items[*index].source.start));
            if !state.affected.contains(&heading)
                || !heading.is_some_and(|index| self.headings[index].ordered)
            {
                continue;
            }
            let mut incomplete = false;
            for index in &list.items {
                if state.values[*index] == Some(b'X')
                    && incomplete
                    && self.items[*index]
                        .checkbox
                        .as_ref()
                        .is_some_and(|entry| entry.1 != b'X')
                {
                    return Err("Earlier Org checkbox is incomplete in this ORDERED entry".into());
                }
                incomplete |= state.values[*index].is_some_and(|value| value != b'X');
            }
        }
        Ok(())
    }
}
