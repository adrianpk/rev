use super::*;
use crate::bookmark::{
    Context, Record, Resolution,
    context::{Progress, SEARCH_CHARS, Search},
};
use crate::buffer::{Stamp, Visit};
use crate::coordinates::Scalar;
use crate::view::View;

pub(in crate::session) struct Navigation {
    record: Record,
    visit: Visit,
    target: Stamp,
    origin: ViewId,
    point: Point,
    position: Option<Scalar>,
    search: Option<Search>,
    view: View,
    contextual: bool,
}

impl Session {
    pub(in crate::session) fn visit_bookmark(&mut self, name: &str) -> Result<(), String> {
        let record = self
            .buffers
            .bookmarks()
            .get(name)
            .cloned()
            .ok_or("No bookmark with that name")?;
        let visit = match self.buffers.prepare_bookmark_visit(record.path()) {
            Ok(visit) => visit,
            Err(error) => {
                self.buffers
                    .bookmarks_mut()
                    .unavailable(record.id(), record.generation());
                return Err(format!("Bookmark target is unavailable: {error}"));
            }
        };
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        if record.resolution() == Resolution::Stale && record.point().stamp.buffer == buffer.id() {
            return Err(
                "Bookmark target is stale; source undo or an explicit set can restore it".into(),
            );
        }
        let live =
            record.resolution() == Resolution::Resolved && record.point().stamp == buffer.stamp();
        let position = if live {
            Some(record.point().position)
        } else if record
            .context()
            .matches(buffer, record.context().position)?
        {
            Some(Scalar(record.context().position))
        } else {
            None
        };
        let search = if position.is_none() {
            Some(Search::new(
                record.context().clone(),
                buffer.len_chars(),
                SEARCH_CHARS,
            )?)
        } else {
            None
        };
        let target = buffer.stamp();
        let mut view = self.view().clone();
        if view.point.stamp.buffer != buffer.id() {
            view.rebind(buffer);
        }
        let navigation = Navigation {
            record,
            visit,
            target,
            origin: self.windows.selected_id(),
            point: self.view().point,
            position,
            search,
            view,
            contextual: !live,
        };
        self.bookmark_visit = if navigation.search.is_some() {
            // Original-position checking and a search batch belong to separate
            // passes so the resolver's scalar budget is not spent twice.
            Some(navigation)
        } else {
            self.step_bookmark_navigation(navigation)?
        };
        if self.bookmark_visit.is_some() {
            self.message("Checking bookmark context; C-g cancels");
        }
        Ok(())
    }

    fn step_bookmark_navigation(
        &mut self,
        mut work: Navigation,
    ) -> Result<Option<Navigation>, String> {
        if work.origin != self.windows.selected_id()
            || work.point != self.view().point
            || !self
                .buffers
                .bookmarks()
                .get(work.record.name())
                .is_some_and(|r| r == &work.record)
        {
            return Err("Bookmark navigation changed; destination was not published".into());
        }
        let buffer = work
            .visit
            .buffer(&self.buffers)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != work.target {
            return Err("Bookmark source revision changed during resolution".into());
        }
        if let Some(search) = &mut work.search {
            match search.step(buffer)? {
                Progress::Pending => return Ok(Some(work)),
                Progress::Found(position) => {
                    work.position = Some(position);
                    work.search = None;
                    return Ok(Some(work));
                }
            }
        }
        let position = work.position.ok_or("Bookmark has no resolved boundary")?;
        work.view.point = Point {
            stamp: buffer.stamp(),
            position,
        };
        work.view.reset_navigation();
        let budget = std::sync::Arc::new(std::sync::Mutex::new(crate::view::LayoutBudget::new()));
        let coordinates = crate::coordinates::Scope::new(buffer, budget);
        let fragments = work.view.layout_scope(coordinates.work());
        let result = work
            .view
            .check_point(buffer)
            .and_then(|()| work.view.reveal(buffer, self.pane_size()));
        drop(fragments);
        drop(coordinates);
        if let Err(error) = result {
            if error.pending() {
                return Ok(Some(work));
            }
            return Err(error.to_string());
        }
        let file = buffer
            .file()
            .ok_or("Bookmark destination is not file-backed")?;
        if matches!(work.visit, Visit::New(..)) {
            let current = crate::file::resolve(work.record.path()).map_err(|e| e.to_string())?;
            if !current.is_present()
                || current.path() != file.path()
                || current.observation() != file.observation()
            {
                return Err("Bookmark file changed during resolution; retry required".into());
            }
        }
        let tree = self
            .tree
            .opening_file(file.path())
            .map_err(|e| e.to_string())?;
        let context = Context::capture(buffer, position).map_err(|e| e.to_string())?;
        let path = file.path().to_owned();
        self.stop_typing().map_err(|e| e.to_string())?;
        // All source, context, geometry and file observations are admitted before
        // publishing the collection binding, canonical visit and destination view.
        self.buffers.bookmarks_mut().resolved(
            work.record.id(),
            work.record.generation(),
            work.view.point,
            context,
            path,
        )?;
        self.buffers.publish_visit(work.visit);
        *self.windows.selected_mut() = work.view;
        if let Some(tree) = tree {
            self.tree = tree;
        }
        self.tree.edit();
        self.sync_input_context();
        self.message(if work.contextual {
            "Bookmark resolved by checked context"
        } else {
            "Bookmark destination"
        });
        Ok(None)
    }

    pub(in crate::session) fn expire_bookmark_navigation(&mut self) -> Action {
        let Some(work) = self.bookmark_visit.take() else {
            return Action::Wait;
        };
        let origin = work.origin;
        let point = work.point;
        let name = work.record.name().to_owned();
        match self.step_bookmark_navigation(work) {
            Ok(pending) => {
                self.bookmark_visit = pending;
                Action::Redraw
            }
            Err(error)
                if self.windows.selected_id() == origin
                    && self.view().point == point
                    && self.input.focus() == crate::input::Focus::Editing
                    && self.pending.is_none() =>
            {
                self.bookmark_prompt(
                    Prompt {
                        command: BookmarkCommand::Jump,
                        view: origin,
                        point,
                        default: String::new(),
                    },
                    &name,
                    &error,
                )
            }
            Err(error) => self.error(&error),
        }
    }
}
