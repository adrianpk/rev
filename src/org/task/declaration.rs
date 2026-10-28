use super::{Log, State, Workflow};

impl Workflow {
    pub fn declaration(&mut self, value: &str) {
        self.declared = true;
        if self.parse(value).is_err() {
            self.invalid = true;
        }
    }
    fn parse(&mut self, value: &str) -> Result<(), String> {
        let start = self.states.len();
        let sequence = self.states.last().map_or(0, |s| s.sequence + 1);
        let mut finished = false;
        for word in value.split_whitespace().take(67) {
            if word == "|" {
                if finished {
                    return Err("Repeated task separator".into());
                }
                finished = true;
            } else {
                self.push(state(word, finished, sequence)?)?;
            }
        }
        if start == self.states.len() {
            return Err("Empty task declaration".into());
        }
        if !finished {
            self.states.last_mut().expect("nonempty").done = true;
        }
        Ok(())
    }
}
fn state(token: &str, done: bool, sequence: usize) -> Result<State, String> {
    let (name, annotation) = match token.split_once('(') {
        Some((name, tail)) => (
            name,
            tail.strip_suffix(')').ok_or("Unclosed task annotation")?,
        ),
        None => (token, ""),
    };
    let (enter, leave) = annotation.split_once('/').unwrap_or((annotation, ""));
    let mut chars = enter.chars();
    let key = chars.next().filter(|c| !matches!(c, '!' | '@'));
    let enter = if key.is_some() { chars.as_str() } else { enter };
    Ok(State {
        name: name.into(),
        done,
        key,
        enter: log(enter)?,
        leave: log(leave)?,
        sequence,
    })
}
fn log(annotation: &str) -> Result<Log, String> {
    match annotation {
        "" => Ok(Log::None),
        "!" => Ok(Log::Time),
        "@" => Ok(Log::Note),
        _ => Err("Invalid task logging annotation".into()),
    }
}
