use super::{Error, Layout, ViewId, Windows};

impl Windows {
    /// Editing leaves in first-child-before-second-child screen order. Pane and
    /// prompt focus outside the editing layout cannot become traversal members.
    pub fn order(&self) -> Vec<ViewId> {
        let mut order = Vec::with_capacity(self.views.len());
        self.layout.leaves(&mut order);
        order
    }

    /// Geometry-independent traversal remains valid while presentation suspends.
    /// The sole view cycles to itself; the final leaf wraps to the first one.
    pub fn next_view(&self) -> Result<ViewId, Error> {
        let order = self.order();
        let index = order
            .iter()
            .position(|id| *id == self.selected)
            .ok_or(Error::Missing(self.selected))?;
        let next = order[(index + 1) % order.len()];
        self.get(next)?;
        Ok(next)
    }

    /// Remove only a view. Its sibling replaces the parent division, and that
    /// sibling's first leaf becomes selected. Buffer lifetime is independent.
    pub fn delete_selected(&mut self) -> Result<ViewId, Error> {
        if self.views.len() == 1 {
            return Err(Error::OnlyWindow);
        }
        let mut candidate = self.layout.clone();
        let replacement = candidate
            .remove(self.selected)
            .ok_or(Error::Missing(self.selected))?;
        self.get(replacement)?;
        self.views
            .remove(&self.selected)
            .ok_or(Error::Missing(self.selected))?;
        self.layout = candidate;
        self.selected = replacement;
        Ok(replacement)
    }

    /// Retain the selected view unchanged, releasing every other editing view.
    /// This does not read, mutate or remove any canonical buffer.
    pub fn delete_others(&mut self) -> usize {
        let removed = self.views.len() - 1;
        self.views.retain(|id, _| *id == self.selected);
        self.layout = Layout::Leaf(self.selected);
        removed
    }
}
