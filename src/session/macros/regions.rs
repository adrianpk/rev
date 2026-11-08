use super::*;
use crate::buffer::edit::{MACRO_LINE_LIMIT, Point};
use crate::coordinates::{Cell, Coordinates, Scalar};
use crate::macros::Region;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl Session {
    pub(super) fn macro_region_start(&mut self) -> Action {
        if self.macros.runner.is_some() || self.macros.recorder.is_some() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Region execution requires an idle macro or manual editing pause",
            );
        }
        if !self.macro_source() {
            return self.message("Macro region execution requires ordinary source editing");
        }
        let Some(definition) = self.macros.ring.front() else {
            return self.message("No keyboard macro is available");
        };
        if definition.entry != self.macro_entry() {
            return self.message("Keyboard macro region entry state differs");
        }
        let positions = match self.macro_region_positions() {
            Ok(positions) => positions,
            Err(error) => return self.stop_macro(StopReason::RegionLimit, &error),
        };
        let Some(&first) = positions.first() else {
            return self.message("Region contains no logical line beginnings");
        };
        let point = self.view().point;
        let buffer = self
            .buffers
            .get_mut(point.stamp.buffer)
            .expect("live selected buffer");
        let anchors = match buffer.macro_lines(positions) {
            Ok(anchors) => anchors,
            Err(error) => return self.error(&error.to_string()),
        };
        buffer.deactivate_region();
        if let Err(error) = self.macro_line_point(Point {
            position: first,
            ..point
        }) {
            return self.error(&error);
        }
        let action = self.start_macro(1);
        let waiting = self.layout_active();
        if let Some(run) = &mut self.macros.runner {
            run.region = Some(Region {
                view: self.windows.selected_id(),
                anchors,
                next: 1,
            });
            run.waiting = waiting;
        }
        action
    }

    fn macro_region_positions(&self) -> Result<Vec<Scalar>, String> {
        let buffer = self.buffer().map_err(|error| error.to_string())?;
        if buffer.rectangle_active() {
            return Err("Macro line execution requires a linear region".into());
        }
        let region = buffer
            .region(self.view().point)
            .map_err(|error| error.to_string())?
            .ok_or("Macro line execution requires an active region")?;
        let coordinates = Coordinates::new(buffer).map_err(|error| error.to_string())?;
        let first = coordinates
            .to_line(Scalar(region.range.start))
            .map_err(|error| error.to_string())?
            .row;
        let mut positions = Vec::new();
        for row in first..buffer.len_lines() {
            let position = coordinates
                .from_cell(Cell { row, column: 0 })
                .map_err(|error| error.to_string())?;
            if position.0 >= region.range.end {
                break;
            }
            if position.0 >= region.range.start {
                if positions.len() == MACRO_LINE_LIMIT {
                    return Err("Macro region exceeds 65536 original line anchors".into());
                }
                positions.push(position);
            }
        }
        Ok(positions)
    }

    pub(super) fn macro_region_next(&mut self) -> Action {
        let started = Instant::now();
        for _ in 0..256 {
            let run = self.macros.runner.as_mut().expect("region runner");
            let region = run.region.as_mut().expect("region targets");
            let anchors = region.anchors.lock().expect("macro line owner poisoned");
            if region.next == anchors.positions.len() {
                drop(anchors);
                return self.macro_run_finished();
            }
            let position = anchors.positions[region.next];
            let stamp = anchors.stamp;
            let view = region.view;
            region.next += 1;
            drop(anchors);
            if let Some(position) = position {
                if view != self.windows.selected_id() || stamp != self.view().point.stamp {
                    return self.stop_macro(
                        StopReason::OwnerChanged,
                        "Macro region source owner changed",
                    );
                }
                if self.macro_entry()
                    != self
                        .macros
                        .runner
                        .as_ref()
                        .expect("region runner")
                        .definition
                        .entry
                {
                    return self.stop_macro(
                        StopReason::EntryState,
                        "Macro region iteration did not restore its entry state",
                    );
                }
                let run = self.macros.runner.as_mut().expect("region runner");
                if run.limits.steps.load(Ordering::Relaxed) >= crate::macros::RUN_STEPS {
                    return self.stop_macro(
                        StopReason::StepBudget,
                        "Keyboard macro logical step limit reached",
                    );
                }
                if !run.limits.iteration() {
                    return self.stop_macro(
                        StopReason::IterationBudget,
                        "Keyboard macro iteration limit reached",
                    );
                }
                if let Err(error) = self.macro_line_point(Point { stamp, position }) {
                    return self.stop_macro(StopReason::OwnerChanged, &error);
                }
                let waiting = self.layout_active();
                let expected = (self.windows.selected_id(), self.view().point);
                let run = self.macros.runner.as_mut().expect("region runner");
                run.iteration += 1;
                run.step = 0;
                run.counter.iteration_start = run.counter.current;
                run.counter.format = run.definition.initial_format.clone();
                run.group = run.outer_group.clone().unwrap_or_else(|| {
                    crate::buffer::edit::AutomaticGroup::new(run.budget.clone())
                });
                run.expected = expected;
                run.waiting = waiting;
                return Action::Redraw;
            }
            if started.elapsed() >= crate::session::events::INPUT_BATCH_TIME {
                break;
            }
        }
        Action::Redraw
    }

    fn macro_line_point(&mut self, point: Point) -> Result<(), String> {
        let mut candidate = self.view().clone();
        candidate.point = point;
        candidate.reset_navigation();
        self.place_view(candidate)
            .map_err(|error| error.to_string())
    }
}
