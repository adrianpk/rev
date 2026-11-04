use super::State;
use crate::{
    command::{
        Navigation, Scroll,
        navigation::{Find, Target},
    },
    coordinates::{Cell, Coordinates, Found, Scalar},
    search::Direction,
    view::{Error, Size, View},
};

#[derive(Clone, Copy)]
pub(super) struct Finding {
    kind: Find,
    character: char,
    found: Found,
}

impl State {
    pub fn navigate(&mut self, request: Navigation, size: Size) -> Result<(), String> {
        let (view, finding) = self
            .navigation_view(request, size)
            .map_err(|e| e.to_string())?;
        self.view = view;
        if let Some(finding) = finding {
            self.finding = Some(finding);
        }
        Ok(())
    }
    pub(super) fn navigation_view(
        &self,
        request: Navigation,
        size: Size,
    ) -> Result<(View, Option<Finding>), Error> {
        let mut view = self.view.clone();
        let mut finding = None;
        if !self.viewport_navigation(&mut view, request, size)? {
            finding = self.find(request)?;
            view.point.position = match finding {
                Some(finding) => finding.found.point,
                None => self.position(request)?,
            };
            view.reset_navigation();
        }
        view.check_point(&self.buffer)?;
        view.reveal(&self.buffer, size)?;
        Ok((view, finding))
    }
    fn viewport_navigation(
        &self,
        view: &mut View,
        request: Navigation,
        size: Size,
    ) -> Result<bool, Error> {
        match request.target {
            Target::Line(direction) => {
                view.scroll(&self.buffer, line(direction), request.amount())?
            }
            Target::Page { backward, half } => {
                self.page(view, (backward, half), request.amount(), size)?
            }
            Target::Screen(screen) => {
                view.point.position =
                    view.screen_target(&self.buffer, size, screen, request.amount())?;
                view.reset_navigation();
            }
            Target::Align(screen) => view.align_screen(&self.buffer, size, screen)?,
            _ => return Ok(false),
        }
        Ok(true)
    }
    fn page(
        &self,
        view: &mut View,
        policy: (bool, bool),
        count: usize,
        size: Size,
    ) -> Result<(), Error> {
        let rows = usize::from(size.text_rows());
        let step = if policy.1 {
            (rows / 2).max(1)
        } else {
            rows.saturating_sub(self.page_overlap).max(1)
        };
        let distance = step
            .checked_mul(count)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let scroll = if policy.0 {
            Scroll::PageBackward
        } else {
            Scroll::PageForward
        };
        view.scroll(&self.buffer, scroll, distance)
    }
    fn position(&self, request: Navigation) -> Result<Scalar, Error> {
        let coordinates = Coordinates::new(&self.buffer)?;
        match request.target {
            Target::Buffer(direction) => self.buffer_target(direction, request.count),
            Target::LineEnd => {
                let mut view = self.view.clone();
                view.scroll(
                    &self.buffer,
                    Scroll::NextLine,
                    request.amount().saturating_sub(1),
                )?;
                Ok(coordinates.line_end(view.point.position)?)
            }
            _ => Ok(coordinates.navigation(
                self.view.point.position,
                request.target,
                request.amount(),
            )?),
        }
    }
    fn buffer_target(&self, direction: Direction, count: Option<u32>) -> Result<Scalar, Error> {
        let coordinates = Coordinates::new(&self.buffer)?;
        let Some(count) = count else {
            return Ok(if direction == Direction::Backward {
                Scalar(0)
            } else {
                coordinates.end()
            });
        };
        let row = (count.max(1) as usize - 1).min(self.buffer.len_lines() - 1);
        Ok(coordinates.from_cell(Cell { row, column: 0 })?)
    }
    fn find(&self, request: Navigation) -> Result<Option<Finding>, Error> {
        let Some((kind, character, origin)) = self.find_descriptor(request)? else {
            return Ok(None);
        };
        let found = Coordinates::new(&self.buffer)?
            .find_target(origin, kind, character, request.amount())?
            .ok_or(crate::coordinates::Error::Scalar(origin))?;
        Ok(Some(Finding {
            kind,
            character,
            found,
        }))
    }
    fn find_descriptor(&self, request: Navigation) -> Result<Option<(Find, char, Scalar)>, Error> {
        let origin = self.view.point.position;
        let descriptor = match request.target {
            Target::Find(kind) => (
                kind,
                request
                    .character
                    .ok_or(crate::coordinates::Error::Scalar(origin))?,
                origin,
            ),
            Target::RepeatFind(reverse) => {
                let previous = self
                    .finding
                    .ok_or(crate::coordinates::Error::Scalar(origin))?;
                (
                    if reverse {
                        previous.kind.reversed()
                    } else {
                        previous.kind
                    },
                    previous.character,
                    if origin == previous.found.point {
                        previous.found.matched
                    } else {
                        origin
                    },
                )
            }
            _ => return Ok(None),
        };
        Ok(Some(descriptor))
    }
}

fn line(direction: Direction) -> Scroll {
    if direction == Direction::Backward {
        Scroll::PreviousLine
    } else {
        Scroll::NextLine
    }
}
