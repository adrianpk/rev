use super::*;
use crate::bookmark::{
    Context, Record, Resolution,
    context::{Progress, Search},
};
use crate::buffer::{Stamp, Visit};
use crate::coordinates::Scalar;
use crate::view::View;

#[derive(Clone, Copy)]
pub(super) enum Origin {
    Name,
    List(u64),
    Ordered(u64),
}
pub(in crate::session) struct Navigation {
    from: Origin,
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
        self.visit_bookmark_from(name, Origin::Name)
    }
    pub(super) fn visit_bookmark_from(&mut self, name: &str, from: Origin) -> Result<(), String> {
        let record = self
            .buffers
            .bookmarks()
            .get(name)
            .cloned()
            .ok_or("No bookmark with that name")?;
        let visit = self.prepare_bookmark_visit(&record)?;
        let navigation = self.bookmark_navigation(record, visit, from)?;
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

    fn bookmark_navigation(
        &self,
        record: Record,
        visit: Visit,
        from: Origin,
    ) -> Result<Navigation, String> {
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        let (position, live) = observed_position(&record, buffer)?;
        let search = if position.is_none() {
            Some(Search::new(
                record.context().clone(),
                buffer.len_chars(),
                self.buffers.snapshot().settings().bookmarks.search_chars,
            )?)
        } else {
            None
        };
        let target = buffer.stamp();
        let mut view = self.view().clone();
        if view.point.stamp.buffer != buffer.id() {
            view.rebind(buffer);
        }
        Ok(Navigation {
            from,
            record,
            visit,
            target,
            origin: self.windows.selected_id(),
            point: self.view().point,
            position,
            search,
            view,
            contextual: !live,
        })
    }

    fn prepare_bookmark_visit(&mut self, record: &Record) -> Result<Visit, String> {
        Ok(match self.buffers.prepare_bookmark_visit(record.path()) {
            Ok(visit) => visit,
            Err(error) => {
                self.buffers
                    .bookmarks_mut()
                    .unavailable(record.id(), record.generation());
                return Err(format!("Bookmark target is unavailable: {error}"));
            }
        })
    }

    fn step_bookmark_navigation(
        &mut self,
        mut work: Navigation,
    ) -> Result<Option<Navigation>, String> {
        self.validate_bookmark_work(&work)?;
        if !self.resolve_bookmark_search(&mut work)? {
            return Ok(Some(work));
        }
        if !self.reveal_bookmark_target(&mut work)? {
            return Ok(Some(work));
        }
        self.finish_bookmark_navigation(work)?;
        Ok(None)
    }

    fn validate_bookmark_work(&self, work: &Navigation) -> Result<(), String> {
        if matches!(work.from, Origin::List(generation) | Origin::Ordered(generation) if generation != self.bookmarks().generation())
        {
            return Err("Bookmark collection changed during navigation; retry required".into());
        }
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
        Ok(())
    }

    fn resolve_bookmark_search(&self, work: &mut Navigation) -> Result<bool, String> {
        let buffer = work
            .visit
            .buffer(&self.buffers)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != work.target {
            return Err("Bookmark source revision changed during resolution".into());
        }
        if let Some(search) = &mut work.search {
            match search.step(buffer)? {
                Progress::Pending => return Ok(false),
                Progress::Found(position) => {
                    work.position = Some(position);
                    work.search = None;
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn reveal_bookmark_target(&self, work: &mut Navigation) -> Result<bool, String> {
        let buffer = work
            .visit
            .buffer(&self.buffers)
            .map_err(|e| e.to_string())?;
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
                return Ok(false);
            }
            return Err(error.to_string());
        }
        Ok(true)
    }

    fn finish_bookmark_navigation(&mut self, work: Navigation) -> Result<(), String> {
        let buffer = work
            .visit
            .buffer(&self.buffers)
            .map_err(|e| e.to_string())?;
        let file = buffer
            .file()
            .ok_or("Bookmark destination is not file-backed")?;
        verify_bookmark_file(&work, file)?;
        let tree = self
            .tree
            .opening_file(file.path())
            .map_err(|e| e.to_string())?;
        let context = Context::capture_with(
            buffer,
            work.view.point.position,
            self.buffers.snapshot().settings().bookmarks.context_chars,
        )
        .map_err(|e| e.to_string())?;
        let path = file.path().to_owned();
        self.stop_typing().map_err(|e| e.to_string())?;
        self.publish_bookmark_navigation(work, context, tree, path)
    }

    fn publish_bookmark_navigation(
        &mut self,
        work: Navigation,
        context: Context,
        tree: Option<crate::tree::Tree>,
        path: std::path::PathBuf,
    ) -> Result<(), String> {
        let buffer = work
            .visit
            .buffer(&self.buffers)
            .map_err(|e| e.to_string())?;
        let jump = self
            .prepare_jump_to(buffer, work.view.point)
            .map_err(|e| e.to_string())?;
        // All source, context, geometry and file observations are admitted before
        // publishing the collection binding, canonical visit and destination view.
        self.buffers.bookmarks_mut().resolved(
            work.record.id(),
            work.record.generation(),
            crate::bookmark::Captured::new(work.view.point, context),
            path,
        )?;
        self.buffers.publish_visit(work.visit);
        if matches!(work.from, Origin::List(_)) {
            self.bookmark_list = None;
        }
        *self.windows.selected_mut() = work.view;
        self.publish_jump(jump);
        if let Some(tree) = tree {
            self.tree = tree;
        }
        self.tree.edit();
        self.sync_input_context();
        if matches!(work.from, Origin::Ordered(_)) {
            self.remember_bookmark_iteration(work.record.id());
            self.message(&format!("Bookmark {:?}", work.record.name()));
            return Ok(());
        }
        self.message(if work.contextual {
            "Bookmark resolved by checked context"
        } else {
            "Bookmark destination"
        });
        Ok(())
    }

    pub(in crate::session) fn expire_bookmark_navigation(&mut self) -> Action {
        let Some(work) = self.bookmark_visit.take() else {
            return Action::Wait;
        };
        let from = work.from;
        let origin = work.origin;
        let point = work.point;
        let name = work.record.name().to_owned();
        match self.step_bookmark_navigation(work) {
            Ok(pending) => {
                self.bookmark_visit = pending;
                Action::Redraw
            }
            Err(error)
                if matches!(from, Origin::Name)
                    && self.windows.selected_id() == origin
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

fn observed_position(
    record: &Record,
    buffer: &crate::buffer::Buffer,
) -> Result<(Option<Scalar>, bool), String> {
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
    Ok((position, live))
}

fn verify_bookmark_file(work: &Navigation, file: &crate::file::Location) -> Result<(), String> {
    if matches!(work.visit, Visit::New(..)) {
        let current = crate::file::resolve(work.record.path()).map_err(|e| e.to_string())?;
        if !current.is_present()
            || current.path() != file.path()
            || current.observation() != file.observation()
        {
            return Err("Bookmark file changed during resolution; retry required".into());
        }
    }
    Ok(())
}
