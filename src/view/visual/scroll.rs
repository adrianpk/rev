use crate::buffer::{Buffer, edit::Point};
use crate::command::Scroll;
use crate::coordinates::{Cell, Scalar};

use super::super::{Error, Fragment, View, fold::Folds};

#[derive(Clone, PartialEq, Eq)]
pub(in crate::view) struct Origin {
    point: Point,
    top: usize,
    wrap_start: Scalar,
    column: Option<usize>,
    folds: Option<Folds>,
    scroll: Scroll,
    distance: usize,
}

/// Disposable progress over two native fragments. It is not an authoritative
/// viewport and holds no View/cache reference, so snapshots cannot form a cycle.
pub(in crate::view) struct Work {
    pub origin: Origin,
    point: Fragment,
    top: Option<Fragment>,
    column: usize,
    moved: usize,
    top_moved: usize,
    point_done: bool,
    target: Option<Scalar>,
}

#[derive(Clone, Copy)]
pub(super) struct Destination {
    point: Scalar,
    column: usize,
    top: Option<(usize, Scalar)>,
}

impl Origin {
    fn new(view: &View, scroll: Scroll, distance: usize) -> Self {
        Self {
            point: view.point,
            top: view.top,
            wrap_start: view.wrap_start,
            column: view.column,
            folds: view.folds.clone(),
            scroll,
            distance,
        }
    }
}

impl Work {
    fn new(view: &View, buffer: &Buffer, width: u16, origin: Origin) -> Result<Self, Error> {
        let point = geometry(view.visual_fragment(buffer, width)?);
        let Cell { column, .. } = view.cell(buffer)?;
        let column = view.column.unwrap_or(
            point
                .screen_column(column)
                .ok_or(crate::coordinates::Error::Overflow)?,
        );
        let top = if origin.scroll.page() {
            view.wrapped_rows(buffer, width, 1)?.pop().map(geometry)
        } else {
            None
        };
        Ok(Self {
            origin,
            point,
            top,
            column,
            moved: 0,
            top_moved: 0,
            point_done: false,
            target: None,
        })
    }

    fn advance(&mut self, view: &View, buffer: &Buffer, width: u16) -> Result<Destination, Error> {
        self.advance_point(view, buffer, width)?;
        if self.target.is_none() {
            self.target = Some(View::fragment_point(buffer, &self.point, self.column)?);
        }
        self.advance_top(view, buffer, width)?;
        Ok(Destination {
            point: self.target.expect("completed source target"),
            column: self.column,
            top: self.top.as_ref().map(|f| (f.line, f.source.start)),
        })
    }

    fn advance_point(&mut self, view: &View, buffer: &Buffer, width: u16) -> Result<(), Error> {
        while !self.point_done && self.moved < self.origin.distance {
            let Some(next) =
                view.adjacent_fragment(buffer, width, &self.point, self.origin.scroll.backward())?
            else {
                self.point_done = true;
                break;
            };
            self.point = geometry(next);
            self.moved += 1;
        }
        Ok(())
    }

    fn advance_top(&mut self, view: &View, buffer: &Buffer, width: u16) -> Result<(), Error> {
        let Some(top) = &mut self.top else {
            return Ok(());
        };
        while self.top_moved < self.moved {
            let Some(next) =
                view.adjacent_fragment(buffer, width, top, self.origin.scroll.backward())?
            else {
                self.top_moved = self.moved;
                break;
            };
            *top = geometry(next);
            self.top_moved += 1;
        }
        Ok(())
    }
}

impl View {
    /// Cache a complete pure destination as well as pending progress: native
    /// preflight and publication must not restart a source-sized page traversal.
    pub(super) fn continued_scroll(
        &mut self,
        buffer: &Buffer,
        width: u16,
        scroll: Scroll,
        distance: usize,
    ) -> Result<(), Error> {
        let origin = Origin::new(self, scroll, distance);
        let retained = self
            .wrap_cache
            .take_scroll(buffer, width)
            .filter(|work| work.origin == origin);
        let mut work = match retained {
            Some(work) => work,
            None => Work::new(self, buffer, width, origin)?,
        };
        let result = work.advance(self, buffer, width);
        if result.is_ok() || result.is_err_and(Error::pending) {
            self.wrap_cache.suspend_scroll(buffer, width, work);
        }
        let destination = result?;
        self.point.position = destination.point;
        self.column = Some(destination.column);
        if let Some((line, point)) = destination.top {
            self.top = line;
            self.wrap_start = point;
        }
        Ok(())
    }
}

// Continuations need geometry, never a retained presentation-text allocation.
fn geometry(mut fragment: Fragment) -> Fragment {
    fragment.text = String::new();
    fragment
}
