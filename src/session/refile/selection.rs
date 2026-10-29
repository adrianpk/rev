use super::*;
use crate::org::agenda::SourceVersion;
impl Session {
    pub(super) fn refile_selection(&self) -> Result<SourceSelection, String> {
        let snapshot = self.buffers.snapshot();
        let paths = &snapshot.settings().org.refile.targets;
        if !paths.is_empty() {
            return declared(snapshot, paths);
        }
        let mut selection = SourceSelection::configured(snapshot)?;
        selection.buffers = self
            .buffers
            .iter()
            .filter(|b| b.is_org())
            .map(|b| b.id())
            .collect();
        if let Some(sources) = &self.agenda_sources {
            for source in &sources.documents {
                match source.version {
                    SourceVersion::Buffer(stamp) => selection.buffers.push(stamp.buffer),
                    SourceVersion::Disk(_) => selection.paths.extend(source.path.iter().cloned()),
                }
            }
        }
        selection.buffers.sort();
        selection.buffers.dedup();
        selection.paths.sort();
        selection.paths.dedup();
        Ok(selection)
    }
    pub(super) fn prompt_refile(&mut self, draft: &str) -> Result<Action, String> {
        let state = self.refile.as_ref().ok_or("No refile selection")?;
        if self.input.focus() != Focus::Editing
            || self.pending.is_some()
            || !self.org_source_current(&state.request)
        {
            return Err("Refile source, view or input focus changed".into());
        }
        self.input
            .argument("Refile target [number/label]: ", draft)
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Refile);
        Ok(self.complete_refile())
    }
    pub(in crate::session) fn complete_refile(&mut self) -> Action {
        let Some(targets) = self.refile.as_ref().and_then(|s| s.targets.as_ref()) else {
            return self.message("No refile targets are active");
        };
        let draft = self.input.prompt().map(|p| p.text()).unwrap_or("");
        let choices = targets
            .headings
            .iter()
            .enumerate()
            .filter(|(_, h)| h.label.contains(draft))
            .take(8)
            .map(|(i, h)| format!("{}: {}", i + 1, h.label))
            .collect::<Vec<_>>()
            .join("; ");
        self.message(&format!(
            "{} targets; {choices}; TAB filters labels; C-g cancels",
            targets.headings.len()
        ))
    }
    pub(in crate::session) fn refile_argument(&mut self, draft: &str) -> Action {
        let Some(mut state) = self.refile.take() else {
            return self.message("No refile request");
        };
        if !self.org_source_current(&state.request) {
            self.fail_refile("Source or view changed");
            return Action::Redraw;
        }
        let targets = state
            .targets
            .take()
            .expect("selection owns discovered targets");
        let index = match targets.select(draft.trim()) {
            Ok(index) => index,
            Err(error) => {
                state.targets = Some(targets);
                self.refile = Some(state);
                if let Err(error) = self.prompt_refile(draft) {
                    self.fail_refile(&error);
                    return Action::Redraw;
                }
                return self.error(&error);
            }
        };
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        let result = self.org.start_refile_plan(
            targets,
            state.subtree.clone(),
            (index, state.copy),
            seconds,
        );
        if let Err(error) = result {
            self.fail_refile(&error);
            return Action::Redraw;
        }
        self.refile = Some(state);
        self.message("Refile validation pending; source unchanged; C-g cancels")
    }
}

fn declared(
    snapshot: &crate::config::Snapshot,
    paths: &[String],
) -> Result<SourceSelection, String> {
    let mut selection = SourceSelection::paths(paths)?;
    if let Some(root) = snapshot
        .origin(&["org", "refile", "targets"])
        .and_then(|p| p.parent())
    {
        for path in &mut selection.paths {
            if path.is_relative() {
                *path = root.join(&*path);
            }
        }
    }
    Ok(selection)
}
