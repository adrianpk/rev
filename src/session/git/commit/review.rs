use super::*;

impl Session {
    pub(in crate::session::git) fn git_commit_review_frame(
        &mut self,
    ) -> Result<Frame, crate::view::Error> {
        let state = self.git.composer.as_ref().expect("commit review");
        if state.family.is_some() {
            return self.git_family_review_frame();
        }
        if state.tag.is_some() {
            return self.git_tag_review_frame();
        }
        let composition = state.composition.as_ref().expect("prepared commit review");
        let (title, target) = match composition.mode {
            crate::git::CommitMode::Amend { head } => ("Amend exact HEAD", head.hex()),
            crate::git::CommitMode::Fixup { target } => ("Fixup exact target", target.hex()),
        };
        let buffer = self.buffers.get(state.buffer)?;
        let text = buffer
            .slice(&buffer.span(0..buffer.len_chars())?)?
            .to_string();
        let checked_head = match &state
            .observation
            .as_ref()
            .expect("commit observation")
            .status
            .as_ref()
            .expect("checked status")
            .head
        {
            Head::Unborn { .. } => "unborn".into(),
            Head::Branch { object, .. } | Head::Detached { object } => object.hex(),
        };
        let metadata = format!(
            "{title}\n{target}\nChecked HEAD: {checked_head}\nMessage: {} bytes; final newline: {}\nAccepted message (Up/Down scroll):",
            text.len(),
            text.ends_with('\n')
        );
        let width = usize::from(self.size.width.max(1));
        let mut lines = metadata
            .lines()
            .chain(text.split('\n'))
            .flat_map(move |line| Wrapped {
                text: label(line.as_bytes()),
                offset: Some(0),
                width,
            })
            .skip(state.top);
        let popup = self.input.palette().and_then(|palette| {
            crate::panel::Composition::navigation(self.size, palette.requested_rows())
        });
        self.input.reveal_commands(
            popup
                .as_ref()
                .and_then(|c| c.auxiliary())
                .map_or(0, |p| p.content().height()),
        );
        let admission = self
            .git
            .engine
            .presentation(self.size.width, self.size.height)?;
        let mut frame = Frame::git(
            buffer,
            self.size,
            self.buffers.palette(),
            &self.echo,
            " Git Commit Review  C-Enter / C-c C-c accept  q / C-g retain draft",
            admission,
            |_| (lines.next().unwrap_or_default(), false),
        )?;
        if let Some(popup) = &popup {
            frame.git_popup(popup)?;
        }
        frame.message_kind(self.echo_kind);
        self.prompt_frame(&mut frame, popup.as_ref())?;
        Ok(frame)
    }
}
