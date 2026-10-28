use super::*;

impl Outline {
    pub(super) fn task_logging(
        &self,
        buffer: &Buffer,
        index: usize,
        states: (&str, &str),
        settings: (&Workflow, &crate::config::Todo),
    ) -> Result<Logging, String> {
        let (workflow, policy) = settings;
        let local = self.task_logging_property(buffer, index, "LOGGING")?;
        let mut workflow = workflow.clone();
        let mut closed = policy.log_done;
        let mut note = false;
        if !local.is_empty() {
            closed = false;
            for state in &mut workflow.states {
                state.enter = Log::None;
                state.leave = Log::None;
            }
            apply(&local, &mut workflow, &mut closed, &mut note)?;
        }
        let state = transition_log(&workflow, states);
        let done = workflow.state(states.1).is_some_and(|s| s.done);
        let drawer = self.task_log_drawer(buffer, index, policy)?;
        Ok(Logging {
            closed: closed && done,
            state,
            note: state == Log::Note || note && done,
            drawer,
        })
    }
    fn task_log_drawer(
        &self,
        buffer: &Buffer,
        index: usize,
        policy: &crate::config::Todo,
    ) -> Result<String, String> {
        let drawer = self.task_logging_property(buffer, index, "LOG_INTO_DRAWER")?;
        let drawer = match drawer.as_str() {
            "" => policy.log_drawer.clone(),
            "nil" => String::new(),
            "t" => "LOGBOOK".into(),
            _ => drawer,
        };
        if !drawer.is_empty() && !crate::org::task::drawer_name(&drawer) {
            return Err("Invalid Org task log drawer".into());
        }
        Ok(drawer)
    }
    pub(super) fn task_logging_property(
        &self,
        buffer: &Buffer,
        index: usize,
        name: &str,
    ) -> Result<String, String> {
        let point = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .from_byte(crate::coordinates::Byte(self.headings[index].start))
            .map_err(|e| e.to_string())?;
        let entry = self.property_target(buffer, point)?;
        self.property_value(buffer, entry, name, &[name.into()])
            .map(|p| p.text)
    }
}
fn apply(
    text: &str,
    workflow: &mut Workflow,
    closed: &mut bool,
    note: &mut bool,
) -> Result<(), String> {
    for token in text.split_whitespace() {
        match token {
            "nil" | "logrepeat" | "lognoterepeat" | "nologrepeat" => (),
            "logdone" => *closed = true,
            "lognotedone" => {
                *closed = true;
                *note = true;
            }
            _ => apply_state(token, workflow)?,
        }
    }
    Ok(())
}
fn apply_state(token: &str, workflow: &mut Workflow) -> Result<(), String> {
    let mut declaration = Workflow::default();
    declaration.declaration(token);
    if declaration.invalid {
        return Err("Unsupported Org LOGGING property".into());
    }
    let parsed = &declaration.states[0];
    let state = workflow
        .states
        .iter_mut()
        .find(|s| s.name == parsed.name)
        .ok_or("Unknown state in Org LOGGING property")?;
    state.enter = parsed.enter;
    state.leave = parsed.leave;
    Ok(())
}

fn transition_log(workflow: &Workflow, states: (&str, &str)) -> Log {
    let enter = workflow.state(states.1).map_or(Log::None, |s| s.enter);
    if enter == Log::None {
        workflow.state(states.0).map_or(Log::None, |s| s.leave)
    } else {
        enter
    }
}
