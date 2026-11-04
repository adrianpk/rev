use crate::{
    buffer::{Buffer, Span},
    coordinates::Scalar,
    search::{Direction, Pattern, Request},
    view::View,
};

pub(super) struct Search {
    pub origin: View,
    pub found: Option<Span>,
    direction: Direction,
    anchor: Scalar,
    query: String,
    pub count: usize,
    pub operator: bool,
}
impl Search {
    pub fn new(view: &View, direction: Direction) -> Self {
        Self {
            origin: view.clone(),
            found: None,
            direction,
            anchor: view.point.position,
            query: String::new(),
            count: 1,
            operator: false,
        }
    }
    pub fn repeat(&mut self, direction: Direction) {
        if let Some(found) = &self.found {
            self.anchor = Scalar(if direction == Direction::Forward {
                found.range.end
            } else {
                found.range.start
            });
        }
        self.direction = direction;
    }
    pub fn update(&mut self, buffer: &Buffer, query: &str) -> Result<Option<Scalar>, String> {
        if query.is_empty() {
            self.found = None;
            self.query.clear();
            return Ok(Some(self.origin.point.position));
        }
        let pattern = Pattern::new(query).map_err(|e| e.to_string())?;
        self.query = query.to_owned();
        let mut request = Request::new(self.anchor, self.direction);
        request.count = self.count;
        self.found = pattern
            .occurrence(buffer, request)
            .map_err(|e| e.to_string())?;
        let Some(found) = &self.found else {
            return Ok(None);
        };
        super::query::point(buffer, found, self.direction).map(Some)
    }
    pub fn confirmed(&self) -> Option<super::query::Query> {
        (!self.query.is_empty()).then(|| super::query::Query {
            text: self.query.clone(),
            direction: self.direction,
            whole_word: false,
            found: self.found.clone(),
        })
    }
    pub fn current(&self, query: &str) -> bool {
        self.query == query
    }
}

use super::{Action, Key, Session, State, query::Query};
use crate::{command::navigation::Search as NavigationSearch, view::Size};

impl State {
    fn resolve_query(
        &self,
        search: NavigationSearch,
        count: usize,
    ) -> Result<(Query, Option<Span>, Direction), String> {
        let query = if matches!(
            search,
            NavigationSearch::WordForward | NavigationSearch::WordBackward
        ) {
            Query::word(&self.buffer, self.view.point.position, search)?
        } else {
            self.last_query
                .clone()
                .ok_or("No confirmed terminal copy search")?
        };
        let (found, direction) =
            query.find(&self.buffer, self.view.point.position, search, count)?;
        Ok((query, found, direction))
    }
    pub(super) fn search_destination(
        &self,
        search: NavigationSearch,
        count: usize,
    ) -> Result<Scalar, String> {
        let (_, found, direction) = self.resolve_query(search, count)?;
        super::query::point(
            &self.buffer,
            &found.ok_or("No match in terminal copy output")?,
            direction,
        )
    }
    fn navigate_search(
        &mut self,
        search: NavigationSearch,
        count: usize,
        size: Size,
    ) -> Result<(), String> {
        let (mut query, found, direction) = self.resolve_query(search, count)?;
        self.last_query = Some(query.clone());
        let found = found.ok_or("No match in terminal copy output")?;
        let mut candidate = self.view.clone();
        candidate.point.position = super::query::point(&self.buffer, &found, direction)?;
        candidate.reset_navigation();
        candidate
            .reveal(&self.buffer, size)
            .map_err(|e| e.to_string())?;
        query.found = Some(found);
        self.last_query = Some(query);
        self.view = candidate;
        Ok(())
    }
}

impl Session {
    pub(super) fn navigate_copy_search(
        &mut self,
        key: Key,
        search: NavigationSearch,
        count: usize,
    ) -> Action {
        if matches!(
            search,
            NavigationSearch::Forward | NavigationSearch::Backward
        ) {
            let direction = if search == NavigationSearch::Forward {
                Direction::Forward
            } else {
                Direction::Backward
            };
            let action = self.start_terminal_copy_search(direction);
            if let Some(state) = self
                .terminals
                .copies
                .get_mut(&key)
                .and_then(|copy| copy.search.as_mut())
            {
                state.count = count;
            }
            return action;
        }
        let Some(size) = self.copy_size(key) else {
            return self.error("Terminal copy presentation is suspended");
        };
        let copy = self
            .terminals
            .copies
            .get_mut(&key)
            .expect("copy search navigation");
        let result = copy.navigate_search(search, count, size);
        self.copy_result(result)
    }
    pub(super) fn finish_copy_search(&mut self, key: Key) -> Action {
        let entry = match self.prepare_copy_search(key) {
            Ok(entry) => entry,
            Err(error) => return self.error(&error),
        };
        let copy = self
            .terminals
            .copies
            .get_mut(&key)
            .expect("copy search context");
        let search = copy.search.take().expect("active copy search");
        if let Some(query) = search.confirmed() {
            copy.last_query = Some(query);
        }
        if search.operator {
            copy.view = search.origin;
        }
        self.input.cancel();
        if let Some(entry) = entry {
            self.kills.publish_transfer(entry);
            return self.message("Terminal search target copied; copy viewport retained");
        }
        self.message("Terminal copy search accepted")
    }
    fn prepare_copy_search(&self, key: Key) -> Result<Option<crate::kill::Entry>, String> {
        let copy = self
            .terminals
            .copies
            .get(&key)
            .expect("copy search context");
        let search = copy.search.as_ref().expect("active copy search");
        if !self
            .input
            .prompt()
            .is_some_and(|prompt| search.current(prompt.text()))
        {
            return Err("Current terminal copy query is invalid".into());
        }
        if !search.operator {
            return Ok(None);
        }
        if search.found.is_none() {
            return Err("No match for terminal copy operator".into());
        }
        let start = search.origin.point.position.0;
        let end = copy.view.point.position.0;
        let span = copy
            .buffer
            .span(start.min(end)..start.max(end))
            .map_err(|e| e.to_string())?;
        self.kills
            .prepare_transfer(
                &copy.buffer,
                &span,
                crate::kill::Transfer {
                    view: self.windows.selected_id(),
                    backward: end < start,
                    shape: crate::kill::Shape::Linear,
                },
            )
            .map(Some)
            .map_err(|e| e.to_string())
    }
}
