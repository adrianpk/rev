use crate::buffer::{Span, Stamp, edit};
use crate::coordinates::{Coordinates, Scalar};
use crate::search::Pattern;
use crate::window::ViewId;

use super::{Action, Pending, Session};

pub(super) struct State {
    owner: Owner,
    stage: Stage,
}

struct Owner {
    view: ViewId,
    stamp: Stamp,
    anchor: Scalar,
    count: usize,
}

enum Stage {
    Query,
    Replacement(Pattern),
    Decision(Candidate),
}

struct Candidate {
    pattern: Pattern,
    replacement: String,
    source: Span,
}

impl State {
    pub(super) fn deciding(&self) -> bool {
        matches!(self.stage, Stage::Decision(_))
    }
}

impl Session {
    pub(super) fn query_replace(&mut self) -> Action {
        let buffer = match self.buffer() {
            Ok(buffer) => buffer,
            Err(error) => return self.message(&error.to_string()),
        };
        if buffer.is_read_only() {
            return self.message(&edit::Error::ReadOnly.to_string());
        }
        let owner = Owner {
            view: self.windows.selected_id(),
            stamp: buffer.stamp(),
            anchor: self.view().point.position,
            count: 0,
        };
        self.replace_prompt(
            State {
                owner,
                stage: Stage::Query,
            },
            "Query replace: ",
        )
    }

    fn replace_prompt(&mut self, state: State, label: &'static str) -> Action {
        if let Err(error) = self.input.replace_argument(label) {
            return self.message(&error.to_string());
        }
        self.pending = Some(Pending::Replace(Box::new(state)));
        self.message("")
    }

    fn replace_valid(&self, owner: &Owner) -> bool {
        owner.view == self.windows.selected_id() && owner.stamp == self.view().point.stamp
    }

    pub(super) fn invalidate_replace(&mut self) {
        if let Some(Pending::Replace(state)) = &self.pending
            && !self.replace_valid(&state.owner)
        {
            self.pending = None;
            self.input.cancel();
            self.message("Query replace source changed");
        }
    }

    pub(super) fn replace_match(&self) -> Option<&Span> {
        let Some(Pending::Replace(state)) = &self.pending else {
            return None;
        };
        if !self.replace_valid(&state.owner) {
            return None;
        }
        match &state.stage {
            Stage::Decision(candidate) => Some(&candidate.source),
            _ => None,
        }
    }

    pub(super) fn replace_argument(&mut self, state: State, text: String) -> Action {
        if !self.replace_valid(&state.owner) {
            return self.message("Query replace source changed");
        }
        match state.stage {
            Stage::Query => self.replace_query(state.owner, &text),
            Stage::Replacement(pattern) => self.replace_value(state.owner, pattern, text),
            _ => self.message("No replacement argument is pending"),
        }
    }

    fn replace_query(&mut self, owner: Owner, query: &str) -> Action {
        if query.is_empty() {
            return self.message("Query must not be empty");
        }
        let pattern = match Pattern::new(query) {
            Ok(pattern) => pattern,
            Err(error) => return self.message(&error.to_string()),
        };
        self.replace_prompt(
            State {
                owner,
                stage: Stage::Replacement(pattern),
            },
            "Replace with: ",
        )
    }

    fn replace_value(&mut self, owner: Owner, pattern: Pattern, replacement: String) -> Action {
        if replacement.contains('\0') {
            return self.message("Replacement contains NUL; binary input rejected");
        }
        self.replace_next(owner, pattern, replacement)
    }

    fn replace_next(&mut self, owner: Owner, pattern: Pattern, replacement: String) -> Action {
        let result = self.replace_find(&owner, &pattern);
        let source = match result {
            Ok(Some(source)) => source,
            Ok(None) => {
                return self.message(&format!("Query replace complete: {} replaced", owner.count));
            }
            Err(error) => return self.message(&error.to_string()),
        };
        if let Err(error) = self.input.replace_decision() {
            return self.message(&error.to_string());
        }
        let count = owner.count;
        let stage = Stage::Decision(Candidate {
            pattern,
            replacement,
            source,
        });
        self.pending = Some(Pending::Replace(Box::new(State { owner, stage })));
        self.message(&format!("Replace occurrence? [y/n/q] ({count} replaced)"))
    }

    fn replace_find(
        &mut self,
        owner: &Owner,
        pattern: &Pattern,
    ) -> Result<Option<Span>, crate::search::Error> {
        let buffer = self.buffers.get(owner.stamp.buffer)?;
        let Some(source) = pattern.find(buffer, owner.anchor)? else {
            return Ok(None);
        };
        let coordinates = Coordinates::new(buffer)?;
        let mut point = Scalar(source.range.end);
        if !coordinates.is_boundary(point)? {
            point = coordinates.next(point)?;
        }
        let view = self.windows.selected_mut();
        view.point.position = point;
        view.reset_navigation();
        Ok(Some(source))
    }

    pub(super) fn replace_answer(&mut self, state: State, accepted: bool) -> Action {
        if !self.replace_valid(&state.owner) {
            return self.message("Query replace source changed");
        }
        let Stage::Decision(candidate) = state.stage else {
            return self.message("No replacement decision is pending");
        };
        let mut owner = state.owner;
        if let Err(error) = self.replace_apply(&mut owner, &candidate, accepted) {
            return self.message(&error.to_string());
        }
        self.replace_next(owner, candidate.pattern, candidate.replacement)
    }

    fn replace_apply(
        &mut self,
        owner: &mut Owner,
        candidate: &Candidate,
        accepted: bool,
    ) -> Result<(), edit::Error> {
        self.buffer()?.slice(&candidate.source)?;
        owner.anchor = Scalar(candidate.source.range.end);
        if accepted {
            let end = candidate
                .source
                .range
                .start
                .checked_add(candidate.replacement.chars().count())
                .ok_or(edit::Error::Capacity)?;
            let count = owner.count.checked_add(1).ok_or(edit::Error::Capacity)?;
            self.replace(candidate.source.range.clone(), &candidate.replacement)?;
            owner.anchor = Scalar(end);
            owner.count = count;
            owner.stamp = self.buffer()?.stamp();
        }
        Ok(())
    }
}
