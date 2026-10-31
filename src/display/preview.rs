//! Optional graphics never insert, replace or renumber projected source rows.
use super::*;
use std::sync::Arc;
/// An image clipped to an existing text pane, separate from source coordinates.
#[derive(Clone)]
pub struct Placement {
    pub image: Arc<crate::org::preview::Image>,
    pub area: crate::window::Area,
}
impl Frame {
    pub fn previews(&self) -> &[Placement] {
        &self.previews
    }
    pub(super) fn preview(
        &mut self,
        buffer: &Buffer,
        view: &View,
        area: Area,
    ) -> Result<(), Error> {
        let images = view.preview.images(buffer.stamp());
        if images.is_empty() {
            return Ok(());
        }
        let fragments = if buffer.soft_wrap() {
            Some(view.wrapped_rows(buffer, area.width(), area.height())?)
        } else {
            None
        };
        let mut bytes = self
            .previews
            .iter()
            .map(|p| p.image.png.bytes.len())
            .sum::<usize>();
        let mut pixels = self
            .previews
            .iter()
            .map(|p| u64::from(p.image.png.width) * u64::from(p.image.png.height))
            .sum::<u64>();
        for image in images {
            let count = u64::from(image.png.width) * u64::from(image.png.height);
            if pixels + count > 16777216 {
                break;
            }
            if self.previews.len() == 32 || bytes + image.png.bytes.len() > 16777216 {
                break;
            }
            let Some(place) = image_area(buffer, view, image.row, area, fragments.as_deref())
            else {
                continue;
            };
            bytes += image.png.bytes.len();
            pixels += count;
            self.previews.push(Placement {
                image: image.clone(),
                area: place,
            });
        }
        Ok(())
    }
}
fn image_area(
    buffer: &Buffer,
    view: &View,
    row: usize,
    area: Area,
    fragments: Option<&[crate::view::Fragment]>,
) -> Option<Area> {
    // Hidden, horizontally scrolled or off-screen anchors have no placement.
    // A picture stays behind text and cannot cover gutters, prompts or peers.
    if fragments.is_none() && view.left != 0 || area.width() == 0 || area.height() == 0 {
        return None;
    }
    let offset = if let Some(fragments) = fragments {
        fragments
            .iter()
            .position(|f| f.line == row && f.columns.start == 0)?
    } else {
        view.project_row(buffer, row)?
            .checked_sub(view.projected_top(buffer))?
    };
    let offset = u16::try_from(offset).ok()?;
    if offset >= area.height() {
        return None;
    }
    Area::new(
        area.column(),
        area.row() + offset,
        area.width().min(40),
        (area.height() - offset).min(12),
    )
}
