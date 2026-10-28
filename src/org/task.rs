//! Bounded task declarations shared by source editing, cookies and matching.
mod declaration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Log {
    #[default]
    None,
    Time,
    Note,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct State {
    pub name: String,
    pub done: bool,
    pub key: Option<char>,
    pub enter: Log,
    pub leave: Log,
    pub sequence: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Workflow {
    pub states: Vec<State>,
    pub declared: bool,
    pub invalid: bool,
}
impl Workflow {
    pub fn configured(policy: &crate::config::Todo) -> Result<Self, String> {
        let mut workflow = Self::default();
        for word in &policy.sequence {
            workflow.push(State {
                name: word.clone(),
                done: policy.finished.contains(word),
                key: None,
                enter: Log::None,
                leave: Log::None,
                sequence: 0,
            })?;
        }
        if workflow.states.is_empty()
            || policy
                .finished
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != policy.finished.len()
            || !policy.finished.iter().all(|s| policy.sequence.contains(s))
        {
            return Err(
                "Task sequence must contain 1..64 unique keywords and every finished state".into(),
            );
        }
        Ok(workflow)
    }
    pub fn state(&self, word: &str) -> Option<&State> {
        self.states.iter().find(|state| state.name == word)
    }
    pub fn effective(&self, policy: &crate::config::Todo) -> Result<Self, String> {
        if self.invalid {
            return Err("Malformed or excessive Org task declaration".into());
        }
        if self.declared {
            Ok(self.clone())
        } else {
            Self::configured(policy)
        }
    }
    pub fn next(&self, old: &str, backwards: bool) -> String {
        let current = self.state(old);
        let sequence = current.map_or(0, |state| state.sequence);
        let states: Vec<_> = self
            .states
            .iter()
            .filter(|s| s.sequence == sequence)
            .collect();
        let index = states.iter().position(|s| s.name == old);
        let next = if backwards {
            index.map_or(states.len().checked_sub(1), |i| i.checked_sub(1))
        } else {
            index.map_or(Some(0), |i| (i + 1 < states.len()).then_some(i + 1))
        };
        next.and_then(|i| states.get(i))
            .map_or_else(String::new, |s| s.name.clone())
    }
    fn push(&mut self, state: State) -> Result<(), String> {
        if self.states.len() == 64
            || !super::query::task_keyword(&state.name)
            || self.state(&state.name).is_some()
        {
            return Err("Task sequence requires at most 64 unique valid keywords".into());
        }
        if state
            .key
            .is_some_and(|key| self.states.iter().any(|s| s.key == Some(key)))
        {
            return Err("Duplicate Org task selection key".into());
        }
        self.states.push(state);
        Ok(())
    }
}

pub(crate) fn drawer_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        && !name.eq_ignore_ascii_case("PROPERTIES")
        && !name.eq_ignore_ascii_case("END")
}
