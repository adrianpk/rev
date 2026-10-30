//! Explicit serial imports. Each feed retains its own save outcome and input.
mod publication;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand as C},
    file::Publication,
    org::feed::{self, Choice, Plan},
};
use std::{collections::VecDeque, path::PathBuf};
#[derive(Clone, Debug)]
pub struct FeedOutcome {
    pub name: String,
    pub path: PathBuf,
    pub added: usize,
    pub skipped: usize,
    pub publication: Option<Publication>,
    pub detail: String,
}
pub(super) enum Prompt {
    Select(C),
    Confirm,
    Result,
}
pub(super) struct State {
    request: Request,
    queue: VecDeque<String>,
    active: Option<Choice>,
    work: Option<feed::Work>,
    inbox: bool,
    cancelled: bool,
}
impl Session {
    pub fn feed_results(&self) -> &[FeedOutcome] {
        &self.feed_results
    }
    pub(super) fn feed_command(&mut self, command: Command) -> Option<Action> {
        if command == Command::KeyboardQuit
            && (self.feeds.is_some() || matches!(self.pending, Some(Pending::Feed(_))))
        {
            return Some(self.cancel_feed());
        }
        if self.feeds.is_some() {
            return self.feed_active_command(command);
        }
        match command {
            Command::Org(C::FeedUpdateAll | C::FeedUpdate | C::FeedGotoInbox) => {
                let Command::Org(command) = command else {
                    unreachable!()
                };
                Some(match self.begin_feed(command) {
                    Ok(()) => Action::Redraw,
                    Err(e) => self.error(&e),
                })
            }
            Command::Org(C::FeedResults) => Some(self.feed_result_prompt()),
            _ => None,
        }
    }
    fn feed_active_command(&mut self, command: Command) -> Option<Action> {
        if matches!(command, Command::Motion(_) | Command::Scroll(_)) {
            None
        } else {
            Some(self.message("Feed import owns input; C-g cancels"))
        }
    }
    fn begin_feed(&mut self, command: C) -> Result<(), String> {
        if self.org.busy() || self.pending.is_some() || self.windows.retained_view_active() {
            return Err("Another Org interaction owns input; C-g cancels".into());
        }
        if self.input.focus() != crate::input::Focus::Editing || self.tree.focused() {
            return Err("Editing owns feed commands".into());
        }
        let names: Vec<_> = self
            .buffers
            .snapshot()
            .settings()
            .org
            .feeds
            .sources
            .iter()
            .map(|f| f.name.clone())
            .collect();
        if names.is_empty() {
            return Err("No feed sources configured; set org.feeds.sources".into());
        }
        if command == C::FeedUpdateAll {
            return self.select_feeds(names, command);
        }
        self.input
            .argument(&format!("Feed ({}): ", names.join(", ")), "")
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Feed(Prompt::Select(command)));
        self.message("Select a declared feed; C-g cancels");
        Ok(())
    }
    fn select_feeds(&mut self, names: Vec<String>, command: C) -> Result<(), String> {
        self.feed_results.clear();
        self.feeds = Some(State {
            request: Request {
                view: self.windows.selected_id(),
                origin: self.view().clone(),
                command,
            },
            queue: names.into(),
            active: None,
            work: None,
            inbox: command == C::FeedGotoInbox,
            cancelled: false,
        });
        self.next_feed()
    }
    fn next_feed(&mut self) -> Result<(), String> {
        loop {
            let name = self.feeds.as_mut().and_then(|s| s.queue.pop_front());
            let Some(name) = name else {
                self.feeds = None;
                self.feed_summary("Feed update complete");
                return Ok(());
            };
            let choice = Choice::configured(self.buffers.snapshot(), &name)?;
            let state = self
                .feeds
                .as_mut()
                .ok_or("Feed selection no longer active")?;
            state.active = Some(choice.clone());
            match choice.prepare(&self.buffers, state.inbox) {
                Ok(work) => {
                    state.work = Some(work);
                }
                Err(e) => {
                    self.reject_feed(e);
                    continue;
                }
            }
            if !state.inbox && state.active.as_ref().and_then(Choice::program).is_some() {
                return self.confirm_feed();
            }
            return self.launch_feed();
        }
    }
    fn confirm_feed(&mut self) -> Result<(), String> {
        let executable = self
            .feeds
            .as_ref()
            .and_then(|s| s.active.as_ref())
            .and_then(Choice::program)
            .ok_or("No selected feed transport")?;
        self.input
            .argument(
                &format!("Fetch using {executable} with host authority? yes/no: "),
                "",
            )
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Feed(Prompt::Confirm));
        self.message("Confirm configured feed transport");
        Ok(())
    }
    fn launch_feed(&mut self) -> Result<(), String> {
        if !self.feed_current() {
            return Err("Feed source context changed before launch".into());
        }
        let state = self.feeds.as_mut().ok_or("No feed selection")?;
        let work = state.work.take().ok_or("No prepared feed job")?;
        self.org
            .start_feed(state.request.origin.point.stamp, work)?;
        self.message("Feed update running; C-g cancels owned work");
        Ok(())
    }
    pub(in crate::session) fn feed_argument(&mut self, prompt: Prompt, value: &str) -> Action {
        let result = match prompt {
            Prompt::Select(command) => self.select_feeds(vec![value.trim().into()], command),
            Prompt::Confirm => match value.trim() {
                "yes" => self.launch_feed(),
                "no" => return self.cancel_feed(),
                _ => self.confirm_feed(),
            },
            Prompt::Result => return self.inspect_feed(value),
        };
        match result {
            Ok(()) => Action::Redraw,
            Err(e) => {
                self.feeds = None;
                self.error(&e)
            }
        }
    }
    pub(super) fn feed_current(&self) -> bool {
        self.feeds.as_ref().is_some_and(|s| {
            !s.cancelled
                && self.windows.selected_id() == s.request.view
                && self
                    .buffer()
                    .is_ok_and(|b| b.stamp() == s.request.origin.point.stamp)
        })
    }
    pub(super) fn finish_feed(&mut self, result: Result<Plan, String>) {
        let current = self.feed_current();
        let result = if current {
            result.and_then(|plan| self.apply_feed(plan))
        } else {
            Err("Feed context changed or cancelled; target unchanged".into())
        };
        if let Err(e) = result {
            self.reject_feed(e);
        }
        if let Some(state) = &mut self.feeds {
            if state.cancelled {
                self.feeds = None;
                self.feed_summary("Feed update cancelled");
                return;
            }
            state.request.origin = self.windows.selected().clone();
        }
        if let Err(e) = self.next_feed() {
            self.feeds = None;
            self.error(&e);
        }
    }
    fn reject_feed(&mut self, detail: String) {
        if let Some(choice) = self.feeds.as_mut().and_then(|s| s.active.take()) {
            self.feed_results.push(FeedOutcome {
                name: choice.definition.name,
                path: choice.path,
                added: 0,
                skipped: 0,
                publication: Some(Publication::NotPublished),
                detail,
            });
        }
    }
    fn cancel_feed(&mut self) -> Action {
        self.pending = None;
        self.input.cancel();
        if let Some(state) = &mut self.feeds {
            state.cancelled = true;
            state.queue.clear();
        }
        if self.org.feed_running {
            self.org.cancel_feed();
            return self.message("Feed cancellation requested; awaiting owned cleanup");
        }
        self.reject_feed("Feed cancelled before launch".into());
        self.feeds = None;
        self.message("Feed update cancelled before launch; target unchanged")
    }
    fn feed_summary(&mut self, label: &str) -> Action {
        let added: usize = self.feed_results.iter().map(|r| r.added).sum();
        let failed = self
            .feed_results
            .iter()
            .filter(|r| r.publication == Some(Publication::NotPublished))
            .count();
        self.message(&format!(
            "{label}: {added} added, {failed} failed; org-feed-results inspects outcomes"
        ))
    }
    fn feed_result_prompt(&mut self) -> Action {
        if self.feed_results.is_empty() {
            return self.message("No feed outcomes");
        }
        match self.input.argument(
            &format!(
                "Feed result (1..{}; empty summary): ",
                self.feed_results.len()
            ),
            "",
        ) {
            Ok(()) => {
                self.pending = Some(Pending::Feed(Prompt::Result));
                Action::Redraw
            }
            Err(e) => self.error(&e.to_string()),
        }
    }
    fn inspect_feed(&mut self, value: &str) -> Action {
        if value.trim().is_empty() {
            return self.feed_summary("Feed outcomes");
        }
        let Some(result) = value
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| self.feed_results.get(n))
        else {
            return self.error("Choose a recorded feed result number");
        };
        self.message(&format!(
            "Feed {}: {:?}; {} added, {} skipped; {}",
            result.name, result.publication, result.added, result.skipped, result.detail
        ))
    }
}
