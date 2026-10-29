//! Temporarily address a canonical source while retaining the agenda's mapped view.
use super::*;
use crate::buffer::edit::Point;
impl Windows {
    pub(crate) fn source_action_active(&self) -> bool {
        self.saved.is_some()
    }

    pub(crate) fn source_action(&mut self, buffer: &Buffer, point: Point) -> Result<(), String> {
        if self.saved.is_some() {
            return Err("Another source action owns the saved view".into());
        }
        if buffer.stamp() != point.stamp {
            return Err("Agenda source changed before action".into());
        }
        let mut source = View::new(buffer);
        source.point = point;
        let owner = self.selected;
        let home = std::mem::replace(self.selected_mut(), source);
        self.saved = Some((owner, home));
        Ok(())
    }
    pub(crate) fn restore_source_action(&mut self) {
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
