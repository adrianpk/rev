use alacritty_terminal::index::Point;

/// Retained-output navigation for one attachment. It owns no child or text;
/// rendering clamps its offset to the admitted grid history.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Viewport {
    pub offset: usize,
    pub selection: Option<Selection>,
}

/// A bounded local cell selection, independent of another mirrored view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Point,
    pub end: Point,
}

impl Selection {
    pub fn contains(self, point: Point) -> bool {
        let anchor = (self.anchor.line.0, self.anchor.column.0);
        let end = (self.end.line.0, self.end.column.0);
        let point = (point.line.0, point.column.0);
        (anchor.min(end)..=anchor.max(end)).contains(&point)
    }
}

impl Viewport {
    pub(crate) fn select(&mut self, point: Point, extend: bool) {
        let anchor = self
            .selection
            .filter(|_| extend)
            .map_or(point, |s| s.anchor);
        self.selection = Some(Selection { anchor, end: point });
    }
}
