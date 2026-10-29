//! Retain a mapped home view while an owned interaction addresses other text.
use super::*;
use crate::buffer::edit::Point;
impl Windows {
    pub(crate) fn retained_view_active(&self) -> bool {
        self.saved.is_some()
    }

    pub(crate) fn retain_view(&mut self, buffer: &Buffer, point: Point) -> Result<(), String> {
        if self.saved.is_some() {
            return Err("Another interaction owns the retained view".into());
        }
        if buffer.stamp() != point.stamp {
            return Err("Addressed buffer changed before retaining its view".into());
        }
        let mut source = View::new(buffer);
        source.point = point;
        let owner = self.selected;
        let home = std::mem::replace(self.selected_mut(), source);
        self.saved = Some((owner, home));
        Ok(())
    }
    pub(crate) fn restore_view(&mut self) {
        if let Some((owner, home)) = self.saved.take()
            && let Some(view) = self.views.get_mut(&owner)
        {
            *view = home;
        }
    }
    pub(crate) fn pin_source_action(&mut self, point: Point) -> Result<usize, String> {
        if self.anchors.len() >= 256 {
            return Err("Agenda actions admit at most 256 distinct source targets".into());
        }
        let index = self.anchors.len();
        self.anchors.push(point);
        Ok(index)
    }
    pub(crate) fn source_anchor(&self, index: usize) -> Option<Point> {
        self.anchors.get(index).copied()
    }
    pub(crate) fn clear_source_anchors(&mut self) {
        self.anchors.clear();
    }
}
