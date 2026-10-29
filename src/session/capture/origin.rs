use super::*;
impl Session {
    pub(in crate::session) fn begin_capture(&mut self) -> Action {
        let result = self.capture_origin();
        let origin = match result {
            Ok(origin) => origin,
            Err(e) => return self.error(&e),
        };
        self.org.cancel();
        self.capture = Some(State {
            origin,
            stage: Stage::Template,
        });
        match self.capture_prompt(Prompt::Template) {
            Ok(()) => Action::Redraw,
            Err(e) => {
                self.capture = None;
                self.error(&e)
            }
        }
    }
    fn capture_origin(&self) -> Result<Origin, String> {
        self.admit_capture()?;
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let point = self.view().point;
        let context = self.capture_context(buffer, point)?;
        let directory = match buffer.file().and_then(|f| f.path().parent()) {
            Some(path) => path.to_owned(),
            None => std::env::current_dir().map_err(|e| e.to_string())?,
        };
        Ok(Origin {
            view: self.windows.selected_id(),
            point,
            directory,
            context,
        })
    }
    fn admit_capture(&self) -> Result<(), String> {
        if !self.buffers.snapshot().settings().org.enabled {
            return Err("Org mode is disabled by org.enabled".into());
        }
        if self.capture.is_some() || self.windows.retained_view_active() {
            return Err("Another interaction already retains an editing view".into());
        }
        if self.agenda_active() || self.habit_agenda.is_some() || self.tree.focused() {
            return Err("Return to an editing view before capture".into());
        }
        Ok(())
    }
    fn capture_context(&self, buffer: &Buffer, point: Point) -> Result<Context, String> {
        let policy = &self.buffers.snapshot().settings().org.time;
        let now = self
            .org_clock
            .unwrap_or(super::super::outline::timestamps::capture)(policy)?
        .now();
        let mut context = Context::new(now);
        context.initial = initial(buffer, point)?;
        context.link = capture::origin_link(buffer, point)?.unwrap_or_default();
        context.annotation = context.link.clone();
        if let Some(kill) = self.kills.text() {
            if kill.len_bytes() > capture::VALUE_BYTES {
                return Err("Capture kill text exceeds 256 KiB".into());
            }
            context.kill = kill.to_string();
        }
        Ok(context)
    }
    pub(super) fn capture_current(&self, origin: &Origin) -> bool {
        self.windows.selected_id() == origin.view
            && self
                .windows
                .get(origin.view)
                .is_ok_and(|v| v.point == origin.point)
            && self
                .buffers
                .get(origin.point.stamp.buffer)
                .is_ok_and(|b| b.stamp() == origin.point.stamp)
    }
}

fn initial(buffer: &Buffer, point: Point) -> Result<String, String> {
    let Some(region) = buffer.region(point).map_err(|e| e.to_string())? else {
        return Ok(String::new());
    };
    let text = buffer.slice(&region).map_err(|e| e.to_string())?;
    if text.len_bytes() > capture::VALUE_BYTES {
        return Err("Capture initial selection exceeds 256 KiB".into());
    }
    Ok(text.to_string())
}
