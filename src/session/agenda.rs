mod control;
mod frame;
use super::{Action, Session};
use crate::{
    org::{
        Cancellation,
        agenda::{
            AgendaQuery, Calendar, CalendarProjection, CalendarQuery, Navigation,
            NavigationCandidate, QueryResult, SourceScan, SourceSelection, SourceSet,
        },
    },
    window::ViewId,
};
pub(super) struct State {
    pub calendar: Calendar,
    pub view: ViewId,
    pub selection: SourceSelection,
    pub cursor: usize,
    pub top: usize,
}
pub(super) enum Work {
    ActionVisit {
        navigation: Navigation,
        view: ViewId,
        deadline: std::time::Instant,
    },
    Custom(Box<super::agenda_custom::Request>),
    Refresh {
        scan: Box<SourceScan>,
        query: CalendarQuery,
        policy: Box<crate::config::Org>,
        selection: SourceSelection,
        view: ViewId,
    },
    Query {
        scan: Box<SourceScan>,
        query: AgendaQuery,
        context: CalendarQuery,
        policy: Box<crate::config::Org>,
        selection: SourceSelection,
        view: ViewId,
    },
    Project {
        projection: CalendarProjection,
        view: ViewId,
    },
    Visit {
        navigation: Navigation,
        view: ViewId,
    },
}
pub(super) enum Output {
    Refresh {
        sources: SourceSet,
        calendar: Calendar,
        selection: SourceSelection,
        view: ViewId,
    },
    Query {
        sources: SourceSet,
        result: Box<QueryResult>,
        selection: SourceSelection,
        view: ViewId,
    },
    Project {
        calendar: Calendar,
        view: ViewId,
    },
    Visit {
        candidate: NavigationCandidate,
        view: ViewId,
    },
}
impl Work {
    pub(super) fn run(self, cancel: &Cancellation) -> Result<Output, String> {
        match self {
            Self::ActionVisit {
                navigation,
                view,
                deadline,
            } => navigation
                .run(&cancel.bounded(deadline))
                .map(|candidate| Output::Visit { candidate, view }),
            Self::Custom(request) => request.run(cancel),
            Self::Refresh {
                scan,
                query,
                policy,
                selection,
                view,
            } => {
                let bounded = cancel.bounded(
                    std::time::Instant::now()
                        + std::time::Duration::from_secs(policy.limits.job_seconds as u64),
                );
                let sources = scan.run(&bounded);
                let calendar = Calendar::observe(&sources, query, &policy, &bounded)?;
                Ok(Output::Refresh {
                    sources,
                    calendar,
                    selection,
                    view,
                })
            }
            Self::Query {
                scan,
                query,
                context,
                policy,
                selection,
                view,
            } => {
                let bounded = cancel.bounded(
                    std::time::Instant::now()
                        + std::time::Duration::from_secs(policy.limits.job_seconds as u64),
                );
                let sources = scan.run(&bounded);
                let result = QueryResult::observe(&sources, query, context, &policy, &bounded)?;
                Ok(Output::Query {
                    sources,
                    result: Box::new(result),
                    selection,
                    view,
                })
            }
            Self::Project { projection, view } => Ok(Output::Project {
                calendar: projection.run(cancel),
                view,
            }),
            Self::Visit { navigation, view } => navigation
                .run(cancel)
                .map(|candidate| Output::Visit { candidate, view }),
        }
    }
}
impl Session {
    pub fn calendar_agenda(&self) -> Option<&Calendar> {
        self.calendar_agenda.as_ref().map(|s| &s.calendar)
    }
    pub(super) fn calendar_active(&self) -> bool {
        self.calendar_agenda
            .as_ref()
            .is_some_and(|s| s.view == self.windows.selected_id())
    }
    pub(super) fn finish_agenda_sources(&mut self, result: Result<Output, String>) {
        match result {
            Ok(Output::Refresh {
                mut sources,
                mut calendar,
                selection,
                view,
            }) => {
                calendar.revalidate(&self.buffers);
                sources.revalidate(&self.buffers);
                self.message(&sources.feedback());
                self.agenda_sources = Some(sources);
                self.install_calendar(calendar, selection, view);
            }
            Ok(Output::Query {
                mut sources,
                mut result,
                selection,
                view,
            }) => {
                sources.revalidate(&self.buffers);
                result.revalidate(&self.buffers);
                self.agenda_sources = Some(sources);
                self.install_query(*result, selection, view);
            }
            Ok(Output::Project { mut calendar, view }) => {
                calendar.revalidate(&self.buffers);
                if let Some(state) = &mut self.calendar_agenda
                    && state.view == view
                {
                    state.calendar = calendar;
                    state.top = 0;
                    state.cursor = 0;
                }
                self.message("Calendar span changed; g refreshes sources");
            }
            Ok(Output::Visit { candidate, view }) => self.finish_agenda_visit(candidate, view),
            Err(error) if self.resolving_agenda_action() => self.failed_agenda_action(&error),
            Err(error) => self.agenda_error(&error),
        }
    }
    fn finish_agenda_visit(&mut self, candidate: NavigationCandidate, view: ViewId) {
        if !self.agenda_active() || self.windows.selected_id() != view {
            self.message("Agenda source return cancelled: view changed");
            return;
        }
        if self.resolving_agenda_action() {
            self.resolved_agenda_action(candidate);
            return;
        }
        if let Err(error) = self.visit_agenda_candidate(candidate) {
            self.error(&error);
        }
    }
    fn agenda_error(&mut self, error: &str) {
        if let Some(state) = &mut self.query_agenda {
            state.result.truncated = true;
        }
        if let Some(state) = &mut self.calendar_agenda {
            state.calendar.truncated = true;
        }
        self.error(error);
    }
    fn install_calendar(&mut self, calendar: Calendar, selection: SourceSelection, view: ViewId) {
        if self.windows.get(view).is_err() {
            self.message("Agenda view closed; source observations retained");
            return;
        }
        self.habit_agenda = None;
        self.query_agenda = None;
        self.calendar_agenda = Some(State {
            calendar,
            selection,
            view,
            cursor: 0,
            top: 0,
        });
        self.sync_input_context();
    }
    pub(super) fn agenda_candidate(
        &mut self,
        candidate: NavigationCandidate,
    ) -> Result<crate::buffer::edit::Point, String> {
        let visit = self
            .buffers
            .prepare_observed_visit(&candidate.path, candidate.loaded, &candidate.observed)
            .map_err(|e| e.to_string())?;
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        let position = buffer
            .byte_span(candidate.byte..candidate.byte)
            .map_err(|e| e.to_string())?
            .range
            .start;
        let point = crate::buffer::edit::Point {
            stamp: buffer.stamp(),
            position: crate::coordinates::Scalar(position),
        };
        self.buffers.publish_visit(visit);
        Ok(point)
    }
    fn visit_agenda_candidate(&mut self, candidate: NavigationCandidate) -> Result<(), String> {
        let point = self.agenda_candidate(candidate)?;
        let id = point.stamp.buffer;
        self.calendar_agenda = None;
        self.query_agenda = None;
        self.activate(id);
        self.sync_input_context();
        self.place_point(point).map_err(|e| e.to_string())?;
        self.message("Agenda source");
        Ok(())
    }
}
