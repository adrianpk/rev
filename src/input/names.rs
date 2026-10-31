use crossterm::event::KeyCode;

use super::{Context, Error, Input, Key, Outcome, Prompt};

/// A bounded snapshot of bookmark names, independent of document text. The
/// session revalidates the accepted name against its canonical collection.
#[derive(Debug)]
pub(crate) struct Names {
    prompt: Prompt,
    names: Vec<String>,
    query: Option<String>,
    selected: usize,
    chosen: bool,
    top: usize,
    policy: crate::config::Minibuffer,
}

impl Names {
    pub(crate) fn prompt(&self) -> &Prompt {
        &self.prompt
    }
    pub(crate) fn candidates(&self) -> Vec<&str> {
        let query = self.query.as_deref().unwrap_or(self.prompt.text());
        let mut names: Vec<_> = self
            .names
            .iter()
            .map(String::as_str)
            .filter(|name| match self.policy.r#match {
                crate::config::Match::Prefix => name.starts_with(query),
                _ => name.contains(query),
            })
            .collect();
        names.sort_by_key(|name| {
            (
                self.policy.r#match == crate::config::Match::PrefixThenSubstring
                    && !name.starts_with(query),
                *name,
            )
        });
        names
    }
    pub(crate) fn selected(&self) -> Option<usize> {
        (self.selected < self.candidates().len()).then_some(self.selected)
    }
    pub(crate) fn top(&self) -> usize {
        self.top
    }
    pub(crate) fn requested_rows(&self) -> u16 {
        self.candidates()
            .len()
            .max(1)
            .min(self.policy.candidate_limit) as u16
    }
    fn reveal(&mut self, rows: u16) {
        if rows == 0 {
            return;
        }
        let rows = usize::from(rows);
        if self.selected < self.top {
            self.top = self.selected;
        } else if self.selected >= self.top + rows {
            self.top = self.selected + 1 - rows;
        }
        self.top = self.top.min(self.candidates().len().saturating_sub(rows));
    }
    fn handle(&mut self, key: Key) -> Outcome {
        let direction = match key.plain() {
            Some(KeyCode::Up) => Some(true),
            Some(KeyCode::Down) => Some(false),
            _ if key.control('p') => Some(true),
            _ if key.control('n') => Some(false),
            _ => None,
        };
        if let Some(previous) = direction {
            let count = self.candidates().len();
            if count == 0 {
                return Outcome::Message("No matching bookmarks");
            }
            self.selected = if previous {
                self.selected.saturating_sub(1)
            } else {
                (self.selected + 1).min(count - 1)
            };
            self.chosen = true;
            return Outcome::Changed;
        }
        match key.plain() {
            Some(KeyCode::Tab) => self.complete(),
            Some(KeyCode::Enter) => {
                let name = if self.chosen {
                    self.candidates()
                        .get(self.selected)
                        .copied()
                        .unwrap_or(self.prompt.text())
                } else {
                    self.prompt.text()
                };
                Outcome::Argument(name.into())
            }
            _ => {
                let before = self.prompt.text().to_owned();
                let outcome = self.prompt.handle(key);
                self.edited(&before);
                outcome
            }
        }
    }
    fn complete(&mut self) -> Outcome {
        let candidates = self.candidates();
        let Some(name) = candidates.get(self.selected) else {
            return Outcome::Message("No matching bookmarks");
        };
        let name = (*name).to_owned();
        let query = self
            .query
            .clone()
            .unwrap_or_else(|| self.prompt.text().to_owned());
        match self.prompt.replace(&name) {
            Ok(()) => {
                self.query = Some(query);
                Outcome::Changed
            }
            Err(_) => Outcome::Message(super::prompt::LIMIT_MESSAGE),
        }
    }
    pub(super) fn paste(&mut self, text: &str) -> Outcome {
        let before = self.prompt.text().to_owned();
        let outcome = self.prompt.paste(text);
        self.edited(&before);
        outcome
    }
    fn edited(&mut self, before: &str) {
        if self.prompt.text() != before {
            self.query = None;
            self.selected = 0;
            self.chosen = false;
            self.top = 0;
        }
    }
}

impl Input {
    pub(crate) fn names_argument(
        &mut self,
        label: &str,
        initial: &str,
        names: Vec<String>,
    ) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Names(Names {
            prompt: Prompt::new(label, initial, self.policy.input_limit)?,
            names,
            query: None,
            selected: 0,
            chosen: false,
            top: 0,
            policy: self.policy.clone(),
        });
        self.clear_prefix();
        Ok(())
    }
    pub(crate) fn names(&self) -> Option<&Names> {
        match &self.context {
            Context::Names(names) => Some(names),
            _ => None,
        }
    }
    pub(crate) fn reveal_names(&mut self, rows: u16) {
        if let Context::Names(names) = &mut self.context {
            names.reveal(rows);
        }
    }
    pub(super) fn names_key(&mut self, key: Key) -> Outcome {
        let Context::Names(names) = &mut self.context else {
            return Outcome::Ignored;
        };
        names.handle(key)
    }
}
